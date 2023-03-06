use std::collections::HashMap;
use std::fs;
use std::{fs::File, path::PathBuf};
use std::io::{Read, Write};

use serde::{Serialize, Deserialize};

use crate::data::asset::Asset;

use super::files::paths::Paths;

#[derive(Debug, thiserror::Error)]
pub enum LockfileError {
	#[error("Error when accessing file:\n{}", .0)]
	Io(#[from] std::io::Error),
	#[error("Failed to parse json:\n{}", .0)]
	SerdeJson(#[from] serde_json::Error)
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct LockfileAsset {
	files: Vec<String>,
	kind: String
}

impl LockfileAsset {
	// Converts an asset to the format used by the lockfile
	pub fn from_asset(asset: &Asset, paths: &Paths) -> Self {
		Self {
			files: vec![
				asset.get_path(paths).to_str()
					.expect("Failed to convert asset path to a string").to_owned()
			],
			kind: asset.kind.to_string()
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
	assets: Vec<LockfileAsset>
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct LockfileContents {
	packages: HashMap<String, HashMap<String, LockfilePackage>>
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
	pub fn update_package(&mut self, name: &str, profile: &str, version: &str, assets: &[LockfileAsset])
	-> Result<(), LockfileError> {
		if let Some(profile) = self.contents.packages.get_mut(profile) {
			if let Some(pkg) = profile.get_mut(name) {
				pkg.version = version.to_owned();
				let mut indices = Vec::new();
				for (i, asset) in pkg.assets.iter().enumerate() {
					if !assets.contains(asset) {
						indices.push(i);
					}
				}
				for i in indices {
					let asset = pkg.assets.remove(i);
					asset.remove()?;
				}
				pkg.assets = assets.to_vec();
			} else {
				profile.insert(
					name.to_owned(),
					LockfilePackage {
						version: version.to_owned(),
						assets: assets.to_vec()
					}
				);
			}
		} else {
			self.contents.packages.insert(profile.to_owned(), HashMap::new());
			self.update_package(name, profile, version, assets)?;
		}
		
		Ok(())
	}

	// Removes a package, its assets, and all associated files
	pub fn remove_package(&mut self, name: &str, profile: &str) -> Result<(), LockfileError> {
		if let Some(profile) = self.contents.packages.get_mut(profile) {
			if let Some(pkg) = profile.remove(name) {
				for asset in pkg.assets {
					asset.remove()?;	
				}
			}
		}

		Ok(())
	}

	// Remove any unused packages for a profile.
	pub fn remove_unused_packages(&mut self, profile: &str, used_packages: &[String])
	-> Result<(), LockfileError> {
		// let mut assets_to_remove = Vec::new();
		if let Some(prof) = self.contents.packages.get_mut(profile) {
			for pkg in used_packages {
				if prof.contains_key(pkg) {
					if let Some(pkg) = prof.remove(pkg) {
						for asset in pkg.assets {
							asset.remove()?;
						}
					}
				}
			}
		}
		
		Ok(())
	}
}
