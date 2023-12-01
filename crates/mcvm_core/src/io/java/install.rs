use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use tar::Archive;

use crate::io::files::{self, paths::Paths};
use crate::io::persistent::{PersistentData, PersistentDataJavaInstallation};
use crate::io::update::UpdateManager;
use crate::net::{self, download};
use mcvm_shared::util::preferred_archive_extension;

use super::JavaMajorVersion;

/// Type of Java installation
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum JavaInstallationKind {
	/// Automatically chooses different Java
	/// flavors based on system conditions
	Auto,
	/// Trys to use a Java installation that is
	/// already on the system
	System,
	/// Adoptium
	Adoptium,
	/// Azul Zulu
	Zulu,
	/// A user-specified installation
	Custom {
		/// The path to the installation. The JVM must live at
		/// `{path}/bin/java`
		path: PathBuf,
	},
}

impl JavaInstallationKind {
	/// Parse a string into a JavaKind
	pub fn parse(string: &str) -> Self {
		match string {
			"auto" => Self::Auto,
			"system" => Self::System,
			"adoptium" => Self::Adoptium,
			"zulu" => Self::Zulu,
			path => Self::Custom {
				path: PathBuf::from(path),
			},
		}
	}
}

/// A Java installation used to launch the game
#[derive(Debug, Clone)]
pub struct JavaInstallation {
	/// The major version of the Java installation
	major_version: JavaMajorVersion,
	/// The path to the directory where the installation is, which will be filled when it is installed
	path: PathBuf,
}

impl JavaInstallation {
	/// Load a new Java installation
	pub(crate) async fn install(
		kind: JavaInstallationKind,
		major_version: JavaMajorVersion,
		mut params: JavaInstallParameters<'_>,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Self> {
		o.start_process();
		o.display(
			MessageContents::StartProcess("Checking for Java updates".into()),
			MessageLevel::Important,
		);

		let vers_str = major_version.to_string();

		let path = match &kind {
			JavaInstallationKind::Auto => install_auto(&vers_str, params, o).await?,
			JavaInstallationKind::System => install_system(&vers_str)?,
			JavaInstallationKind::Adoptium => install_adoptium(&vers_str, &mut params, o).await?,
			JavaInstallationKind::Zulu => install_zulu(&vers_str, &mut params, o).await?,
			JavaInstallationKind::Custom { path } => path.clone(),
		};

		o.display(
			MessageContents::Success("Java updated".into()),
			MessageLevel::Important,
		);

		let out = Self {
			major_version,
			path,
		};

		Ok(out)
	}

	/// Get the major version of the Java installation
	pub fn get_major_version(&self) -> &JavaMajorVersion {
		&self.major_version
	}

	/// Get the path to the Java installation
	pub fn get_path(&self) -> &Path {
		&self.path
	}

	/// Get the path to the JVM.
	pub fn get_jvm_path(&self) -> PathBuf {
		#[cfg(target_family = "windows")]
		let path = "bin/java.exe";
		#[cfg(not(target_family = "windows"))]
		let path = "bin/java";
		self.path.join(path)
	}
}

/// Container struct for parameters for loading Java installations
pub(crate) struct JavaInstallParameters<'a> {
	pub paths: &'a Paths,
	pub update_manager: &'a mut UpdateManager,
	pub persistent: &'a mut PersistentData,
	pub req_client: &'a reqwest::Client,
}

async fn install_auto(
	major_version: &str,
	mut params: JavaInstallParameters<'_>,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<PathBuf> {
	let out = install_system(major_version);
	if let Ok(out) = out {
		return Ok(out);
	}
	let out = install_adoptium(major_version, &mut params, o).await;
	if let Ok(out) = out {
		return Ok(out);
	}
	let out = install_zulu(major_version, &mut params, o).await;
	if let Ok(out) = out {
		return Ok(out);
	}
	bail!("Failed to automatically install Java")
}

fn install_system(major_version: &str) -> anyhow::Result<PathBuf> {
	let installation = get_system_java_installation(major_version);
	if let Some(installation) = installation {
		Ok(installation)
	} else {
		bail!("No valid system Java installation was found");
	}
}

async fn install_adoptium(
	major_version: &str,
	params: &mut JavaInstallParameters<'_>,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<PathBuf> {
	if params.update_manager.allow_offline {
		if let Some(directory) = params
			.persistent
			.get_java_path(PersistentDataJavaInstallation::Adoptium, major_version)
		{
			Ok(directory)
		} else {
			update_adoptium(major_version, params, o)
				.await
				.context("Failed to update Adoptium Java")
		}
	} else {
		update_adoptium(major_version, params, o)
			.await
			.context("Failed to update Adoptium Java")
	}
}

async fn install_zulu(
	major_version: &str,
	params: &mut JavaInstallParameters<'_>,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<PathBuf> {
	if params.update_manager.allow_offline {
		if let Some(directory) = params
			.persistent
			.get_java_path(PersistentDataJavaInstallation::Zulu, major_version)
		{
			Ok(directory)
		} else {
			update_zulu(major_version, params, o)
				.await
				.context("Failed to update Zulu Java")
		}
	} else {
		update_zulu(major_version, params, o)
			.await
			.context("Failed to update Zulu Java")
	}
}

