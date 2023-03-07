use std::collections::HashMap;
use std::fs::{self, File};
use std::path::PathBuf;
use std::io::{Read, Write};

use serde::{Serialize, Deserialize};

use crate::data::addon::{Addon, AddonKind};
use crate::package::reg::PkgIdentifier;

use super::files::paths::Paths;

#[derive(Debug, thiserror::Error)]
pub enum LockfileError {
	#[error("Error when accessing file:\n{}", .0)]
	Io(#[from] std::io::Error),
	#[error("Failed to parse json:\n{}", .0)]
	SerdeJson(#[from] serde_json::Error)
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct LockfileAddon {
	name: String,
	files: Vec<String>,
	kind: String
}

impl LockfileAddon {
	// Converts an addon to the format used by the lockfile
	pub fn from_addon(addon: &Addon, paths: &Paths) -> Self {
		Self {
			name: addon.name.clone(),
			files: vec![
				addon.get_path(paths).to_str()
					.expect("Failed to convert addon path to a string").to_owned()
			],
			kind: addon.kind.to_string()
		}
	}

	pub fn to_addon(&self, id: PkgIdentifier) -> Addon {
		Addon {
			kind: AddonKind::from_str(&self.kind).expect("Unknown addon kind"),
			name: self.name.clone(),
			id
		}
	}
	
	pub fn remove(&self) -> Result<(), LockfileError> {
		for file in self.files.iter() {
			let path = PathBuf::from(file);
			if path.exists() {
				fs::remove_file(path)?;
			}
		}
		
		Ok(())
	}
}

#[derive(Serialize, Deserialize)]
pub struct LockfilePackage {
	version: String,
	addons: Vec<LockfileAddon>
}

#[derive(Serialize, Deserialize)]
struct LockfileProfile {
	version: String,
	paper_build: Option<u16>
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct LockfileContents {
	#[serde(default)]
	packages: HashMap<String, HashMap<String, LockfilePackage>>,
	#[serde(default)]
	profiles: HashMap<String, LockfileProfile>
}

// A file that remembers what files and packages are currently installed
pub struct Lockfile {
	contents: LockfileContents
}

impl Lockfile {
	pub fn open(paths: &Paths) -> Result<Self, LockfileError> {
		let path = Self::get_path(paths);
		let contents = if path.exists() {
			let mut file = File::open(path)?;
			let mut contents = String::new();
			file.read_to_string(&mut contents)?;
			serde_json::from_str(&contents)?
		} else {
			LockfileContents::default()
		};
		Ok(Self {
			contents
		})
	}

	pub fn get_path(paths: &Paths) -> PathBuf {
		paths.internal.join("lock.json")
	}

	// Finish using the lockfile and write to the disk
	pub fn finish(&mut self, paths: &Paths) -> Result<(), LockfileError> {
		let out = serde_json::to_string_pretty(&self.contents)?;
		let mut file = File::create(Self::get_path(paths))?;
		file.write_all(out.as_bytes())?;
		
		Ok(())
	}

	// Updates a package with a new version
	pub fn update_package(&mut self, name: &str, instance: &str, version: &str, addons: &[LockfileAddon])
	-> Result<Vec<String>, LockfileError> {
		let mut addons_to_remove = Vec::new();
		if let Some(instance) = self.contents.packages.get_mut(instance) {
			if let Some(pkg) = instance.get_mut(name) {
				pkg.version = version.to_owned();
				let mut indices = Vec::new();
				for (i, addon) in pkg.addons.iter().enumerate() {
					if !addons.contains(addon) {
						indices.push(i);
						addons_to_remove.push(addon.name.clone());
					}
				}
				for i in indices {
					let addon = pkg.addons.remove(i);
					addon.remove()?;
				}
				pkg.addons = addons.to_vec();
			} else {
				instance.insert(
					name.to_owned(),
					LockfilePackage {
						version: version.to_owned(),
						addons: addons.to_vec()
					}
				);
			}
		} else {
			self.contents.packages.insert(instance.to_owned(), HashMap::new());
			self.update_package(name, instance, version, addons)?;
		}
		
		Ok(addons_to_remove)
	}

	// Remove any unused packages for a profile. Returns any addons that need to be removed from the instance
	pub fn remove_unused_packages(&mut self, instance: &str, used_packages: &[String])
	-> Result<Vec<Addon>, LockfileError> {
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
						addon.remove()?;
						let id = PkgIdentifier { name: pkg_id.clone(), version: pkg.version.clone() };
						addons_to_remove.push(addon.to_addon(id));
					}
				}
			}

			Ok(addons_to_remove)
		} else {
			Ok(vec![])
		}
	}

	// Updates a profile in the lockfile. Returns true if the version has changed
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
					paper_build: None
				}
			);

			false
		}
	}

	// Updates a profile with a new paper build. Returns true if the version has changed
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
