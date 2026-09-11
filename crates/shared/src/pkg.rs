use anyhow::bail;
use itertools::Itertools;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;
use std::sync::Arc;

use crate::loaders::Loader;
use crate::minecraft::AddonKind;
use crate::util::is_valid_identifier;
use crate::versions::{VersionPattern, parse_versioned_string};

/// Type for the ID of a package
pub type PackageID = Arc<str>;

/// Used to store a request for a package that will be fulfilled later
#[derive(Debug, Clone, PartialOrd, Ord, Deserialize, Serialize)]
pub struct PkgRequest {
	/// The source of this request.
	/// Could be a dependent, a recommender, or anything else.
	#[serde(default)]
	pub source: PkgRequestSource,
	/// The ID of the package to request
	pub id: PackageID,
	/// The requested repository of the package
	#[serde(default)]
	pub repository: Option<String>,
	/// The requested content version of the package
	#[serde(default)]
	pub content_version: VersionPattern,
	/// The optional slug specifier for this request. Just for display, and does not affect functionality
	#[serde(default)]
	pub slug: Option<String>,
}

/// Where a package was requested from
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PkgRequestSource {
	/// This package was required by the user
	#[default]
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

	/// Gets the original source package all the way up the chain
	pub fn get_original_source(&self) -> Option<&ArcPkgReq> {
		match self {
			Self::Dependency(source) | Self::Bundled(source) => match &source.source {
				Self::Dependency(..) | Self::Bundled(..) => source.source.get_original_source(),
				_ => Some(source),
			},
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
	#[inline(always)]
	pub fn new(
		id: impl Into<PackageID>,
		source: PkgRequestSource,
		content_version: VersionPattern,
		repository: Option<String>,
	) -> Self {
		Self {
			id: id.into(),
			source,
			content_version,
			repository,
			slug: None,
		}
	}

	/// Create a new PkgRequest that matches all content versions and repositories
	#[inline(always)]
	pub fn any(id: impl Into<PackageID>, source: PkgRequestSource) -> Self {
		Self::new(id, source, VersionPattern::Any, None)
	}

	/// Parse the package name and content version from a string
	pub fn parse(string: impl AsRef<str>, source: PkgRequestSource) -> Self {
		let string = string.as_ref();
		let (id_and_repo, version) = parse_versioned_string(string);

		let (id, repository) = if let Some((repo, id)) = id_and_repo.split_once(":") {
			// Empty repository should just be none
			(id, Some(repo).filter(|x| !x.is_empty()))
		} else {
			(id_and_repo, None)
		};

		let (id, slug) = if let Some((slug, id)) = id.split_once(".") {
			(id, Some(slug))
		} else {
			(id, None)
		};

		Self {
			source,
			id: id.into(),
			content_version: version,
			repository: repository.map(|x| x.to_string()),
			slug: slug.map(|x| x.to_string()),
		}
	}

	/// Create a new request with the content version changed
	pub fn with_content_version(&self, content_version: VersionPattern) -> Self {
		Self {
			source: self.source.clone(),
			id: self.id.clone(),
			repository: self.repository.clone(),
			content_version,
			slug: self.slug.clone(),
		}
	}

	/// Create a new request with the slug changed if it is not already present
	pub fn with_slug(&self, slug: Option<String>) -> Self {
		Self {
			source: self.source.clone(),
			id: self.id.clone(),
			repository: self.repository.clone(),
			content_version: self.content_version.clone(),
			slug: slug.or(self.slug.clone()),
		}
	}

	/// Puts this request inside of an Arc
	pub fn arc(self) -> ArcPkgReq {
		Arc::new(self)
	}

	/// Create a dependency list for debugging
	pub fn debug_sources(&self) -> String {
		self.debug_sources_inner(String::new())
	}

	/// Converts to a string without the content version
	pub fn to_string_no_version(&self) -> String {
		let mut out = String::new();
		if let Some(repo) = &self.repository {
			out.push_str(&format!("{repo}:"));
		}
		if let Some(slug) = &self.slug {
			out.push_str(&format!("{slug}.{}", self.id));
		} else {
			out.push_str(&self.id);
		}

		out
	}

	/// Converts to repository:id or id, omitting the slug and content version
	pub fn to_string_no_version_or_slug(&self) -> String {
		if let Some(repo) = &self.repository {
			format!("{repo}:{}", self.id)
		} else {
			self.id.to_string()
		}
	}

	/// Recursive inner function for debugging sources
	fn debug_sources_inner(&self, list: String) -> String {
		match &self.source {
			PkgRequestSource::UserRequire => format!("{}{list}", self.id),
			PkgRequestSource::Dependency(source) => {
				format!("{} -> {}", source.debug_sources_inner(list), self.id)
			}
			PkgRequestSource::Refused(source) => {
				format!("{} =X=> {}", source.debug_sources_inner(list), self.id)
			}
			PkgRequestSource::Bundled(bundler) => {
				format!("{} => {}", bundler.debug_sources_inner(list), self.id)
			}
			PkgRequestSource::Repository => format!("Repository -> {}{list}", self.id),
		}
	}
}

impl PartialEq for PkgRequest {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id && self.repository == other.repository
	}
}

impl Eq for PkgRequest {}

impl Hash for PkgRequest {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.id.hash(state);
		self.repository.hash(state);
	}
}

