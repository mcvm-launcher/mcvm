use anyhow::Context;
use serde::Deserialize;

use crate::io::files::create_leading_dirs;
use crate::io::files::paths::Paths;
use crate::net::download::download_file;
use crate::package::reg::PkgIdentifier;

use std::fmt::Display;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub enum AddonKind {
	ResourcePack,
	Mod,
	Plugin,
	Shader,
}

impl AddonKind {
	pub fn from_str(string: &str) -> Option<Self> {
		match string {
			"resource_pack" => Some(Self::ResourcePack),
			"mod" => Some(Self::Mod),
			"plugin" => Some(Self::Plugin),
			"shader" => Some(Self::Shader),
			_ => None,
		}
	}

	/// Plural version of to_string
	pub fn to_plural_string(&self) -> String {
		match self {
			Self::ResourcePack => String::from("resource_packs"),
			Self::Mod => String::from("mods"),
			Self::Plugin => String::from("plugins"),
			Self::Shader => String::from("shaders"),
		}
	}
}

impl Display for AddonKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", match self {
			Self::ResourcePack => "resource_pack",
			Self::Mod => "mod",
			Self::Plugin => "plugin",
			Self::Shader => "shader",
		})
	}
}

#[derive(Debug, Clone)]
pub struct Addon {
	pub kind: AddonKind,
	pub name: String,
	pub id: PkgIdentifier,
}

impl Addon {
	pub fn new(kind: AddonKind, name: &str, id: PkgIdentifier) -> Self {
		Self {
			kind,
			name: name.to_owned(),
			id,
		}
	}

	/// Get the addon directory where this addon is stored
	pub fn get_dir(&self, paths: &Paths) -> PathBuf {
		paths.addons.join(self.kind.to_plural_string())
	}

	/// Get the path to the addon
	pub fn get_path(&self, paths: &Paths) -> PathBuf {
		self.get_dir(paths)
			.join(&self.id.name)
			.join(&self.id.version)
			.join(&self.name)
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
		let path = self.addon.get_path(paths);
		if !self.force && path.exists() {
			return Ok(());
		}
		create_leading_dirs(&path)?;
		match &self.location {
			AddonLocation::Remote(url) => {
				download_file(url, &path)
					.await
					.context("Failed to download addon")?;
			}
			AddonLocation::Local(actual_path) => {
				tokio::fs::hard_link(actual_path, path)
					.await
					.context("Failed to hardlink local addon")?;
			}
		}
		Ok(())
	}
}

#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Modloader {
	#[default]
	Vanilla,
	Forge,
	Fabric,
	Quilt,
}

impl Display for Modloader {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Vanilla => write!(f, "None"),
			Self::Forge => write!(f, "Forge"),
			Self::Fabric => write!(f, "Fabric"),
			Self::Quilt => write!(f, "Quilt"),
		}
	}
}

#[derive(Debug, Clone)]
pub enum ModloaderMatch {
	Vanilla,
	Forge,
	Fabric,
	Quilt,
	FabricLike,
}

impl ModloaderMatch {
	pub fn from_str(string: &str) -> Option<Self> {
		match string {
			"vanilla" => Some(Self::Vanilla),
			"forge" => Some(Self::Forge),
			"fabric" => Some(Self::Fabric),
			"quilt" => Some(Self::Quilt),
			"fabriclike" => Some(Self::FabricLike),
			_ => None,
		}
	}

	pub fn matches(&self, other: &Modloader) -> bool {
		match self {
			Self::Vanilla => matches!(other, Modloader::Vanilla),
			Self::Forge => matches!(other, Modloader::Forge),
			Self::Fabric => matches!(other, Modloader::Fabric),
			Self::Quilt => matches!(other, Modloader::Quilt),
			Self::FabricLike => matches!(other, Modloader::Fabric | Modloader::Quilt),
		}
	}
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum PluginLoader {
	#[default]
	Vanilla,
	Paper,
}

impl Display for PluginLoader {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Vanilla => write!(f, "None"),
			Self::Paper => write!(f, "Paper"),
		}
	}
}

#[derive(Debug, Clone)]
pub enum PluginLoaderMatch {
	Vanilla,
	BukkitLike,
}

impl PluginLoaderMatch {
	pub fn from_str(string: &str) -> Option<Self> {
		match string {
			"vanilla" => Some(Self::Vanilla),
			"bukkitlike" => Some(Self::BukkitLike),
			_ => None,
		}
	}

	pub fn matches(&self, other: &PluginLoader) -> bool {
		match self {
			Self::Vanilla => matches!(other, PluginLoader::Vanilla),
			Self::BukkitLike => matches!(other, PluginLoader::Paper),
		}
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
