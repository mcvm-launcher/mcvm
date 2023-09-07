use anyhow::{bail, Context};
use mcvm_shared::addon::{Addon, AddonKind};
use reqwest::Client;

use crate::io::files::paths::Paths;
use crate::io::files::{create_leading_dirs, update_hardlink};
use crate::net::download;
use crate::util::hash::{get_best_hash, hash_file_with_best_hash};
use mcvm_shared::modifications::{Modloader, ServerType};

use std::path::{Path, PathBuf};

/// Get the addon directory where an addon is stored
pub fn get_dir(addon: &Addon, paths: &Paths) -> PathBuf {
	paths.addons.join(addon.kind.to_plural_string())
}

/// Get the path to an addon stored in the internal addons folder
pub fn get_path(addon: &Addon, paths: &Paths, instance_id: &str) -> PathBuf {
	let pkg_dir = get_dir(addon, paths).join(&addon.pkg_id.id);
	if let Some(version) = &addon.version {
		pkg_dir.join(addon.id.clone()).join(version)
	} else {
		pkg_dir.join(format!("{}_{instance_id}", addon.id))
	}
}

/// Whether this addon has different behavior based on the filename
pub fn filename_important(addon: &Addon) -> bool {
	matches!(addon.kind, AddonKind::ResourcePack)
}

/// Gets the formulaic filename for an addon in the instance, meant to reduce name clashes
pub fn get_addon_instance_filename(package_id: &str, id: &str, kind: &AddonKind) -> String {
	format!("mcvm_{package_id}_{id}{}", kind.get_extension())
}

/// Split an addon filename into base and extension
pub fn split_filename(addon: &Addon) -> (&str, &str) {
	if let Some(index) = addon.file_name.find('.') {
		addon.file_name.split_at(index)
	} else {
		(&addon.file_name, "")
	}
}

/// Whether this addon should be updated
pub fn should_update(addon: &Addon, paths: &Paths, instance_id: &str) -> bool {
	addon.version.is_none() || !get_path(addon, paths, instance_id).exists()
}

/// Checks if this path is in the stored addons directory
pub fn is_stored_addon_path(path: &Path, paths: &Paths) -> bool {
	path.starts_with(&paths.addons)
}

/// The location of an addon
#[derive(Debug, Clone)]
pub enum AddonLocation {
	/// Located at a remote URL
	Remote(String),
	/// Located on the local filesystem
	Local(PathBuf),
}

/// A request for an addon file that will be fulfilled later
#[derive(Debug, Clone)]
pub struct AddonRequest {
	/// The addon that will be retrieved
	pub addon: Addon,
	/// Where the addon is located
	location: AddonLocation,
}

impl AddonRequest {
	/// Create a new AddonRequest from an addon and location
	pub fn new(addon: Addon, location: AddonLocation) -> Self {
		Self { addon, location }
	}

	/// Get the addon and store it
	pub async fn acquire(
		&self,
		paths: &Paths,
		instance_id: &str,
		client: &Client,
	) -> anyhow::Result<()> {
		let path = get_path(&self.addon, paths, instance_id);
		create_leading_dirs(&path)?;
		match &self.location {
			AddonLocation::Remote(url) => {
				download::file(url, &path, client)
					.await
					.context("Failed to download addon")?;
			}
			AddonLocation::Local(actual_path) => {
				update_hardlink(actual_path, &path).context("Failed to hardlink local addon")?;
			}
		}

		let result = self.check_hashes(&path);
		// Remove the addon file if it fails the checksum
		if result.is_err() {
			std::fs::remove_file(path).context("Failed to remove stored addon file")?;
		}
		result?;

		Ok(())
	}

	/// Check the addon's hashes. The stored addon file must exist at this time
	fn check_hashes(&self, path: &Path) -> anyhow::Result<()> {
		let best_hash = get_best_hash(&self.addon.hashes);
		if let Some(best_hash) = best_hash {
			let matches = hash_file_with_best_hash(path, best_hash)
				.context("Failed to checksum addon file")?;

			if !matches {
				bail!("Checksum for addon file does not match");
			}
		}

		Ok(())
	}
}

/// Checks if the modloader and plugin loader are compatible with each other
pub fn game_modifications_compatible(modloader: &Modloader, plugin_loader: &ServerType) -> bool {
	matches!(
		(modloader, plugin_loader),
		(Modloader::Vanilla, _) | (_, ServerType::Vanilla)
	)
}

#[cfg(test)]
mod tests {
	use mcvm_shared::pkg::{PackageAddonOptionalHashes, PkgIdentifier};

	use super::*;

	#[test]
	fn test_game_mods_compat() {
		assert!(game_modifications_compatible(
			&Modloader::Fabric,
			&ServerType::Vanilla
		));
		assert!(game_modifications_compatible(
			&Modloader::Vanilla,
			&ServerType::Vanilla
		));
		assert!(!game_modifications_compatible(
			&Modloader::Forge,
			&ServerType::Paper
		));
	}

	#[test]
	fn test_addon_split_filename() {
		let addon = Addon {
			kind: AddonKind::Mod,
			id: "foo".into(),
			file_name: "FooBar.baz.jar".into(),
			pkg_id: PkgIdentifier::new("package", 10),
			version: None,
			hashes: PackageAddonOptionalHashes::default(),
		};
		assert_eq!(split_filename(&addon), ("FooBar", ".baz.jar"));
	}
}