impl Display for PkgRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if let Some(repo) = &self.repository {
			write!(f, "{repo}:")?;
		}

		if let Some(slug) = &self.slug {
			write!(f, "{slug}.{}", self.id)?;
		} else {
			write!(f, "{}", self.id)?;
		}

		if self.content_version != VersionPattern::Any {
			write!(f, "@{}", self.content_version)?;
		}

		Ok(())
	}
}

impl From<&str> for PkgRequest {
	fn from(string: &str) -> Self {
		Self::parse(string, PkgRequestSource::UserRequire)
	}
}

impl From<&String> for PkgRequest {
	fn from(string: &String) -> Self {
		Self::parse(string, PkgRequestSource::UserRequire)
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
	Stable,
	/// Whatever the latest version is
	#[default]
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
pub struct AddonHashes<T: Default> {
	/// The SHA-256 hash of this addon file
	pub sha256: T,
	/// The SHA-512 hash of this addon file
	pub sha512: T,
	/// The SHA1 hash of this addon file
	pub sha1: T,
}

impl AddonOptionalHashes {
	/// Checks if this set of optional hashes is empty
	pub fn is_empty(&self) -> bool {
		self.sha256.is_none() && self.sha512.is_none()
	}
}

/// Optional AddonHashes
pub type AddonOptionalHashes = AddonHashes<Option<String>>;

/// Different types of packages, mostly AddonKinds
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum PackageKind {
	/// A mod package
	Mod,
	/// A resource pack package
	ResourcePack,
	/// A datapack package
	Datapack,
	/// A plugin package
	Plugin,
	/// A shader package
	Shader,
	/// A package that bundles other packages
	Bundle,
	/// A modpack
	Modpack,
}

impl PackageKind {
	/// Converts to an addon kind, if possible
	pub fn to_addon_kind(&self) -> Option<AddonKind> {
		match self {
			Self::Mod => Some(AddonKind::Mod),
			Self::ResourcePack => Some(AddonKind::ResourcePack),
			Self::Datapack => Some(AddonKind::Datapack),
			Self::Plugin => Some(AddonKind::Plugin),
			Self::Shader => Some(AddonKind::Shader),
			Self::Modpack => Some(AddonKind::Modpack),
			Self::Bundle => None,
		}
	}

	/// Converts from an addon kind
	pub fn from_addon_kind(kind: AddonKind) -> Self {
		match kind {
			AddonKind::Mod => Self::Mod,
			AddonKind::ResourcePack => Self::ResourcePack,
			AddonKind::Datapack => Self::Datapack,
			AddonKind::Plugin => Self::Plugin,
			AddonKind::Shader => Self::Shader,
			AddonKind::Modpack => Self::Modpack,
		}
	}

	/// Converts to a capitalized string
	pub fn to_string_pretty(&self) -> &'static str {
		match self {
			Self::Mod => "Mod",
			Self::ResourcePack => "Resource pack",
			Self::Datapack => "Datapack",
			Self::Plugin => "Plugin",
			Self::Shader => "Shader",
			Self::Modpack => "Modpack",
			Self::Bundle => "Bundle",
		}
	}
}

impl FromStr for PackageKind {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"mod" => Ok(Self::Mod),
			"resource_pack" => Ok(Self::ResourcePack),
			"datapack" => Ok(Self::Datapack),
			"plugin" => Ok(Self::Plugin),
			"shader" => Ok(Self::Shader),
			"bundle" => Ok(Self::Bundle),
			"modpack" => Ok(Self::Modpack),
			other => bail!("Unknown package type '{other}'"),
		}
	}
}

