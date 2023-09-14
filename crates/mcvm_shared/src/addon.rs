use std::fmt::Display;

use serde::Deserialize;

use crate::pkg::{PackageAddonOptionalHashes, PackageID};

/// Some content that is installed on Minecraft
#[derive(Debug, Clone)]
pub struct Addon {
	/// What type of addon this is
	pub kind: AddonKind,
	/// The ID of this addon, unique among a package
	pub id: String,
	/// The addon's file name
	pub file_name: String,
	/// The ID of the package that installed this addon
	pub pkg_id: PackageID,
	/// Version of the addon, used for caching
	pub version: Option<String>,
	/// Hashes of the addon
	pub hashes: PackageAddonOptionalHashes,
}

/// Different kinds of addons
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonKind {
	/// A Minecraft resource pack
	ResourcePack,
	/// A game modification that needs to be loaded by a custom loader
	Mod,
	/// A server plugin that modifies game behavior
	Plugin,
	/// A graphics shader that needs to be loaded by a shader modification
	Shader,
	/// A Minecraft datapack
	Datapack,
}

impl AddonKind {
	/// Parse an AddonKind from a string
	pub fn parse_from_str(string: &str) -> Option<Self> {
		match string {
			"resource_pack" => Some(Self::ResourcePack),
			"mod" => Some(Self::Mod),
			"plugin" => Some(Self::Plugin),
			"shader" => Some(Self::Shader),
			"datapack" => Some(Self::Datapack),
			_ => None,
		}
	}

	/// Plural version of to_string
	pub fn to_plural_string(&self) -> String {
		match self {
			Self::ResourcePack => "resource_packs".into(),
			Self::Mod => "mods".into(),
			Self::Plugin => "plugins".into(),
			Self::Shader => "shaders".into(),
			Self::Datapack => "datapacks".into(),
		}
	}

	/// Gets the file extension for this addon kind
	pub fn get_extension(&self) -> &str {
		match self {
			AddonKind::Mod | AddonKind::Plugin => ".jar",
			AddonKind::ResourcePack | AddonKind::Shader | AddonKind::Datapack => ".zip",
		}
	}
}

impl Display for AddonKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::ResourcePack => "resource_pack",
				Self::Mod => "mod",
				Self::Plugin => "plugin",
				Self::Shader => "shader",
				Self::Datapack => "datapack",
			}
		)
	}
}

/// Checks for a valid addon version identifier that is compatible with all systems
pub fn is_addon_version_valid(version: &str) -> bool {
	if !version.is_ascii() {
		return false;
	}

	for c in version.chars() {
		if !c.is_ascii_alphanumeric() && c != '-' {
			return false;
		}
	}

	true
}

/// Checks for a valid addon filename
pub fn is_filename_valid(kind: AddonKind, filename: &str) -> bool {
	filename.ends_with(kind.get_extension())
}
