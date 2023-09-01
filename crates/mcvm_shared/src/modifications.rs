use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// A loader for Minecraft mods
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Modloader {
	/// No loader, just the default game
	#[default]
	Vanilla,
	/// MinecraftForge
	Forge,
	/// Fabric Loader
	Fabric,
	/// Quilt Loader
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

/// Matcher for different types of loader
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModloaderMatch {
	/// Matches vanilla
	Vanilla,
	/// Matches MinecraftForge
	Forge,
	/// Matches Fabric Loader
	Fabric,
	/// Matches Quilt Loader
	Quilt,
	/// Matches any loader that supports loading Fabric mods
	#[serde(rename = "fabriclike")]
	FabricLike,
}

impl ModloaderMatch {
	/// Parse a ModloaderMatch from a string
	pub fn parse_from_str(string: &str) -> Option<Self> {
		match string {
			"vanilla" => Some(Self::Vanilla),
			"forge" => Some(Self::Forge),
			"fabric" => Some(Self::Fabric),
			"quilt" => Some(Self::Quilt),
			"fabriclike" => Some(Self::FabricLike),
			_ => None,
		}
	}

	/// Checks if a modloader matches
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

/// Different types of server changes. These are mostly mutually exclusive.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum ServerType {
	/// Unspecified. Usually inherits from something else
	#[default]
	None,
	/// No modifications, just the default game
	Vanilla,
	/// Paper server
	Paper,
	/// MinecraftForge
	Forge,
	/// Fabric Loader
	Fabric,
	/// Quilt Loader
	Quilt,
}

impl Display for ServerType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::None => write!(f, "Vanilla"),
			Self::Vanilla => write!(f, "Vanilla"),
			Self::Forge => write!(f, "Forge"),
			Self::Fabric => write!(f, "Fabric"),
			Self::Quilt => write!(f, "Quilt"),
			Self::Paper => write!(f, "Paper"),
		}
	}
}

/// Matcher for different types of server plugin loaders
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginLoaderMatch {
	/// The default game with no plugin support
	Vanilla,
	/// Matches any server that can load Bukkit plugins
	Bukkit,
}

impl PluginLoaderMatch {
	/// Parse a PluginLoaderMatch from a string
	pub fn parse_from_str(string: &str) -> Option<Self> {
		match string {
			"vanilla" => Some(Self::Vanilla),
			"bukkit" => Some(Self::Bukkit),
			_ => None,
		}
	}

	/// Checks if a plugin loader matches
	pub fn matches(&self, other: &ServerType) -> bool {
		match self {
			Self::Vanilla => matches!(other, ServerType::Vanilla),
			Self::Bukkit => matches!(other, ServerType::Paper),
		}
	}
}

/// Different modifications for the client. Mostly mututally exclusive
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum ClientType {
	/// Unspecified. Usually inherits from something else
	#[default]
	None,
	/// No modifications, just the default game
	Vanilla,
	/// MinecraftForge
	Forge,
	/// Fabric Loader
	Fabric,
	/// Quilt Loader
	Quilt,
}

impl Display for ClientType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::None => write!(f, "Vanilla"),
			Self::Vanilla => write!(f, "Vanilla"),
			Self::Forge => write!(f, "Forge"),
			Self::Fabric => write!(f, "Fabric"),
			Self::Quilt => write!(f, "Quilt"),
		}
	}
}