impl Display for PackageKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Mod => "mod",
				Self::ResourcePack => "resource_pack",
				Self::Datapack => "datapack",
				Self::Plugin => "plugin",
				Self::Shader => "shader",
				Self::Modpack => "modpack",
				Self::Bundle => "bundle",
			}
		)
	}
}

/// Parameters for a package search
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq, Hash)]
pub struct PackageSearchParameters {
	/// The number of packages to return
	pub count: u8,
	/// How many results to skip
	pub skip: usize,
	/// The fuzzy search term for ids, names, or descriptions
	pub search: Option<String>,
	/// The addon kinds / package types to include
	pub types: Vec<PackageKind>,
	/// The Minecraft versions to include
	pub minecraft_versions: Vec<String>,
	/// The loaders to include
	pub loaders: Vec<Loader>,
	/// The package categories to include
	pub categories: Vec<PackageCategory>,
}

/// How much of a package we want to query depending on what operation we are doing.
/// Allows lazy-loading parts of a package that we don't need
#[derive(Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq)]
pub enum PackageQueryDepth {
	/// Metadata and partial properties only
	MetaAndProps,
	/// The whole package
	#[default]
	Full,
}

/// A category for a package
#[allow(missing_docs)]
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum PackageCategory {
	Adventure,
	Atmosphere,
	Audio,
	Blocks,
	Building,
	Cartoon,
	Challenge,
	Combat,
	Compatability,
	Decoration,
	Economy,
	Entities,
	Equipment,
	Exploration,
	Extensive,
	Fantasy,
	Fonts,
	Food,
	GameMechanics,
	Gui,
	Items,
	Language,
	Library,
	Lightweight,
	Magic,
	Minigame,
	Mobs,
	Multiplayer,
	Optimization,
	Realistic,
	Simplistic,
	Space,
	Social,
	Storage,
	Structures,
	Technology,
	Transportation,
	Tweaks,
	Utility,
	VanillaPlus,
	Worldgen,
}

/// A list of overrides that apply to the package installation process
#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct PackageOverrides {
	/// Packages to not install
	pub suppress: Vec<String>,
	/// Packages to force installation of
	pub force: Vec<String>,
}

impl PackageOverrides {
	/// Merges another set of overrides on top of this one
	pub fn merge(&mut self, other: Self) {
		self.suppress = merge_package_lists(self.suppress.clone().into_iter(), &other.suppress);
		self.force = merge_package_lists(self.force.clone().into_iter(), &other.force);
	}
}

/// Checks if a package is overridden in a list
pub fn is_package_overridden(package: &PkgRequest, list: &[String]) -> bool {
	list.iter()
		.map(|x| PkgRequest::parse(x, PkgRequestSource::UserRequire))
		.any(|x| &x == package)
}

