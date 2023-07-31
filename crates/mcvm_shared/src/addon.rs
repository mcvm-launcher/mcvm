use std::fmt::Display;

use crate::pkg::PkgIdentifier;

#[derive(Debug, Clone, Copy)]
pub enum AddonKind {
	ResourcePack,
	Mod,
	Plugin,
	Shader,
	Datapack,
}

impl AddonKind {
	pub fn from_str(string: &str) -> Option<Self> {
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
			Self::ResourcePack => String::from("resource_packs"),
			Self::Mod => String::from("mods"),
			Self::Plugin => String::from("plugins"),
			Self::Shader => String::from("shaders"),
			Self::Datapack => String::from("datapacks"),
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

#[derive(Debug, Clone)]
pub struct Addon {
	pub kind: AddonKind,
	pub id: String,
	pub file_name: String,
	pub pkg_id: PkgIdentifier,
	/// Version of the addon, used for caching
	pub version: Option<String>,
}

impl Addon {
	pub fn new(
		kind: AddonKind,
		id: &str,
		file_name: &str,
		pkg_id: PkgIdentifier,
		version: Option<String>,
	) -> Self {
		Self {
			kind,
			id: id.to_owned(),
			file_name: file_name.to_owned(),
			pkg_id,
			version,
		}
	}
}

/// Checks for a valid addon version identifier that is compatible with all systems
pub fn is_addon_version_valid(version: &str) -> bool {
	if !version.is_ascii() {
		return false;
	}

	for c in version.chars() {
		if !c.is_ascii_alphanumeric() {
			return false;
		}
	}

	true
}

/// Checks for a valid addon filename
pub fn is_filename_valid(kind: AddonKind, filename: &str) -> bool {
	filename.ends_with(kind.get_extension())
}
