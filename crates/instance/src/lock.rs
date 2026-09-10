use std::{
	collections::{HashMap, HashSet},
	fs::File,
	io::BufReader,
	path::{Path, PathBuf},
	sync::Arc,
};

use anyhow::Context;
use nitro_shared::{
	loaders::Loader,
	minecraft::AddonKind,
	pkg::{AddonOptionalHashes, ArcPkgReq, PkgRequest, PkgRequestSource},
};
use serde::{Deserialize, Serialize};

use crate::addon::Addon;

/// Stored install info about an instance
#[derive(Debug)]
pub struct InstanceLockfile {
	contents: InstanceLockfileContents,
	path: PathBuf,
}

impl InstanceLockfile {
	/// Open the lockfile at the specified path
	pub fn open(path: &Path) -> anyhow::Result<Self> {
		let contents: InstanceLockfileContents = if path.exists() {
			serde_json::from_reader(BufReader::new(File::open(path)?))
				.context("Failed to read instance lockfile")?
		} else {
			InstanceLockfileContents::default()
		};

		Ok(Self {
			contents,
			path: path.to_owned(),
		})
	}

	/// Get the path to the lockfile
	pub fn get_path(inst_dir: Option<&Path>, instance_id: &str, internal_dir: &Path) -> PathBuf {
		if let Some(inst_dir) = inst_dir {
			inst_dir.join("nitro_lock.json")
		} else {
			internal_dir
				.join("lock/instances")
				.join(format!("{instance_id}.json"))
		}
	}

	/// Finish using the lockfile and write to the disk
	pub fn write(&self) -> anyhow::Result<()> {
		if let Some(parent) = self.path.parent() {
			let _ = std::fs::create_dir_all(parent);
		}
		serde_json::to_writer(File::create(&self.path)?, &self.contents)
			.context("Failed to write to lockfile")?;

		Ok(())
	}

	/// Updates a package with a new version.
	/// Returns a list of files to be removed
	pub fn update_package(
		&mut self,
		req: &PkgRequest,
		addons: &[LockfileAddon],
		content_version: Option<String>,
	) -> Vec<PathBuf> {
		let mut files_to_remove = Vec::new();
		let req = req.to_string_no_version_or_slug();

		let existing_package_addons: Vec<LockfileAddon> = self
			.contents
			.addons
			.iter()
			.filter(|addon| addon.is_from_package(&req))
			.cloned()
			.collect();

		// Update the package
		if let Some(pkg) = self.contents.packages.get_mut(&req) {
			pkg.content_version = content_version;
		} else {
			self.contents
				.packages
				.insert(req.clone(), LockfilePackage { content_version });
		}

		// Remove all addons for the package currently in the list, and remove files that aren't in the package anymore
		self.contents.addons.retain(|addon| {
			if !addon.is_from_package(&req) {
				return true;
			}

			if !addons.iter().any(|x| x.id == addon.id) {
				files_to_remove.extend(addon.to_addon().target_paths.clone());
			}

			false
		});

		// Update addon files
		for requested in addons {
			let Some(addon_id) = &requested.id else {
				continue;
			};

			if let Some(current) = existing_package_addons
				.iter()
				.find(|x| x.is_package_addon(&req, addon_id))
			{
				files_to_remove.extend(
					current
						.files
						.iter()
						.filter(|x| !requested.files.contains(x))
						.map(PathBuf::from),
				);
			}
		}

		// Add new addons
		for requested in addons {
			self.contents.addons.push(requested.clone());
		}

		files_to_remove
	}

	/// Remove any unused packages for an instance.
	/// Returns any addons that need to be removed from the instance.
	pub fn remove_unused_packages(
		&mut self,
		used_packages: &[ArcPkgReq],
	) -> anyhow::Result<Vec<Addon>> {
		let mut pkgs_to_remove = Vec::new();
		for req in self.contents.packages.keys() {
			let req2 = Arc::new(PkgRequest::parse(req, PkgRequestSource::UserRequire));
			if used_packages.contains(&req2) {
				continue;
			}

			pkgs_to_remove.push(req.clone());
		}

		let mut addons_to_remove = Vec::new();
		for pkg_id in pkgs_to_remove {
			self.contents.packages.remove(&pkg_id);
			for addon in &self.contents.addons {
				if addon.is_from_package(&pkg_id) {
					addons_to_remove.push(addon.to_addon());
				}
			}
		}

		Ok(addons_to_remove)
	}

