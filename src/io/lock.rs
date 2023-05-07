use std::collections::HashMap;
use std::fs::{self, File};
use std::path::PathBuf;

use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};

use mcvm_shared::addon::{Addon, AddonKind};
use mcvm_shared::pkg::PkgIdentifier;

use crate::data::addon::get_addon_path;

use super::files::paths::Paths;

/// Format for an addon in the lockfile
#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct LockfileAddon {
	#[serde(alias = "name")]
	id: String,
	file_name: Option<String>,
	files: Vec<String>,
	kind: String,
}

impl LockfileAddon {
	/// Converts an addon to the format used by the lockfile
	pub fn from_addon(addon: &Addon, paths: &Paths) -> Self {
		Self {
			id: addon.id.clone(),
			file_name: Some(addon.file_name.clone()),
			files: vec![get_addon_path(addon, paths)
				.to_str()
				.expect("Failed to convert addon path to a string")
				.to_owned()],
			kind: addon.kind.to_string(),
		}
	}

	/// Converts this LockfileAddon to an Addon
	pub fn to_addon(&self, pkg_id: PkgIdentifier) -> anyhow::Result<Addon> {
		Ok(Addon {
			kind: AddonKind::from_str(&self.kind)
				.ok_or(anyhow!("Invalid addon kind '{}'", self.kind))?,
			id: self.id.clone(),
			file_name: self
				.file_name
				.clone()
				.expect("Filename should have been filled in or fixed"),
			pkg_id,
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

#[derive(Serialize, Deserialize)]
pub struct LockfilePackage {
	version: u32,
	addons: Vec<LockfileAddon>,
}

#[derive(Serialize, Deserialize)]
struct LockfileProfile {
	version: String,
	paper_build: Option<u16>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct LockfileContents {
	#[serde(default)]
	packages: HashMap<String, HashMap<String, LockfilePackage>>,
	#[serde(default)]
	profiles: HashMap<String, LockfileProfile>,
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
pub struct Lockfile {
	contents: LockfileContents,
}

impl Lockfile {
	/// Open the lockfile
	pub fn open(paths: &Paths) -> anyhow::Result<Self> {
		let path = Self::get_path(paths);
		let mut contents = if path.exists() {
			let mut file = File::open(&path).context("Failed to open lockfile")?;
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

	/// Updates a package with a new version
	pub fn update_package(
		&mut self,
		name: &str,
		instance: &str,
		version: u32,
		addons: &[LockfileAddon],
	) -> anyhow::Result<Vec<String>> {
		let mut addons_to_remove = Vec::new();
		if let Some(instance) = self.contents.packages.get_mut(instance) {
			if let Some(pkg) = instance.get_mut(name) {
				pkg.version = version.to_owned();
				let mut indices = Vec::new();
				for (i, addon) in pkg.addons.iter().enumerate() {
					if !addons.contains(addon) {
						indices.push(i);
						addons_to_remove.push(addon.id.clone());
					}
				}
				for i in indices {
					pkg.addons.remove(i);
				}
				pkg.addons = addons.to_vec();
			} else {
				instance.insert(
					name.to_owned(),
					LockfilePackage {
						version: version.to_owned(),
						addons: addons.to_vec(),
					},
				);
			}
		} else {
			self.contents
				.packages
				.insert(instance.to_owned(), HashMap::new());
			self.update_package(name, instance, version, addons)?;
		}

		Ok(addons_to_remove)
	}

	/// Remove any unused packages for an instance.
	/// Returns any addons that need to be removed from the instance.
	pub fn remove_unused_packages(
		&mut self,
		instance: &str,
		used_packages: &[String],
	) -> anyhow::Result<Vec<Addon>> {
		if let Some(inst) = self.contents.packages.get_mut(instance) {
			let mut pkgs_to_remove = Vec::new();
			for (pkg, ..) in inst.iter() {
				if !used_packages.contains(pkg) {
					pkgs_to_remove.push(pkg.clone());
				}
			}

			let mut addons_to_remove = Vec::new();
			for pkg_id in pkgs_to_remove {
				if let Some(pkg) = inst.remove(&pkg_id) {
					for addon in pkg.addons {
						let id = PkgIdentifier {
							name: pkg_id.clone(),
							version: pkg.version,
						};
						addons_to_remove.push(addon.to_addon(id)?);
					}
				}
			}

			Ok(addons_to_remove)
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
}
