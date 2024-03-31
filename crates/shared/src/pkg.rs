#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::hash::Hash;
use std::sync::Arc;

use crate::util::is_valid_identifier;
use crate::versions::VersionPattern;

/// Type for the ID of a package
pub type PackageID = Arc<str>;

/// Used to store a request for a package that will be fulfilled later
#[derive(Debug, Clone, PartialOrd, Ord)]
pub struct PkgRequest {
	/// The source of this request.
	/// Could be a dependent, a recommender, or anything else.
	pub source: PkgRequestSource,
	/// The ID of the package to request
	pub id: PackageID,
	/// The requested content version of the package
	pub content_version: VersionPattern,
}

/// Where a package was requested from
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PkgRequestSource {
	/// This package was required by the user
	UserRequire,
	/// This package was bundled by another package
	Bundled(ArcPkgReq),
	/// This package was depended on by another package
	Dependency(ArcPkgReq),
	/// This package was refused by another package
	Refused(ArcPkgReq),
	/// This package was requested by some automatic system
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
	pub fn get_source(&self) -> Option<ArcPkgReq> {
		match self {
			Self::Dependency(source) | Self::Bundled(source) => Some(source.clone()),
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

impl PkgRequest {
	/// Create a new PkgRequest
	pub fn new(
		id: impl Into<PackageID>,
		source: PkgRequestSource,
		content_version: VersionPattern,
	) -> Self {
		Self {
			id: id.into(),
			source,
			content_version,
		}
	}

	/// Parse the package name and content version from a string
	pub fn parse(string: impl AsRef<str>, source: PkgRequestSource) -> Self {
		let string = string.as_ref();
		if let Some(index) = string.find('@') {
			let (id, mut version) = string.split_at(index);
			if index + 1 < string.len() {
				// Cut off the at symbol
				version = &version[1..];
				PkgRequest::new(id, source, VersionPattern::from(version))
			} else {
				PkgRequest::new(id, source, VersionPattern::Any)
			}
		} else {
			PkgRequest::new(string, source, VersionPattern::Any)
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

/// A PkgRequest wrapped in an Arc
pub type ArcPkgReq = Arc<PkgRequest>;

/// Stability setting for a package
#[derive(Deserialize, Serialize, Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum PackageStability {
	/// Whatever the latest stable version is
	#[default]
	Stable,
	/// Whatever the latest version is
	Latest,
}

impl PackageStability {
	/// Parse a PackageStability from a string
	pub fn parse_from_str(string: &str) -> Option<Self> {
		match string {
			"stable" => Some(Self::Stable),
			"latest" => Some(Self::Latest),
			_ => None,
		}
	}
}

/// The maximum length for a package identifier
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct PackageAddonHashes<T: Default> {
	/// The SHA-256 hash of this addon file
	pub sha256: T,
	/// The SHA-512 hash of this addon file
	pub sha512: T,
}

impl PackageAddonOptionalHashes {
	/// Checks if this set of optional hashes is empty
	pub fn is_empty(&self) -> bool {
		self.sha256.is_none() && self.sha512.is_none()
	}
}

/// Optional PackageAddonHashes
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
