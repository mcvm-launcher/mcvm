use anyhow::Context;
use shared::addon::Addon;

use crate::io::files::{create_leading_dirs, update_hardlink};
use crate::io::files::paths::Paths;
use crate::net::download;
use shared::modifications::{Modloader, PluginLoader};

use std::path::PathBuf;

/// Get the addon directory where an addon is stored
pub fn get_addon_dir(addon: &Addon, paths: &Paths) -> PathBuf {
	paths.addons.join(addon.kind.to_plural_string())
}

/// Get the path to an addon
pub fn get_addon_path(addon: &Addon, paths: &Paths) -> PathBuf {
	get_addon_dir(addon, paths)
		.join(&addon.pkg_id.name)
		.join(addon.pkg_id.version.to_string())
		.join(&addon.file_name)
}

#[derive(Debug, Clone)]
pub enum AddonLocation {
	Remote(String),
	Local(PathBuf),
}

#[derive(Debug, Clone)]
pub struct AddonRequest {
	pub addon: Addon,
	location: AddonLocation,
	force: bool,
}

impl AddonRequest {
	pub fn new(addon: Addon, location: AddonLocation, force: bool) -> Self {
		Self {
			addon,
			location,
			force,
		}
	}

	/// Get the addon and store it
	pub async fn acquire(&self, paths: &Paths) -> anyhow::Result<()> {
		let path = get_addon_path(&self.addon, paths);
		if !self.force && path.exists() {
			return Ok(());
		}
		create_leading_dirs(&path)?;
		match &self.location {
			AddonLocation::Remote(url) => {
				download::file(url, &path)
					.await
					.context("Failed to download addon")?;
			}
			AddonLocation::Local(actual_path) => {
				update_hardlink(actual_path, &path)
					.context("Failed to hardlink local addon")?;
			}
		}
		Ok(())
	}
}

/// Checks if the modloader and plugin loader are compatible with each other
pub fn game_modifications_compatible(modloader: &Modloader, plugin_loader: &PluginLoader) -> bool {
	match (modloader, plugin_loader) {
		(Modloader::Vanilla, _) => true,
		(_, PluginLoader::Vanilla) => true,
		_ => false,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_game_mods_compat() {
		assert!(game_modifications_compatible(
			&Modloader::Fabric,
			&PluginLoader::Vanilla
		));
		assert!(game_modifications_compatible(
			&Modloader::Vanilla,
			&PluginLoader::Vanilla
		));
		assert!(!game_modifications_compatible(
			&Modloader::Forge,
			&PluginLoader::Paper
		));
	}
}
