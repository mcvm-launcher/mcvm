use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Default)]
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

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum ServerType {
	#[default]
	None,
	Vanilla,
	Paper,
	Forge,
	Fabric,
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

	pub fn matches(&self, other: &ServerType) -> bool {
		match self {
			Self::Vanilla => matches!(other, ServerType::Vanilla),
			Self::Bukkit => matches!(other, ServerType::Paper),
		}
	}
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum ClientType {
	#[default]
	None,
	Vanilla,
	Forge,
	Fabric,
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