/// Updates Adoptium and returns the path to the installation
async fn update_adoptium(
	major_version: &str,
	params: &mut JavaInstallParameters<'_>,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<PathBuf> {
	let out_dir = params.paths.java.join("adoptium");
	files::create_dir(&out_dir)?;
	let version = net::java::adoptium::get_latest(major_version, params.req_client)
		.await
		.context("Failed to obtain Adoptium information")?;

	let release_name = version.release_name.clone();
	let mut extracted_bin_name = release_name.clone();
	extracted_bin_name.push_str("-jre");
	let extracted_bin_dir = out_dir.join(&extracted_bin_name);

	if !params
		.persistent
		.update_java_installation(
			PersistentDataJavaInstallation::Adoptium,
			major_version,
			&release_name,
			&extracted_bin_dir,
		)
		.context("Failed to update Java in lockfile")?
	{
		return Ok(extracted_bin_dir);
	}

	params.persistent.dump(params.paths).await?;

	let arc_extension = preferred_archive_extension();
	let arc_name = format!("adoptium{major_version}{arc_extension}");
	let arc_path = out_dir.join(arc_name);

	let bin_url = version.binary.package.link;

	o.display(
		MessageContents::StartProcess(format!(
			"Downloading Adoptium Temurin JRE version {release_name}"
		)),
		MessageLevel::Important,
	);
	download::file(bin_url, &arc_path, params.req_client)
		.await
		.context("Failed to download JRE binaries")?;

	// Extraction
	o.display(
		MessageContents::StartProcess("Extracting JRE".into()),
		MessageLevel::Important,
	);
	extract_archive(&arc_path, &out_dir).context("Failed to extract")?;
	o.display(
		MessageContents::StartProcess("Removing archive".into()),
		MessageLevel::Important,
	);
	std::fs::remove_file(arc_path).context("Failed to remove archive")?;

	o.display(
		MessageContents::Success("Java installation finished".into()),
		MessageLevel::Important,
	);
	o.end_process();

	Ok(extracted_bin_dir)
}

/// Updates Zulu and returns the path to the installation
async fn update_zulu(
	major_version: &str,
	params: &mut JavaInstallParameters<'_>,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<PathBuf> {
	let out_dir = params.paths.java.join("zulu");
	files::create_dir(&out_dir)?;

	let package = net::java::zulu::get_latest(major_version, params.req_client)
		.await
		.context("Failed to get the latest Zulu version")?;

	let extracted_dir = out_dir.join(net::java::zulu::extract_dir_name(&package.name));

	if !params
		.persistent
		.update_java_installation(
			PersistentDataJavaInstallation::Zulu,
			major_version,
			&package.name,
			&extracted_dir,
		)
		.context("Failed to update Java in lockfile")?
	{
		return Ok(extracted_dir);
	}

	params.persistent.dump(params.paths).await?;

	let arc_path = out_dir.join(&package.name);

	o.display(
		MessageContents::StartProcess(format!(
			"Downloading Azul Zulu JRE version {}",
			package.name
		)),
		MessageLevel::Important,
	);
	download::file(&package.download_url, &arc_path, params.req_client)
		.await
		.context("Failed to download JRE binaries")?;

	// Extraction
	o.display(
		MessageContents::StartProcess("Extracting JRE".into()),
		MessageLevel::Important,
	);
	extract_archive(&arc_path, &out_dir).context("Failed to extract")?;
	o.display(
		MessageContents::StartProcess("Removing archive".into()),
		MessageLevel::Important,
	);
	std::fs::remove_file(arc_path).context("Failed to remove archive")?;

	o.display(
		MessageContents::Success("Java installation finished".into()),
		MessageLevel::Important,
	);
	o.end_process();

	Ok(extracted_dir)
}

/// Extracts the Adoptium/Zulu JRE archive (either a tar or a zip)
fn extract_archive(arc_path: &Path, out_dir: &Path) -> anyhow::Result<()> {
	let file = File::open(arc_path).context("Failed to read archive file")?;
	let mut file = BufReader::new(file);
	if cfg!(windows) {
		zip_extract::extract(&mut file, out_dir, false).context("Failed to extract zip file")?;
	} else {
		let mut decoder =
			libflate::gzip::Decoder::new(&mut file).context("Failed to decode tar.gz")?;
		let mut arc = Archive::new(&mut decoder);
		arc.unpack(out_dir).context("Failed to unarchive tar")?;
	}

	Ok(())
}

/// Gets the optimal path to a system Java installation
fn get_system_java_installation(#[allow(unused_variables)] major_version: &str) -> Option<PathBuf> {
	#[cfg(target_os = "windows")]
	{
		// OpenJDK
		let dir = PathBuf::from("C:/Program Files/Java");
		if dir.exists() {
			let read = std::fs::read_dir(dir);
			if let Ok(read) = read {
				for path in read {
					let Ok(path) = path else { continue };
					if !path.path().is_dir() {
						continue;
					}
					let name = path.file_name().to_string_lossy().to_string();
					if !name.starts_with("jdk-") {
						continue;
					}
					if !name.contains(&format!("-{major_version}.")) {
						continue;
					}
					return Some(path.path());
				}
			}
		}
	}
	#[cfg(target_os = "linux")]
	{
		// OpenJDK
		let dir = PathBuf::from("/usr/lib/jvm");
		if dir.exists() {
			let read = std::fs::read_dir(dir);
			if let Ok(read) = read {
				for path in read {
					let Ok(path) = path else { continue };
					if !path.path().is_dir() {
						continue;
					}
					let name = path.file_name().to_string_lossy().to_string();
					if !name.starts_with("java-") {
						continue;
					}
					if !name.contains(&format!("-{major_version}-")) {
						continue;
					}
					return Some(path.path());
				}
			}
		}
	}
	None
}
