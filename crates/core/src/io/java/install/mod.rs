/// System Java installation
mod system;

use std::fs::File;
use std::io::{BufReader, Read, Seek};

#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::translate;
use tar::Archive;
use zip::ZipArchive;

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
	/// GraalVM
	GraalVM,
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
			"graalvm" => Self::GraalVM,
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
			MessageContents::StartProcess(translate!(o, StartCheckingForJavaUpdates)),
			MessageLevel::Important,
		);

		let vers_str = major_version.to_string();

		let path = match &kind {
			JavaInstallationKind::Auto => install_auto(&vers_str, params, o).await?,
			JavaInstallationKind::System => system::install(&vers_str)?,
			JavaInstallationKind::Adoptium => install_adoptium(&vers_str, &mut params, o).await?,
			JavaInstallationKind::Zulu => install_zulu(&vers_str, &mut params, o).await?,
			JavaInstallationKind::GraalVM => install_graalvm(&vers_str, &mut params, o).await?,
			JavaInstallationKind::Custom { path } => path.clone(),
		};

		o.display(
			MessageContents::Success(translate!(o, FinishCheckingForJavaUpdates)),
			MessageLevel::Important,
		);

		o.end_process();

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

	/// Verifies that this installation is set up correctly
	pub fn verify(&self) -> anyhow::Result<bool> {
		let jvm_path = self.get_jvm_path();
		if !jvm_path.exists() || !jvm_path.is_file() {
			return Ok(false);
		}
		#[cfg(target_family = "unix")]
		{
			// Check if JVM is executable
			let mode = jvm_path
				.metadata()
				.context("Failed to get JVM metadata")?
				.permissions()
				.mode();
			if mode & 0o111 == 0 {
				return Ok(false);
			}
		}

		Ok(true)
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
	let out = system::install(major_version);
	if let Ok(out) = out {
		return Ok(out);
	}
	let out = install_adoptium(major_version, &mut params, o).await;
	if let Ok(out) = out {
		return Ok(out);
	}
	let out = install_graalvm(major_version, &mut params, o).await;
	if let Ok(out) = out {
		return Ok(out);
	}
	let out = install_zulu(major_version, &mut params, o).await;
	if let Ok(out) = out {
		return Ok(out);
	}
	bail!("Failed to automatically install Java")
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

async fn install_graalvm(
	major_version: &str,
	params: &mut JavaInstallParameters<'_>,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<PathBuf> {
	if params.update_manager.allow_offline {
		if let Some(directory) = params
			.persistent
			.get_java_path(PersistentDataJavaInstallation::GraalVM, major_version)
		{
			Ok(directory)
		} else {
			update_graalvm(major_version, params, o)
				.await
				.context("Failed to update GraalVM Java")
		}
	} else {
		update_graalvm(major_version, params, o)
			.await
			.context("Failed to update GraalVM Java")
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
		MessageContents::StartProcess(translate!(
			o,
			DownloadingAdoptium,
			"version" = &release_name
		)),
		MessageLevel::Important,
	);
	download::file(bin_url, &arc_path, params.req_client)
		.await
		.context("Failed to download JRE binaries")?;

	// Extraction
	o.display(
		MessageContents::StartProcess(translate!(o, StartExtractingJava)),
		MessageLevel::Important,
	);
	extract_archive_file(&arc_path, &out_dir).context("Failed to extract")?;
	o.display(
		MessageContents::StartProcess(translate!(o, StartRemovingJavaArchive)),
		MessageLevel::Important,
	);
	std::fs::remove_file(arc_path).context("Failed to remove archive")?;

	o.display(
		MessageContents::Success(translate!(o, FinishJavaInstallation)),
		MessageLevel::Important,
	);

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
		MessageContents::StartProcess(translate!(o, DownloadingZulu, "version" = &package.name)),
		MessageLevel::Important,
	);
	download::file(&package.download_url, &arc_path, params.req_client)
		.await
		.context("Failed to download JRE binaries")?;

	// Extraction
	o.display(
		MessageContents::StartProcess(translate!(o, StartExtractingJava)),
		MessageLevel::Important,
	);
	extract_archive_file(&arc_path, &out_dir).context("Failed to extract")?;
	o.display(
		MessageContents::StartProcess(translate!(o, StartRemovingJavaArchive)),
		MessageLevel::Important,
	);
	std::fs::remove_file(arc_path).context("Failed to remove archive")?;

	o.display(
		MessageContents::Success(translate!(o, FinishJavaInstallation)),
		MessageLevel::Important,
	);

	Ok(extracted_dir)
}

/// Updates GraalVM and returns the path to the installation
async fn update_graalvm(
	major_version: &str,
	params: &mut JavaInstallParameters<'_>,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<PathBuf> {
	let out_dir = params.paths.java.join("graalvm");
	files::create_dir(&out_dir)?;

	o.display(
		MessageContents::StartProcess(translate!(o, DownloadingGraalVM)),
		MessageLevel::Important,
	);
	let archive = net::java::graalvm::get_latest(major_version, params.req_client)
		.await
		.context("Failed to download the latest GraalVM version")?;

	// We have to extract now since we need the extracted dir
	o.display(
		MessageContents::StartProcess(translate!(o, StartExtractingJava)),
		MessageLevel::Important,
	);
	let dir_name = extract_archive(std::io::Cursor::new(archive), &out_dir)
		.context("Failed to extract GraalVM archive")?;

	let extracted_dir = out_dir.join(&dir_name);

	let version = dir_name.replace("graalvm-jdk-", "");

	if !params
		.persistent
		.update_java_installation(
			PersistentDataJavaInstallation::GraalVM,
			major_version,
			&version,
			&extracted_dir,
		)
		.context("Failed to update Java in lockfile")?
	{
		return Ok(extracted_dir);
	}

	params.persistent.dump(params.paths).await?;

	o.display(
		MessageContents::Success(translate!(o, FinishJavaInstallation)),
		MessageLevel::Important,
	);

	Ok(extracted_dir)
}

/// Extracts the archive file
fn extract_archive_file(arc_path: &Path, out_dir: &Path) -> anyhow::Result<()> {
	let file = File::open(arc_path).context("Failed to read archive file")?;
	let file = BufReader::new(file);

	extract_archive(file, out_dir)?;

	Ok(())
}

/// Extracts the JRE archive (either a tar or a zip) and also returns the internal extraction directory name
fn extract_archive<R: Read + Seek>(reader: R, out_dir: &Path) -> anyhow::Result<String> {
	let dir_name = if cfg!(windows) {
		let mut archive = ZipArchive::new(reader).context("Failed to open zip archive")?;

		let dir_name = archive
			.file_names()
			.next()
			.context("Missing archive internal directory")?
			.to_string();

		archive
			.extract(out_dir)
			.context("Failed to extract zip file")?;

		dir_name
	} else {
		let mut decoder =
			libflate::gzip::Decoder::new(reader).context("Failed to decode tar.gz")?;
		// Get the archive twice because of archive shenanigans
		let mut arc = Archive::new(&mut decoder);

		// Wow
		let dir_name = arc
			.entries()
			.context("Failed to get Tar entries")?
			.next()
			.context("Missing archive internal directory")?
			.context("Failed to get entry")?
			.path()
			.context("Failed to get entry path name")?
			.to_string_lossy()
			.to_string();

		let mut arc = Archive::new(&mut decoder);
		arc.unpack(out_dir).context("Failed to unarchive tar")?;

		dir_name
	};

	Ok(dir_name)
}
