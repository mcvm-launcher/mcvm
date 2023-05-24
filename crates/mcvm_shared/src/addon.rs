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

/// Checks for a valid addon filename
pub fn is_filename_valid(kind: AddonKind, filename: &str) -> bool {
	match kind {
		AddonKind::Mod | AddonKind::Plugin => filename.ends_with(".jar"),
		AddonKind::ResourcePack | AddonKind::Shader | AddonKind::Datapack => {
			filename.ends_with(".zip")
		}
	}
}