/// Merges two package lists, removing duplicates and preferring requests with a version
pub fn merge_package_lists(list1: impl Iterator<Item = String>, list2: &[String]) -> Vec<String> {
	let mut out: Vec<_> = list1
		.map(|x| PkgRequest::parse(x, PkgRequestSource::UserRequire))
		.collect();
	for item in list2 {
		let item = PkgRequest::parse(item, PkgRequestSource::UserRequire);
		if let Some(existing) = out.iter_mut().find(|x| **x == item) {
			if existing.content_version == VersionPattern::Any {
				existing.content_version = item.content_version.clone();
			}
		} else {
			out.push(item);
		}
	}

	out.into_iter().map(|x| x.to_string()).collect()
}

/// Error from package resolution
#[allow(missing_docs)]
#[derive(thiserror::Error, Debug)]
pub enum ResolutionError {
	/// Error that happens when resolving a single package
	#[error("When resolving the package {0}:\n{1}")]
	PackageContext(ArcPkgReq, Box<ResolutionError>),
	#[error("Failed to preload packages")]
	FailedToPreload(anyhow::Error),
	#[error("Failed to get properties of package {0}:\n{1}")]
	FailedToGetProperties(ArcPkgReq, anyhow::Error),
	#[error("No valid versions found for package {0}. Constraints: {1:?}")]
	NoValidVersionsFound(ArcPkgReq, Vec<VersionPattern>),
	#[error("{pkg} extends the functionality of the package {1}, which is not installed", pkg = .0.as_ref().map(|x| format!("The package {}", x.debug_sources())).unwrap_or("A package".into()))]
	ExtensionNotFulfilled(Option<ArcPkgReq>, ArcPkgReq),
	#[error(
		"Package {0} has been explicitly required by package {1}. This means it must be required by the user in their config."
	)]
	ExplicitRequireNotFulfilled(ArcPkgReq, ArcPkgReq),
	#[error("Package {0} is incompatible with the packages {refusers}", refusers = .1.iter().join(", "))]
	IncompatiblePackage(ArcPkgReq, Vec<Arc<str>>),
	#[error("Failed to evaluate package {0}:\n{1:?}")]
	FailedToEvaluate(ArcPkgReq, anyhow::Error),
	#[error("Miscellaneous error:\n{0:?}")]
	Misc(anyhow::Error),
}

/// A change to an installed package, used for user display
#[derive(Clone, PartialEq)]
pub enum PackageDiff {
	/// A new package was added
	Added(ArcPkgReq),
	/// A large number of packages were added
	ManyAdded(u16),
	/// An existing package was removed
	Removed(ArcPkgReq),
	/// A large number of packages were removed
	ManyRemoved(u16),
	/// An existing package had it's version changed. Contains the old and new version
	VersionChanged(ArcPkgReq, String, String),
}

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

	#[test]
	fn test_request_source_debug() {
		let req = PkgRequest::parse(
			"foo",
			PkgRequestSource::Dependency(Arc::new(PkgRequest::parse(
				"bar",
				PkgRequestSource::Dependency(Arc::new(PkgRequest::parse(
					"baz",
					PkgRequestSource::Repository,
				))),
			))),
		);
		let debug = req.debug_sources();
		assert_eq!(debug, "Repository -> baz -> bar -> foo");
	}

	#[test]
	fn test_pkg_req_parsing() {
		let req = PkgRequest::parse("foo", PkgRequestSource::UserRequire);
		assert_eq!(req.id, "foo".into());
		assert_eq!(req.repository, None);
		let req = PkgRequest::parse("foo@1.19.2", PkgRequestSource::UserRequire);
		assert_eq!(req.id, "foo".into());
		assert_eq!(req.content_version, VersionPattern::Single("1.19.2".into()));
		let req = PkgRequest::parse("modrinth:foo@1.19.2", PkgRequestSource::UserRequire);
		assert_eq!(req.id, "foo".into());
		assert_eq!(req.repository, Some("modrinth".into()));
		assert_eq!(req.content_version, VersionPattern::Single("1.19.2".into()));
		let req = PkgRequest::parse(":foo", PkgRequestSource::UserRequire);
		assert_eq!(req.id, "foo".into());
		assert_eq!(req.repository, None);

		let _ = PkgRequest::parse(":@", PkgRequestSource::UserRequire);
	}
}