	/// Gets the current Minecraft version
	pub fn get_minecraft_version(&self) -> Option<&String> {
		self.contents.minecraft_version.as_ref()
	}

	/// Gets the current loader
	pub fn get_loader(&self) -> &Loader {
		&self.contents.loader
	}

	/// Gets the current loader version
	pub fn get_loader_version(&self) -> Option<&String> {
		self.contents.loader_version.as_ref()
	}

	/// Updates the Minecraft version
	pub fn update_minecraft_version(&mut self, version: &str) {
		self.contents.minecraft_version = Some(version.to_string());
	}

	/// Updates the loader
	pub fn update_loader(&mut self, loader: Loader) {
		self.contents.loader = loader;
	}

	/// Updates the loader version
	pub fn update_loader_version(&mut self, version: Option<String>) {
		self.contents.loader_version = version;
	}

	/// Updates the configured packages for the instance. Should be called after packages are installed.
	pub fn update_configured_packages(&mut self, packages: HashSet<String>) {
		self.contents.configured_packages = packages;
	}

	/// Get the locked addons
	pub fn get_addons(&self) -> impl Iterator<Item = &LockfileAddon> {
		self.contents.addons.iter()
	}

	/// Get the locked packages
	pub fn get_packages(&self) -> &HashMap<String, LockfilePackage> {
		&self.contents.packages
	}

	/// Gets the last configured packages when they were installed, for checking for changes in the package configuration
	pub fn get_configured_packages(&self) -> &HashSet<String> {
		&self.contents.configured_packages
	}

	/// Get the locked modpack
	pub fn get_modpack(&self) -> Option<&LockfileModpack> {
		self.contents.modpack.as_ref()
	}

	/// Updates the locked modpack and it's addons. Returns a list of files to remove
	pub fn update_modpack(
		&mut self,
		modpack: LockfileModpack,
		addons: &[LockfileAddon],
	) -> Vec<String> {
		let mut files_to_remove = Vec::new();
		self.contents.modpack = Some(modpack);

		self.contents.addons.retain(|x| {
			if x.from_modpack {
				// Remove files in the old but not in the new
				for file in &x.files {
					if !addons.iter().any(|x| x.files.iter().any(|x| x == file)) {
						files_to_remove.push(file.clone());
					}
				}

				false
			} else {
				true
			}
		});

		self.contents.addons.extend(addons.iter().cloned());

		files_to_remove
	}
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub(crate) struct InstanceLockfileContents {
	/// The currently installed Minecraft version of the instance
	pub minecraft_version: Option<String>,
	/// The currently installed loader of the instance
	pub loader: Loader,
	/// The currently installed loader version of the instance
	pub loader_version: Option<String>,
	/// Currently installed packages for the instance
	pub packages: HashMap<String, LockfilePackage>,
	/// Currently configured packages for the instance
	#[serde(default)]
	pub configured_packages: HashSet<String>,
	/// Currently installed addons on the instance
	#[serde(default)]
	pub addons: Vec<LockfileAddon>,
	/// Currently installed modpack on the instance
	#[serde(default)]
	pub modpack: Option<LockfileModpack>,
}

/// Package stored in the instance lockfile
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LockfilePackage {
	/// The selected content version of this package
	pub content_version: Option<String>,
}

/// Addon stored in the instance lockfile
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct LockfileAddon {
	/// ID of the addon
	#[serde(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub id: Option<String>,
	/// Source package for this addon
	#[serde(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub package: Option<String>,
	/// Whether this addon was from a modpack
	#[serde(default)]
	pub from_modpack: bool,
	/// Filename of the addon
	pub file_name: String,
	/// Files for the addon
	pub files: Vec<String>,
	/// The kind of the addon
	pub kind: AddonKind,
	/// Hashes for the addon
	#[serde(default)]
	#[serde(skip_serializing_if = "AddonOptionalHashes::is_empty")]
	pub hashes: AddonOptionalHashes,
}

impl LockfileAddon {
	/// Checks if this is addon is from a specific package
	pub fn is_from_package(&self, req: &str) -> bool {
		self.package.as_ref().is_some_and(|x| x == req)
	}

