use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use super::files::paths::Paths;
use super::json_from_file;

/// A file that remembers important info like what versions and files are currently installed
#[derive(Debug)]
pub struct PersistentData {
	contents: PersistentDataContents,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
struct PersistentDataContents {
	java: PersistentDataJava,
	versions: HashMap<String, PersistentDataVersionInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PersistentDataVersionInfo {
	version: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct PersistentDataJavaVersion {
	version: String,
	path: String,
}

/// Contains maps of major versions to information about installations
#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
struct PersistentDataJava {
	adoptium: HashMap<String, PersistentDataJavaVersion>,
	zulu: HashMap<String, PersistentDataJavaVersion>,
}

/// Used as a function argument
pub(crate) enum PersistentDataJavaInstallation {
	/// Adoptium Java
	Adoptium,
	/// Zulu Java
	Zulu,
}

impl PersistentDataContents {
	/// Fix changes in persistent data format
	pub fn fix(&mut self) {}
}

impl PersistentData {
	/// Open the persistent data file
	pub fn open(paths: &Paths) -> anyhow::Result<Self> {
		let path = Self::get_path(paths);
		let mut contents = if path.exists() {
			json_from_file(&path).context("Failed to get JSON contents")?
		} else {
			PersistentDataContents::default()
		};
		contents.fix();
		Ok(Self { contents })
	}

	/// Get the path to the persistent data file
	pub fn get_path(paths: &Paths) -> PathBuf {
		paths.internal.join("core_persistent.json")
	}

	/// Finish using the persistent data file and write to the disk
	pub async fn dump(&mut self, paths: &Paths) -> anyhow::Result<()> {
		let path = Self::get_path(paths);
		let file =
			BufWriter::new(File::create(path).context("Failed to open persistent data file")?);
		serde_json::to_writer_pretty(file, &self.contents)
			.context("Failed to serialize persistent data contents")?;

		Ok(())
	}

	/// Updates a Java installation with a new version. Returns true if the version has changed.
	pub(crate) fn update_java_installation(
		&mut self,
		installation: PersistentDataJavaInstallation,
		major_version: &str,
		version: &str,
		path: &Path,
	) -> anyhow::Result<bool> {
		let installation = match installation {
			PersistentDataJavaInstallation::Adoptium => &mut self.contents.java.adoptium,
			PersistentDataJavaInstallation::Zulu => &mut self.contents.java.zulu,
		};
		let path_str = path.to_string_lossy().to_string();
		if let Some(current_version) = installation.get_mut(major_version) {
			if current_version.version == version {
				Ok(false)
			} else {
				// Remove the old installation, if it exists
				let current_version_path = PathBuf::from(&current_version.path);
				if current_version_path.exists() {
					fs::remove_dir_all(current_version_path)
						.context("Failed to remove old Java installation")?;
				}
				current_version.version = version.to_string();
				current_version.path = path_str;
				Ok(true)
			}
		} else {
			installation.insert(
				major_version.to_string(),
				PersistentDataJavaVersion {
					version: version.to_string(),
					path: path_str,
				},
			);
			Ok(true)
		}
	}

	/// Gets the path to a Java installation
	pub(crate) fn get_java_path(
		&self,
		installation: PersistentDataJavaInstallation,
		version: &str,
	) -> Option<PathBuf> {
		let installation = match installation {
			PersistentDataJavaInstallation::Adoptium => &self.contents.java.adoptium,
			PersistentDataJavaInstallation::Zulu => &self.contents.java.zulu,
		};
		let version = installation.get(version)?;
		Some(PathBuf::from(version.path.clone()))
	}
}
