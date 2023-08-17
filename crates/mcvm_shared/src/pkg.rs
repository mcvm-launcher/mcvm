use serde::{Deserialize, Serialize};

use crate::util::is_valid_identifier;

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

pub static MAX_PACKAGE_ID_LENGTH: usize = 32;

/// Checks if a package identifier is valid
pub fn is_valid_package_id(id: &str) -> bool {
	if !is_valid_identifier(id) {
		return false;
	}

	for c in id.chars() {
		if c.is_ascii_uppercase() {
			return false;
		}
		if c == '_' || c == '.' {
			return false;
		}
	}

	if id.len() > MAX_PACKAGE_ID_LENGTH {
		return false;
	}

	true
}

/// Hashes used for package addons
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone, Default)]
#[serde(default)]
pub struct PackageAddonHashes<T> {
	pub sha256: T,
	pub sha512: T,
}

pub type PackageAddonOptionalHashes = PackageAddonHashes<Option<String>>;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_package_id_validation() {
		assert!(is_valid_package_id("hello"));
		assert!(is_valid_package_id("32"));
		assert!(is_valid_package_id("hello-world"));
		assert!(!is_valid_package_id("hello_world"));
		assert!(!is_valid_package_id("hello.world"));
		assert!(!is_valid_package_id("\\"));
		assert!(!is_valid_package_id(
			"very-very-long-long-long-package-name-thats-too-long"
		));
	}
}
