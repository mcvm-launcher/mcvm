use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context};
use mcvm_shared::output::{MCVMOutput, MessageContents};
use serde::{Deserialize, Serialize};

use mcvm_shared::addon::{Addon, AddonKind};
use mcvm_shared::pkg::{PackageAddonOptionalHashes, PkgIdentifier};

use super::files::paths::Paths;

/// Format for an addon in the lockfile
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct LockfileAddon {
	#[serde(alias = "name")]
	id: String,
	file_name: Option<String>,
	files: Vec<String>,
	kind: String,
	version: Option<String>,
	#[serde(default)]
	hashes: PackageAddonOptionalHashes,
}

impl LockfileAddon {
	/// Converts an addon to the format used by the lockfile.
	/// Paths is the list of paths for the addon in the instance
	pub fn from_addon(addon: &Addon, paths: Vec<PathBuf>) -> Self {
		Self {
			id: addon.id.clone(),
			file_name: Some(addon.file_name.clone()),
			files: paths
				.iter()
				.map(|x| {
					x.to_str()
						.expect("Failed to convert addon path to a string")
						.to_owned()
				})
				.collect(),
			kind: addon.kind.to_string(),
			version: addon.version.clone(),
			hashes: addon.hashes.clone(),
		}
	}

	/// Converts this LockfileAddon to an Addon
	pub fn to_addon(&self, pkg_id: PkgIdentifier) -> anyhow::Result<Addon> {
		Ok(Addon {
			kind: AddonKind::parse_from_str(&self.kind)
				.ok_or(anyhow!("Invalid addon kind '{}'", self.kind))?,
			id: self.id.clone(),
			file_name: self
				.file_name
				.clone()
				.expect("Filename should have been filled in or fixed"),
			pkg_id,
			version: self.version.clone(),
			hashes: self.hashes.clone(),
		})
	}

	pub fn _remove(&self) -> anyhow::Result<()> {
		for file in self.files.iter() {
			let path = PathBuf::from(file);
			if path.exists() {
				fs::remove_file(path).context("Failed to remove addon")?;
			}
		}

		Ok(())
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LockfilePackage {
	version: u32,
	addons: Vec<LockfileAddon>,
}

#[derive(Serialize, Deserialize, Debug)]
struct LockfileProfile {
	version: String,
	paper_build: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug)]
struct LockfileJavaVersion {
	version: String,
	path: String,
}

/// Contains maps of major versions to information about installations
#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
struct LockfileJava {
	adoptium: HashMap<String, LockfileJavaVersion>,
	zulu: HashMap<String, LockfileJavaVersion>,
}

/// Used as a function argument
pub enum LockfileJavaInstallation {
	Adoptium,
	Zulu,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
struct LockfileContents {
	packages: HashMap<String, HashMap<String, LockfilePackage>>,
	profiles: HashMap<String, LockfileProfile>,
	java: LockfileJava,
}

impl LockfileContents {
	/// Fix changes in lockfile format
	pub fn fix(&mut self) {
		for (.., instance) in &mut self.packages {
			for (.., package) in instance {
				for addon in &mut package.addons {
					if addon.file_name.is_none() {
						addon.file_name = Some(addon.id.clone())
					}
				}
			}
		}
	}
}

/// A file that remembers important info like what files and packages are currently installed
#[derive(Debug)]
pub struct Lockfile {
	contents: LockfileContents,
}

impl Lockfile {
	/// Open the lockfile
	pub fn open(paths: &Paths) -> anyhow::Result<Self> {
		let path = Self::get_path(paths);
		let mut contents = if path.exists() {
			let file = File::open(&path).context("Failed to open lockfile")?;
			let mut file = BufReader::new(file);
			serde_json::from_reader(&mut file).context("Failed to parse JSON")?
		} else {
			LockfileContents::default()
		};
		contents.fix();
		Ok(Self { contents })
	}

	/// Get the path to the lockfile
	pub fn get_path(paths: &Paths) -> PathBuf {
		paths.internal.join("lock.json")
	}

	/// Finish using the lockfile and write to the disk
	pub async fn finish(&mut self, paths: &Paths) -> anyhow::Result<()> {
		let out = serde_json::to_string_pretty(&self.contents)
			.context("Failed to serialize lockfile contents")?;
		tokio::fs::write(Self::get_path(paths), out)
			.await
			.context("Failed to write to lockfile")?;

		Ok(())
	}

