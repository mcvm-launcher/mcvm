use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use mcvm_shared::later::Later;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use reqwest::Client;
use tar::Archive;

use crate::io::files::{self, paths::Paths};
use crate::io::persistent::{PersistentData, PersistentDataJavaInstallation};
use crate::io::update::{UpdateManager, UpdateMethodResult};
use crate::net::{self, download};
use mcvm_shared::util::preferred_archive_extension;

/// Type of Java installation
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum JavaInstallationKind {
	/// Automatically chooses different Java
	/// flavors based on system conditions
	Auto(Later<String>),
	/// System Java
	System(Later<String>),
	/// Adoptium
	Adoptium(Later<String>),
	/// Azul Zulu
	Zulu(Later<String>),
	/// A user-specified installation
	Custom(PathBuf),
}

impl JavaInstallationKind {
	/// Parse a string into a JavaKind
	pub fn parse(string: &str) -> Self {
		match string {
			"system" => Self::System(Later::Empty),
			"adoptium" => Self::Adoptium(Later::Empty),
			"zulu" => Self::Zulu(Later::Empty),
			path => Self::Custom(PathBuf::from(path)),
		}
	}
}

/// A Java installation used to launch the game
#[derive(Debug, Clone)]
pub struct JavaInstallation {
	kind: JavaInstallationKind,
	/// The path to the directory where the installation is, which will be filled when it is installed
	pub path: Later<PathBuf>,
}

impl JavaInstallation {
	/// Create a new Java
	pub fn new(kind: JavaInstallationKind) -> Self {
		Self {
			kind,
			path: Later::Empty,
		}
	}

	/// Add a major version to a Java installation that supports it
	pub fn add_version(&mut self, version: &str) {
		match &mut self.kind {
			JavaInstallationKind::Auto(vers)
			| JavaInstallationKind::System(vers)
			| JavaInstallationKind::Adoptium(vers)
			| JavaInstallationKind::Zulu(vers) => vers.fill(version.to_owned()),
			JavaInstallationKind::Custom(..) => {}
		};
	}

	/// Download / install all needed files
	pub async fn install(
		&mut self,
		paths: &Paths,
		manager: &UpdateManager,
		lock: &mut PersistentData,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<UpdateMethodResult> {
		let out = UpdateMethodResult::new();

		o.start_process();
		o.display(
			MessageContents::StartProcess("Checking for Java updates".into()),
			MessageLevel::Important,
		);

		match &self.kind {
			JavaInstallationKind::Auto(major_version) => {
				self.path.fill(
					install_auto(major_version.get(), paths, manager, lock, client, o).await?,
				);
			}
			JavaInstallationKind::System(major_version) => {
				self.path.fill(install_system(major_version.get())?);
			}
			JavaInstallationKind::Adoptium(major_version) => {
				self.path.fill(
					install_adoptium(major_version.get(), paths, manager, lock, client, o).await?,
				);
			}
			JavaInstallationKind::Zulu(major_version) => {
				self.path.fill(
					install_zulu(major_version.get(), paths, manager, lock, client, o).await?,
				);
			}
			JavaInstallationKind::Custom(path) => {
				self.path.fill(path.clone());
			}
		}
		o.display(
			MessageContents::Success("Java updated".into()),
			MessageLevel::Important,
		);

		Ok(out)
	}

	/// Get the path to the JVM. Will panic if not installed.
	pub fn get_jvm_path(&self) -> PathBuf {
		self.path.get().join("bin/java")
	}
}

async fn install_auto(
	major_version: &str,
	paths: &Paths,
	manager: &UpdateManager,
	lock: &mut PersistentData,
	client: &Client,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<PathBuf> {
	let out = install_system(major_version);
	if let Ok(out) = out {
		return Ok(out);
	}
	let out = install_adoptium(major_version, paths, manager, lock, client, o).await;
	if let Ok(out) = out {
		return Ok(out);
	}
	let out = install_zulu(major_version, paths, manager, lock, client, o).await;
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
	paths: &Paths,
	manager: &UpdateManager,
	lock: &mut PersistentData,
	client: &Client,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<PathBuf> {
	if manager.allow_offline {
		if let Some(directory) =
			lock.get_java_path(PersistentDataJavaInstallation::Adoptium, major_version)
		{
			Ok(directory)
		} else {
			update_adoptium(major_version, lock, paths, client, o)
				.await
				.context("Failed to update Adoptium Java")
		}
	} else {
		update_adoptium(major_version, lock, paths, client, o)
			.await
			.context("Failed to update Adoptium Java")
	}
}

async fn install_zulu(
	major_version: &str,
	paths: &Paths,
	manager: &UpdateManager,
	lock: &mut PersistentData,
	client: &Client,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<PathBuf> {
	if manager.allow_offline {
		if let Some(directory) =
			lock.get_java_path(PersistentDataJavaInstallation::Zulu, major_version)
		{
			Ok(directory)
		} else {
			update_zulu(major_version, lock, paths, client, o)
				.await
				.context("Failed to update Zulu Java")
		}
	} else {
		update_zulu(major_version, lock, paths, client, o)
			.await
			.context("Failed to update Zulu Java")
	}
}

/// Updates Adoptium and returns the path to the installation
async fn update_adoptium(
	major_version: &str,
	lock: &mut PersistentData,
	paths: &Paths,
	client: &Client,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<PathBuf> {
	let out_dir = paths.java.join("adoptium");
	files::create_dir(&out_dir)?;
	let version = net::java::adoptium::get_latest(major_version, client)
		.await
		.context("Failed to obtain Adoptium information")?;

	let release_name = version.release_name.clone();
	let mut extracted_bin_name = release_name.clone();
	extracted_bin_name.push_str("-jre");
	let extracted_bin_dir = out_dir.join(&extracted_bin_name);

	if !lock
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

	lock.finish(paths).await?;

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
	download::file(bin_url, &arc_path, client)
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
	lock: &mut PersistentData,
	paths: &Paths,
	client: &Client,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<PathBuf> {
	let out_dir = paths.java.join("zulu");
	files::create_dir(&out_dir)?;

	let package = net::java::zulu::get_latest(major_version, client)
		.await
		.context("Failed to get the latest Zulu version")?;

	let extracted_dir = out_dir.join(net::java::zulu::extract_dir_name(&package.name));

	if !lock
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

	lock.finish(paths).await?;

	let arc_path = out_dir.join(&package.name);

	o.display(
		MessageContents::StartProcess(format!(
			"Downloading Azul Zulu JRE version {}",
			package.name
		)),
		MessageLevel::Important,
	);
	download::file(&package.download_url, &arc_path, client)
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
	None
}
