use std::fmt::Display;

use crate::pkg::PkgIdentifier;

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
		write!(
			f,
			"{}",
			match self {
				Self::ResourcePack => "resource_pack",
				Self::Mod => "mod",
				Self::Plugin => "plugin",
				Self::Shader => "shader",
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
}

impl Addon {
	pub fn new(kind: AddonKind, id: &str, file_name: &str, pkg_id: PkgIdentifier) -> Self {
		Self {
			kind,
			id: id.to_owned(),
			file_name: file_name.to_owned(),
			pkg_id,
		}
	}
}