	/// Updates a package with a new version.
	/// Returns a list of addon files to be removed
	pub fn update_package(
		&mut self,
		id: &str,
		instance: &str,
		version: u32,
		addons: &[LockfileAddon],
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Vec<PathBuf>> {
		let mut files_to_remove = Vec::new();
		let mut new_files = Vec::new();
		if let Some(instance) = self.contents.packages.get_mut(instance) {
			if let Some(pkg) = instance.get_mut(id) {
				pkg.version = version.to_owned();
				let mut indices = Vec::new();
				// Check for addons that need to be removed
				for (i, current) in pkg.addons.iter().enumerate() {
					if !addons.iter().any(|x| x.id == current.id) {
						indices.push(i);
						files_to_remove.extend(current.files.iter().map(PathBuf::from));
					}
				}
				for i in indices {
					pkg.addons.remove(i);
				}
				// Check for addons that need to be updated
				for requested in addons {
					if let Some(current) = pkg.addons.iter().find(|x| x.id == requested.id) {
						files_to_remove.extend(
							current
								.files
								.iter()
								.filter(|x| !requested.files.contains(x))
								.map(PathBuf::from),
						);
						new_files.extend(
							requested
								.files
								.iter()
								.filter(|x| !current.files.contains(x))
								.cloned(),
						);
					} else {
						new_files.extend(requested.files.clone());
					};
				}

				pkg.addons = addons.to_vec();
			} else {
				instance.insert(
					id.to_owned(),
					LockfilePackage {
						version: version.to_owned(),
						addons: addons.to_vec(),
					},
				);
				new_files.extend(addons.iter().flat_map(|x| x.files.clone()));
			}
		} else {
			self.contents
				.packages
				.insert(instance.to_owned(), HashMap::new());
			self.update_package(id, instance, version, addons, o)?;
		}

		for file in &new_files {
			if PathBuf::from(file).exists() {
				let allow = o.prompt_yes_no(false, MessageContents::Warning(
					format!("The existing file '{file}' has the same path as an addon. Overwrite it?")
				))
				.context("Prompt failed")?;
			
				if !allow {
					bail!("File '{file}' would be overwritten by an addon");
				}
			}
		}

		Ok(files_to_remove)
	}

	/// Remove any unused packages for an instance.
	/// Returns any addon files that need to be removed from the instance.
	pub fn remove_unused_packages(
		&mut self,
		instance: &str,
		used_packages: &[String],
	) -> anyhow::Result<Vec<PathBuf>> {
		if let Some(inst) = self.contents.packages.get_mut(instance) {
			let mut pkgs_to_remove = Vec::new();
			for (pkg, ..) in inst.iter() {
				if !used_packages.contains(pkg) {
					pkgs_to_remove.push(pkg.clone());
				}
			}

			let mut files_to_remove = Vec::new();
			for pkg_id in pkgs_to_remove {
				if let Some(pkg) = inst.remove(&pkg_id) {
					for addon in pkg.addons {
						files_to_remove.extend(addon.files.iter().map(PathBuf::from));
					}
				}
			}

			Ok(files_to_remove)
		} else {
			Ok(vec![])
		}
	}

	/// Updates a profile in the lockfile. Returns true if the version has changed.
	pub fn update_profile_version(&mut self, profile: &str, version: &str) -> bool {
		if let Some(profile) = self.contents.profiles.get_mut(profile) {
			if profile.version == version {
				false
			} else {
				profile.version = version.to_owned();
				true
			}
		} else {
			self.contents.profiles.insert(
				profile.to_owned(),
				LockfileProfile {
					version: version.to_owned(),
					paper_build: None,
				},
			);

			false
		}
	}

	/// Updates a profile with a new paper build. Returns true if the version has changed.
	pub fn update_profile_paper_build(&mut self, profile: &str, build_num: u16) -> bool {
		if let Some(profile) = self.contents.profiles.get_mut(profile) {
			if let Some(paper_build) = profile.paper_build.as_mut() {
				if *paper_build == build_num {
					false
				} else {
					*paper_build = build_num;
					true
				}
			} else {
				profile.paper_build = Some(build_num);
				false
			}
		} else {
			false
		}
	}

	/// Updates a Java installation with a new version. Returns true if the version has changed.
	pub fn update_java_installation(
		&mut self,
		installation: LockfileJavaInstallation,
		major_version: &str,
		version: &str,
		path: &Path,
	) -> anyhow::Result<bool> {
		let installation = match installation {
			LockfileJavaInstallation::Adoptium => &mut self.contents.java.adoptium,
			LockfileJavaInstallation::Zulu => &mut self.contents.java.zulu,
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
				LockfileJavaVersion {
					version: version.to_string(),
					path: path_str,
				},
			);
			Ok(true)
		}
	}

	/// Gets the path to a Java installation
	pub fn get_java_path(
		&self,
		installation: LockfileJavaInstallation,
		version: &str,
	) -> Option<PathBuf> {
		let installation = match installation {
			LockfileJavaInstallation::Adoptium => &self.contents.java.adoptium,
			LockfileJavaInstallation::Zulu => &self.contents.java.zulu,
		};
		let version = installation.get(version)?;
		Some(PathBuf::from(version.path.clone()))
	}
}
