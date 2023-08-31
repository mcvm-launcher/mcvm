/// Operating Java memory arguments
pub mod args;
/// Use of Java's classpath format
pub mod classpath;

use crate::data::profile::update::manager::{UpdateManager, UpdateMethodResult};
use crate::io::files::{self, paths::Paths};
use crate::net;
use crate::net::download;
use crate::util::{json, preferred_archive_extension};

use anyhow::Context;
use libflate::gzip::Decoder;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use reqwest::Client;
use tar::Archive;

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use super::lock::{Lockfile, LockfileJavaInstallation};
use mcvm_shared::later::Later;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum JavaKind {
	Adoptium(Later<String>),
	Zulu(Later<String>),
	Custom(PathBuf),
}

impl JavaKind {
	pub fn parse(string: &str) -> Self {
		match string {
			"adoptium" => Self::Adoptium(Later::Empty),
			"zulu" => Self::Zulu(Later::Empty),
			path => Self::Custom(PathBuf::from(String::from(shellexpand::tilde(path)))),
		}
	}
}

/// A Java installation used to launch the game
#[derive(Debug, Clone)]
pub struct Java {
	kind: JavaKind,
	pub path: Later<PathBuf>,
}

impl Java {
	pub fn new(kind: JavaKind) -> Self {
		Self {
			kind,
			path: Later::Empty,
		}
	}

	/// Add a major version to a Java installation that supports it
	pub fn add_version(&mut self, version: &str) {
		match &mut self.kind {
			JavaKind::Adoptium(vers) | JavaKind::Zulu(vers) => vers.fill(version.to_owned()),
			JavaKind::Custom(..) => {}
		};
	}

	/// Download / install all needed files
	pub async fn install(
		&mut self,
		paths: &Paths,
		manager: &UpdateManager,
		lock: &mut Lockfile,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<UpdateMethodResult> {
		let out = UpdateMethodResult::new();

		o.start_process();
		o.display(
			MessageContents::StartProcess("Checking for Java updates".to_string()),
			MessageLevel::Important,
		);

		match &self.kind {
			JavaKind::Adoptium(major_version) => {
				let directory = if manager.allow_offline {
					if let Some(directory) =
						lock.get_java_path(LockfileJavaInstallation::Adoptium, major_version.get())
					{
						Ok(directory)
					} else {
						update_adoptium(major_version.get(), lock, paths, o)
							.await
							.context("Failed to update Adoptium Java")
					}
				} else {
					update_adoptium(major_version.get(), lock, paths, o)
						.await
						.context("Failed to update Adoptium Java")
				}?;
				self.path.fill(directory);
			}
			JavaKind::Zulu(major_version) => {
				let directory = if manager.allow_offline {
					if let Some(directory) =
						lock.get_java_path(LockfileJavaInstallation::Zulu, major_version.get())
					{
						Ok(directory)
					} else {
						update_zulu(major_version.get(), lock, paths, o)
							.await
							.context("Failed to update Zulu Java")
					}
				} else {
					update_zulu(major_version.get(), lock, paths, o)
						.await
						.context("Failed to update Zulu Java")
				}?;
				self.path.fill(directory);
			}
			JavaKind::Custom(path) => {
				self.path.fill(path.clone());
			}
		}
		o.display(
			MessageContents::Success("Java updated".to_string()),
			MessageLevel::Important,
		);

		Ok(out)
	}
}

/// Updates Adoptium and returns the path to the installation
async fn update_adoptium(
	major_version: &str,
	lock: &mut Lockfile,
	paths: &Paths,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<PathBuf> {
	let out_dir = paths.java.join("adoptium");
	files::create_dir(&out_dir)?;
	let version = net::java::adoptium::get_latest(major_version)
		.await
		.context("Failed to obtain Adoptium information")?;

	let release_name = json::access_str(&version, "release_name")?;

	let mut extracted_bin_name = json::access_str(&version, "release_name")?.to_string();
	extracted_bin_name.push_str("-jre");
	let extracted_bin_dir = out_dir.join(&extracted_bin_name);

	if !lock
		.update_java_installation(
			LockfileJavaInstallation::Adoptium,
			major_version,
			release_name,
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

	let bin_url = json::access_str(
		json::access_object(json::access_object(&version, "binary")?, "package")?,
		"link",
	)?;

	o.display(
		MessageContents::StartProcess(format!(
			"Downloading Adoptium Temurin JRE version {release_name}"
		)),
		MessageLevel::Important,
	);
	download::file(bin_url, &arc_path, &Client::new())
		.await
		.context("Failed to download JRE binaries")?;

	// Extraction
	o.display(
		MessageContents::StartProcess("Extracting JRE".to_string()),
		MessageLevel::Important,
	);
	extract_archive(&arc_path, &out_dir).context("Failed to extract")?;
	o.display(
		MessageContents::StartProcess("Removing archive".to_string()),
		MessageLevel::Important,
	);
	tokio::fs::remove_file(arc_path)
		.await
		.context("Failed to remove archive")?;

	o.display(
		MessageContents::Success("Java installation finished".to_string()),
		MessageLevel::Important,
	);
	o.end_process();

	Ok(extracted_bin_dir)
}

/// Updates Zulu and returns the path to the installation
async fn update_zulu(
	major_version: &str,
	lock: &mut Lockfile,
	paths: &Paths,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<PathBuf> {
	let out_dir = paths.java.join("zulu");
	files::create_dir(&out_dir)?;

	let package = net::java::zulu::get_latest(major_version)
		.await
		.context("Failed to get the latest Zulu version")?;

	let extracted_dir = out_dir.join(net::java::zulu::extract_dir_name(&package.name));

	if !lock
		.update_java_installation(
			LockfileJavaInstallation::Zulu,
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
	download::file(&package.download_url, &arc_path, &Client::new())
		.await
		.context("Failed to download JRE binaries")?;

	// Extraction
	o.display(
		MessageContents::StartProcess("Extracting JRE".to_string()),
		MessageLevel::Important,
	);
	extract_archive(&arc_path, &out_dir).context("Failed to extract")?;
	o.display(
		MessageContents::StartProcess("Removing archive".to_string()),
		MessageLevel::Important,
	);
	tokio::fs::remove_file(arc_path)
		.await
		.context("Failed to remove archive")?;

	o.display(
		MessageContents::Success("Java installation finished".to_string()),
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
		let mut decoder = Decoder::new(&mut file).context("Failed to decode tar.gz")?;
		let mut arc = Archive::new(&mut decoder);
		arc.unpack(out_dir).context("Failed to unarchive tar")?;
	}

	Ok(())
}
