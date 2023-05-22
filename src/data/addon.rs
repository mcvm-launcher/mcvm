use anyhow::Context;
use mcvm_shared::addon::{Addon, AddonKind};

use crate::io::files::paths::Paths;
use crate::io::files::{create_leading_dirs, update_hardlink};
use crate::net::download;
use mcvm_shared::modifications::{Modloader, ServerType};

use std::path::PathBuf;

/// Get the addon directory where an addon is stored
pub fn get_dir(addon: &Addon, paths: &Paths) -> PathBuf {
	paths.addons.join(addon.kind.to_plural_string())
}

/// Get the path to an addon stored in the internal addons folder
pub fn get_path(addon: &Addon, paths: &Paths) -> PathBuf {
	let pkg_dir = get_dir(addon, paths).join(&addon.pkg_id.name);
	if let Some(version) = &addon.version {
		pkg_dir.join(addon.id.clone()).join(version)
	} else {
		pkg_dir.join(format!("{}_unknown", addon.id))
	}
}

/// Whether this addon has different behavior based on the filename
pub fn filename_important(addon: &Addon) -> bool {
	matches!(addon.kind, AddonKind::ResourcePack)
}

/// Split an addon filename into base and extension
pub fn split_filename<'a>(addon: &'a Addon) -> (&'a str, &'a str) {
	if let Some(index) = addon.file_name.find('.') {
		addon.file_name.split_at(index)
	} else {
		(&addon.file_name, "")
	}
}

/// Get the filename of the addon file stored in the instance
pub fn get_instance_filename(addon: &Addon) -> String {
	if filename_important(addon) {
		addon.file_name.clone()
	} else {
		if let Some(version) = &addon.version {
			let (base, extension) = split_filename(addon);
			format!("{base}_{}{extension}", version)
		} else {
			addon.file_name.clone()
		}
	}
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
}

impl AddonRequest {
	pub fn new(addon: Addon, location: AddonLocation) -> Self {
		Self { addon, location }
	}

	/// Get the addon and store it
	pub async fn acquire(&self, paths: &Paths) -> anyhow::Result<()> {
		let path = get_path(&self.addon, paths);
		create_leading_dirs(&path)?;
		match &self.location {
			AddonLocation::Remote(url) => {
				download::file(url, &path)
					.await
					.context("Failed to download addon")?;
			}
			AddonLocation::Local(actual_path) => {
				update_hardlink(actual_path, &path).context("Failed to hardlink local addon")?;
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
	use mcvm_shared::pkg::PkgIdentifier;

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
		let addon = Addon::new(
			AddonKind::Mod,
			"foo",
			"FooBar.baz.jar",
			PkgIdentifier::new("package", 10),
			None,
		);
		assert_eq!(split_filename(&addon), ("FooBar", ".baz.jar"));
	}
}
