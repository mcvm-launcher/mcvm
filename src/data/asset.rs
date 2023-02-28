use std::path::PathBuf;

use crate::{io::files::paths::Paths, package::reg::PkgIdentifier};

#[derive(Debug, Clone)]
pub enum AssetKind {
	ResourcePack,
	Datapack,
	Mod,
	Plugin
}

impl AssetKind {
	pub fn from_str(string: &str) -> Option<Self> {
		match string {
			"resource_pack" => Some(Self::ResourcePack),
			"datapack" => Some(Self::Datapack),
			"mod" => Some(Self::Mod),
			"plugin" => Some(Self::Plugin),
			_ => None
		}
	}

	pub fn to_plural_string(&self) -> String {
		match self {
			Self::ResourcePack => String::from("resource_packs"),
			Self::Datapack => String::from("datapacks"),
			Self::Mod => String::from("mods"),
			Self::Plugin => String::from("plugins")
		}
	}
}

pub struct Asset {
	pub kind: AssetKind,
	pub name: String,
	pub id: PkgIdentifier
}

impl Asset {
	pub fn new(kind: AssetKind, name: &str, id: PkgIdentifier) -> Self {
		Self {
			kind,
			name: name.to_owned(),
			id
		}
	}

	pub fn get_dir(&self, paths: &Paths) -> PathBuf {
		paths.mcvm_assets.join(self.kind.to_plural_string())
	}

	pub fn get_path(&self, paths: &Paths) -> PathBuf {
		self.get_dir(paths).join(&self.id.name).join(&self.id.version).join(&self.name)
	}
}

pub struct AssetDownload {
	asset: Asset,
	url: String
}

impl AssetDownload {
	pub fn new(asset: Asset, url: &str) -> Self {
		Self {
			asset,
			url: url.to_owned()
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum Modloader {
	Vanilla,
	Forge,
	Fabric
}

impl Modloader {
	pub fn from_str(string: &str) -> Option<Self> {
		match string {
			"vanilla" => Some(Self::Vanilla),
			"forge" => Some(Self::Forge),
			"fabric" => Some(Self::Fabric),
			_ => None
		}
	}
}