	/// Checks if this is a specific addon from a specific package
	pub fn is_package_addon(&self, req: &str, addon_id: &str) -> bool {
		self.is_from_package(req) && self.id.as_ref().is_some_and(|x| x == addon_id)
	}

	/// Converts this lockfile addon to an addon
	pub fn to_addon(&self) -> Addon {
		Addon {
			kind: self.kind,
			file_name: self.file_name.clone(),
			original_path: None,
			target_paths: self.files.iter().map(PathBuf::from).collect(),
			source: None,
			hashes: self.hashes.clone(),
		}
	}
}

/// Information about a modpack in the lockfile
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LockfileModpack {
	/// Display name of the modpack
	pub name: String,
	/// Stored path of the modpack
	pub path: String,
	/// Suppressed packages of the modpack
	pub packages: Vec<String>,
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::PathBuf;

	fn create_test_lockfile() -> InstanceLockfile {
		InstanceLockfile {
			contents: InstanceLockfileContents {
				minecraft_version: Some("1.20.1".to_string()),
				loader: Loader::Fabric,
				loader_version: Some("0.14.0".to_string()),
				packages: HashMap::new(),
				configured_packages: HashSet::new(),
				addons: Vec::new(),
				modpack: None,
			},
			path: PathBuf::from("/tmp/test_lock.json"),
		}
	}

	#[test]
	fn test_update_package_new_package() {
		let mut lockfile = create_test_lockfile();
		let req = PkgRequest::parse("test-pkg", PkgRequestSource::UserRequire);

		let addon = LockfileAddon {
			id: Some("addon1".to_string()),
			package: Some("test-pkg".to_string()),
			from_modpack: false,
			file_name: "addon1.jar".to_string(),
			files: vec!["mods/addon1.jar".to_string()],
			kind: AddonKind::Mod,
			hashes: AddonOptionalHashes::default(),
		};

		let files_to_remove =
			lockfile.update_package(&req, &[addon.clone()], Some("1.0.0".to_string()));

		assert_eq!(files_to_remove.len(), 0);
		assert_eq!(lockfile.contents.packages.len(), 1);
		assert_eq!(lockfile.contents.addons.len(), 1);
		assert_eq!(
			lockfile.contents.packages["test-pkg"].content_version,
			Some("1.0.0".to_string())
		);
	}

	#[test]
	fn test_update_package_version_change() {
		let mut lockfile = create_test_lockfile();
		let req = PkgRequest::parse("test-pkg", PkgRequestSource::UserRequire);

		// Initial version
		lockfile.update_package(&req, &[], Some("1.0.0".to_string()));

		// Update to new version
		let addon = LockfileAddon {
			id: Some("addon1".to_string()),
			package: Some("test-pkg".to_string()),
			from_modpack: false,
			file_name: "addon1.jar".to_string(),
			files: vec!["mods/addon1.jar".to_string()],
			kind: AddonKind::Mod,
			hashes: AddonOptionalHashes::default(),
		};

		lockfile.update_package(&req, &[addon], Some("2.0.0".to_string()));

		assert_eq!(
			lockfile.contents.packages["test-pkg"].content_version,
			Some("2.0.0".to_string())
		);
	}

	#[test]
	fn test_update_package_addon_file_changed() {
		let mut lockfile = create_test_lockfile();
		let req = PkgRequest::parse("test-pkg", PkgRequestSource::UserRequire);

		// Initial addon with old file
		let old_addon = LockfileAddon {
			id: Some("addon1".to_string()),
			package: Some("test-pkg".to_string()),
			from_modpack: false,
			file_name: "addon1_old.jar".to_string(),
			files: vec!["mods/addon1_old.jar".to_string()],
			kind: AddonKind::Mod,
			hashes: AddonOptionalHashes::default(),
		};

		lockfile.update_package(&req, &[old_addon], None);

		// Update with new file
		let new_addon = LockfileAddon {
			id: Some("addon1".to_string()),
			package: Some("test-pkg".to_string()),
			from_modpack: false,
			file_name: "addon1_new.jar".to_string(),
			files: vec!["mods/addon1_new.jar".to_string()],
			kind: AddonKind::Mod,
			hashes: AddonOptionalHashes::default(),
		};

		let files_to_remove = lockfile.update_package(&req, &[new_addon], None);

		assert_eq!(files_to_remove.len(), 1);
		assert_eq!(files_to_remove[0], PathBuf::from("mods/addon1_old.jar"));
		assert_eq!(lockfile.contents.addons.len(), 1);
		assert_eq!(lockfile.contents.addons[0].file_name, "addon1_new.jar");
	}

	#[test]
	fn test_update_package_addon_removed() {
		let mut lockfile = create_test_lockfile();
		let req = PkgRequest::parse("test-pkg", PkgRequestSource::UserRequire);

		// Initial addons
		let addon1 = LockfileAddon {
			id: Some("addon1".to_string()),
			package: Some("test-pkg".to_string()),
			from_modpack: false,
			file_name: "addon1.jar".to_string(),
			files: vec!["mods/addon1.jar".to_string()],
			kind: AddonKind::Mod,
			hashes: AddonOptionalHashes::default(),
		};

		let addon2 = LockfileAddon {
			id: Some("addon2".to_string()),
			package: Some("test-pkg".to_string()),
			from_modpack: false,
			file_name: "addon2.jar".to_string(),
			files: vec!["mods/addon2.jar".to_string()],
			kind: AddonKind::Mod,
			hashes: AddonOptionalHashes::default(),
		};

		lockfile.update_package(&req, &[addon1.clone(), addon2], None);

		// Update to only keep addon1
		let files_to_remove = lockfile.update_package(&req, &[addon1], None);

		assert_eq!(files_to_remove.len(), 1);
		assert_eq!(files_to_remove[0], PathBuf::from("mods/addon2.jar"));
		assert_eq!(lockfile.contents.addons.len(), 1);
	}

	#[test]
	fn test_update_package_multiple_files_per_addon() {
		let mut lockfile = create_test_lockfile();
		let req = PkgRequest::parse("test-pkg", PkgRequestSource::UserRequire);

		// Addon with multiple files
		let addon_old = LockfileAddon {
			id: Some("addon1".to_string()),
			package: Some("test-pkg".to_string()),
			from_modpack: false,
			file_name: "addon1.jar".to_string(),
			files: vec![
				"mods/addon1.jar".to_string(),
				"config/addon1.toml".to_string(),
			],
			kind: AddonKind::Mod,
			hashes: AddonOptionalHashes::default(),
		};

		lockfile.update_package(&req, &[addon_old], None);

		// Update: remove config file
		let addon_new = LockfileAddon {
			id: Some("addon1".to_string()),
			package: Some("test-pkg".to_string()),
			from_modpack: false,
			file_name: "addon1.jar".to_string(),
			files: vec!["mods/addon1.jar".to_string()],
			kind: AddonKind::Mod,
			hashes: AddonOptionalHashes::default(),
		};

		let files_to_remove = lockfile.update_package(&req, &[addon_new], None);

		assert_eq!(files_to_remove.len(), 1);
		assert!(
			files_to_remove
				.iter()
				.any(|x| x == &PathBuf::from("config/addon1.toml"))
		);
	}

	#[test]
	fn test_remove_unused_packages() {
		let mut lockfile = create_test_lockfile();
		let req1 = PkgRequest::parse("pkg1", PkgRequestSource::UserRequire);
		let req2 = PkgRequest::parse("pkg2", PkgRequestSource::UserRequire);

		// Add two packages with addons
		let addon1 = LockfileAddon {
			id: Some("addon1".to_string()),
			package: Some("pkg1".to_string()),
			from_modpack: false,
			file_name: "addon1.jar".to_string(),
			files: vec!["mods/addon1.jar".to_string()],
			kind: AddonKind::Mod,
			hashes: AddonOptionalHashes::default(),
		};

		let addon2 = LockfileAddon {
			id: Some("addon2".to_string()),
			package: Some("pkg2".to_string()),
			from_modpack: false,
			file_name: "addon2.jar".to_string(),
			files: vec!["mods/addon2.jar".to_string()],
			kind: AddonKind::Mod,
			hashes: AddonOptionalHashes::default(),
		};

		lockfile.update_package(&req1, &[addon1], None);
		lockfile.update_package(&req2, &[addon2], None);

		// Only keep pkg1
		let used = vec![Arc::new(req1)];
		let removed_addons = lockfile.remove_unused_packages(&used).unwrap();

		assert_eq!(lockfile.contents.packages.len(), 1);
		assert!(lockfile.contents.packages.contains_key("pkg1"));
		assert_eq!(removed_addons.len(), 1);
		assert_eq!(removed_addons[0].file_name, "addon2.jar");
	}

	#[test]
	fn test_update_modpack_new() {
		let mut lockfile = create_test_lockfile();

		let modpack = LockfileModpack {
			name: "Test Modpack".to_string(),
			path: "/path/to/modpack".to_string(),
			packages: vec!["pkg1".to_string()],
		};

		let addon = LockfileAddon {
			id: None,
			package: None,
			from_modpack: true,
			file_name: "mod1.jar".to_string(),
			files: vec!["mods/mod1.jar".to_string()],
			kind: AddonKind::Mod,
			hashes: AddonOptionalHashes::default(),
		};

		let files_to_remove = lockfile.update_modpack(modpack.clone(), &[addon.clone()]);

		assert_eq!(files_to_remove.len(), 0);
		assert_eq!(
			lockfile.contents.modpack.as_ref().unwrap().name,
			"Test Modpack"
		);
		assert_eq!(lockfile.contents.addons.len(), 1);
		assert!(lockfile.contents.addons[0].from_modpack);
	}

	#[test]
	fn test_update_modpack_addon_removed() {
		let mut lockfile = create_test_lockfile();

		let modpack1 = LockfileModpack {
			name: "Modpack v1".to_string(),
			path: "/path/to/modpack".to_string(),
			packages: vec![],
		};

		let addon1 = LockfileAddon {
			id: None,
			package: None,
			from_modpack: true,
			file_name: "mod1.jar".to_string(),
			files: vec!["mods/mod1.jar".to_string()],
			kind: AddonKind::Mod,
			hashes: AddonOptionalHashes::default(),
		};

		let addon2 = LockfileAddon {
			id: None,
			package: None,
			from_modpack: true,
			file_name: "mod2.jar".to_string(),
			files: vec!["mods/mod2.jar".to_string()],
			kind: AddonKind::Mod,
			hashes: AddonOptionalHashes::default(),
		};

		lockfile.update_modpack(modpack1, &[addon1.clone(), addon2]);

		// Update: keep only addon1
		let modpack2 = LockfileModpack {
			name: "Modpack v2".to_string(),
			path: "/path/to/modpack".to_string(),
			packages: vec![],
		};

		let files_to_remove = lockfile.update_modpack(modpack2, &[addon1]);

		assert_eq!(files_to_remove.len(), 1);
		assert_eq!(files_to_remove[0], "mods/mod2.jar");
		assert_eq!(lockfile.contents.addons.len(), 1);
	}

	#[test]
	fn test_minecraft_version_updates() {
		let mut lockfile = create_test_lockfile();

		assert_eq!(
			lockfile.get_minecraft_version(),
			Some(&"1.20.1".to_string())
		);

		lockfile.update_minecraft_version("1.21");
		assert_eq!(lockfile.get_minecraft_version(), Some(&"1.21".to_string()));
	}

	#[test]
	fn test_loader_updates() {
		let mut lockfile = create_test_lockfile();

		assert_eq!(lockfile.get_loader(), &Loader::Fabric);

		lockfile.update_loader(Loader::Forge);
		assert_eq!(lockfile.get_loader(), &Loader::Forge);
	}

	#[test]
	fn test_loader_version_updates() {
		let mut lockfile = create_test_lockfile();

		assert_eq!(lockfile.get_loader_version(), Some(&"0.14.0".to_string()));

		lockfile.update_loader_version(Some("0.15.0".to_string()));
		assert_eq!(lockfile.get_loader_version(), Some(&"0.15.0".to_string()));

		lockfile.update_loader_version(None);
		assert_eq!(lockfile.get_loader_version(), None);
	}

	#[test]
	fn test_mixed_addons_from_packages_and_modpack() {
		let mut lockfile = create_test_lockfile();
		let req = PkgRequest::parse("test-pkg", PkgRequestSource::UserRequire);

		// Add addon from package
		let pkg_addon = LockfileAddon {
			id: Some("addon1".to_string()),
			package: Some("test-pkg".to_string()),
			from_modpack: false,
			file_name: "addon1.jar".to_string(),
			files: vec!["mods/addon1.jar".to_string()],
			kind: AddonKind::Mod,
			hashes: AddonOptionalHashes::default(),
		};

		lockfile.update_package(&req, &[pkg_addon], None);

		// Add addon from modpack
		let modpack = LockfileModpack {
			name: "Test".to_string(),
			path: "/test".to_string(),
			packages: vec![],
		};

		let modpack_addon = LockfileAddon {
			id: None,
			package: None,
			from_modpack: true,
			file_name: "modpack_addon.jar".to_string(),
			files: vec!["mods/modpack_addon.jar".to_string()],
			kind: AddonKind::Mod,
			hashes: AddonOptionalHashes::default(),
		};

		lockfile.update_modpack(modpack, &[modpack_addon]);

		assert_eq!(lockfile.contents.addons.len(), 2);
		assert_eq!(
			lockfile
				.contents
				.addons
				.iter()
				.filter(|a| a.from_modpack)
				.count(),
			1
		);
		assert_eq!(
			lockfile
				.contents
				.addons
				.iter()
				.filter(|a| !a.from_modpack)
				.count(),
			1
		);
	}

	#[test]
	fn test_update_package_preserves_non_package_addons() {
		let mut lockfile = create_test_lockfile();
		let req = PkgRequest::parse("test-pkg", PkgRequestSource::UserRequire);

		// Add addon from package
		let pkg_addon = LockfileAddon {
			id: Some("addon1".to_string()),
			package: Some("test-pkg".to_string()),
			from_modpack: false,
			file_name: "addon1.jar".to_string(),
			files: vec!["mods/addon1.jar".to_string()],
			kind: AddonKind::Mod,
			hashes: AddonOptionalHashes::default(),
		};

		// Add standalone addon (not from package)
		let standalone_addon = LockfileAddon {
			id: Some("standalone".to_string()),
			package: None,
			from_modpack: false,
			file_name: "standalone.jar".to_string(),
			files: vec!["mods/standalone.jar".to_string()],
			kind: AddonKind::Mod,
			hashes: AddonOptionalHashes::default(),
		};

		lockfile.contents.addons.push(standalone_addon);
		lockfile.update_package(&req, &[pkg_addon], None);

		// Update package (which should not affect standalone addon)
		let updated_addon = LockfileAddon {
			id: Some("addon1".to_string()),
			package: Some("test-pkg".to_string()),
			from_modpack: false,
			file_name: "addon1_v2.jar".to_string(),
			files: vec!["mods/addon1_v2.jar".to_string()],
			kind: AddonKind::Mod,
			hashes: AddonOptionalHashes::default(),
		};

		lockfile.update_package(&req, &[updated_addon], None);

		assert_eq!(lockfile.contents.addons.len(), 2);
		assert!(
			lockfile
				.contents
				.addons
				.iter()
				.any(|a| a.id == Some("standalone".to_string()))
		);
	}

	#[test]
	fn test_addon_without_id() {
		let mut lockfile = create_test_lockfile();
		let req = PkgRequest::parse("test-pkg", PkgRequestSource::UserRequire);

		// Addon without ID (skipped in update_package loop)
		let addon_no_id = LockfileAddon {
			id: None,
			package: Some("test-pkg".to_string()),
			from_modpack: false,
			file_name: "addon_no_id.jar".to_string(),
			files: vec!["mods/addon_no_id.jar".to_string()],
			kind: AddonKind::Mod,
			hashes: AddonOptionalHashes::default(),
		};

		lockfile.update_package(&req, &[addon_no_id], None);

		assert_eq!(lockfile.contents.addons.len(), 1);
	}
}
