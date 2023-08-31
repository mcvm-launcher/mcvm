use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::hash::Hash;

use crate::util::is_valid_identifier;

/// A known identifier for a package
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct PkgIdentifier {
	pub id: String,
	pub version: u32,
}

impl PkgIdentifier {
	pub fn new(id: &str, version: u32) -> Self {
		Self {
			id: id.to_owned(),
			version,
		}
	}
}

/// Where a package was requested from
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PkgRequestSource {
	UserRequire,
	Bundled(Box<PkgRequest>),
	Dependency(Box<PkgRequest>),
	Refused(Box<PkgRequest>),
	Repository,
}

impl Ord for PkgRequestSource {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.to_num().cmp(&other.to_num())
	}
}

impl PartialOrd for PkgRequestSource {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl PkgRequestSource {
	/// Gets the source package of this package, if any
	pub fn get_source(&self) -> Option<&PkgRequest> {
		match self {
			Self::Dependency(source) | Self::Bundled(source) => Some(source),
			_ => None,
		}
	}

	/// Gets whether this source list is only bundles that lead up to a UserRequire
	pub fn is_user_bundled(&self) -> bool {
		matches!(self, Self::Bundled(source) if source.source.is_user_bundled())
			|| matches!(self, Self::UserRequire)
	}

	/// Converts to a number, used for ordering
	fn to_num(&self) -> u8 {
		match self {
			Self::UserRequire => 0,
			Self::Bundled(..) => 1,
			Self::Dependency(..) => 2,
			Self::Refused(..) => 3,
			Self::Repository => 4,
		}
	}
}

/// Used to store a request for a package that will be fulfilled later
#[derive(Debug, Clone, PartialOrd, Ord)]
pub struct PkgRequest {
	pub source: PkgRequestSource,
	pub id: String,
}

impl PkgRequest {
	pub fn new(id: &str, source: PkgRequestSource) -> Self {
		Self {
			id: id.to_owned(),
			source,
		}
	}

	/// Create a dependency list for debugging. Recursive, so call with an empty string
	pub fn debug_sources(&self, list: String) -> String {
		match &self.source {
			PkgRequestSource::UserRequire => format!("{}{list}", self.id),
			PkgRequestSource::Dependency(source) => {
				format!("{} -> {}", source.debug_sources(list), self.id)
			}
			PkgRequestSource::Refused(source) => {
				format!("{} =X=> {}", source.debug_sources(list), self.id)
			}
			PkgRequestSource::Bundled(bundler) => {
				format!("{} => {}", bundler.debug_sources(list), self.id)
			}
			PkgRequestSource::Repository => format!("Repository -> {}{list}", self.id),
		}
	}
}

impl PartialEq for PkgRequest {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

impl Eq for PkgRequest {}

impl Hash for PkgRequest {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.id.hash(state);
	}
}

impl Display for PkgRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.id)
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

pub const MAX_PACKAGE_ID_LENGTH: usize = 32;

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
