use serde::{Deserialize, Serialize};

/// A known identifier for a package
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct PkgIdentifier {
	pub name: String,
	pub version: u32,
}

impl PkgIdentifier {
	pub fn new(name: &str, version: u32) -> Self {
		Self {
			name: name.to_owned(),
			version,
		}
	}
}

/// Stability setting for a package
#[derive(Deserialize, Serialize, Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum PackageStability {
	#[default]
	Stable,
	Latest,
}

impl PackageStability {
	pub fn parse_from_str(string: &str) -> Option<Self> {
		match string {
			"stable" => Some(Self::Stable),
			"latest" => Some(Self::Latest),
			_ => None,
		}
	}
}
