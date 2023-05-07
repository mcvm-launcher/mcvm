use std::fmt::Display;

use serde::Deserialize;

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
	Bukkit,
}

impl PluginLoaderMatch {
	pub fn from_str(string: &str) -> Option<Self> {
		match string {
			"vanilla" => Some(Self::Vanilla),
			"bukkit" => Some(Self::Bukkit),
			_ => None,
		}
	}

	pub fn matches(&self, other: &PluginLoader) -> bool {
		match self {
			Self::Vanilla => matches!(other, PluginLoader::Vanilla),
			Self::Bukkit => matches!(other, PluginLoader::Paper),
		}
	}
}
