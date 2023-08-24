pub mod declarative;
pub mod repo;
pub mod resolve;

use std::{fmt::Display, hash::Hash};

use anyhow::Context;
use async_trait::async_trait;
use declarative::{deserialize_declarative_package, validate_declarative_package};
// Re-export
pub use mcvm_parse as parse;
use parse::properties::PackageProperties;
use serde::{Deserialize, Serialize};

/// Content type of a package
#[derive(Deserialize, Serialize, Debug, Copy, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum PackageContentType {
	#[default]
	Script,
	Declarative,
}

/// Parses and validates a package
pub fn parse_and_validate(contents: &str, content_type: PackageContentType) -> anyhow::Result<()> {
	match content_type {
		PackageContentType::Script => {
			let parsed = parse::parse::lex_and_parse(contents).context("Parsing failed")?;
			parse::metadata::eval_metadata(&parsed).context("Metadata evaluation failed")?;
			parse::properties::eval_properties(&parsed).context("Properties evaluation failed")?;
		}
		PackageContentType::Declarative => {
			let contents = deserialize_declarative_package(contents).context("Parsing failed")?;
			validate_declarative_package(&contents).context("Package was invalid")?;
		}
	}

	Ok(())
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
	pub name: String,
}

impl PkgRequest {
	pub fn new(name: &str, source: PkgRequestSource) -> Self {
		Self {
			name: name.to_owned(),
			source,
		}
	}

	/// Create a dependency list for debugging. Recursive, so call with an empty string
	pub fn debug_sources(&self, list: String) -> String {
		match &self.source {
			PkgRequestSource::UserRequire => format!("{}{list}", self.name),
			PkgRequestSource::Dependency(source) => {
				format!("{} -> {}", source.debug_sources(list), self.name)
			}
			PkgRequestSource::Refused(source) => {
				format!("{} =X=> {}", source.debug_sources(list), self.name)
			}
			PkgRequestSource::Bundled(bundler) => {
				format!("{} => {}", bundler.debug_sources(list), self.name)
			}
			PkgRequestSource::Repository => format!("Repository -> {}{list}", self.name),
		}
	}
}

impl PartialEq for PkgRequest {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name
	}
}

impl Eq for PkgRequest {}

impl Hash for PkgRequest {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.name.hash(state);
	}
}

impl Display for PkgRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.name)
	}
}

/// A required package
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct RequiredPackage {
	/// The package id that is required
	pub value: String,
	/// Whether this is an explicit dependency
	pub explicit: bool,
}

/// A recommended package
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Deserialize)]
pub struct RecommendedPackage {
	/// The package id that is required
	pub value: String,
	/// Whether to invert this recommendation
	pub invert: bool,
}

/// Trait for a user-configured package
pub trait ConfiguredPackage: Clone {
	type EvalInput<'a>: Clone;

	/// Get the package ID
	fn get_package(&self) -> &PkgRequest;

	/// Override the EvalInput for this package based on configuration
	fn override_configured_package_input(
		&self,
		properties: &PackageProperties,
		input: &mut Self::EvalInput<'_>,
	) -> anyhow::Result<()>;
}

/// Trait for the result from evaluating a package, used for resolution
pub trait PackageEvalRelationsResult {
	fn get_deps(&self) -> Vec<Vec<RequiredPackage>>;
	fn get_conflicts(&self) -> Vec<String>;
	fn get_recommendations(&self) -> Vec<RecommendedPackage>;
	fn get_bundled(&self) -> Vec<String>;
	fn get_compats(&self) -> Vec<(String, String)>;
	fn get_extensions(&self) -> Vec<String>;
}

/// Trait for a central package registry that can evaluate packages
#[async_trait]
pub trait PackageEvaluator<'a> {
	/// Type passed to most functions, used for common / cached values
	type CommonInput;
	/// Type passed to the evaluation function
	type EvalInput<'b>: Clone;
	/// Result from package relationship evaluation
	type EvalRelationsResult<'b>: PackageEvalRelationsResult;
	/// Configured package type
	type ConfiguredPackage: ConfiguredPackage<EvalInput<'a> = Self::EvalInput<'a>>;

	/// Evaluate the relationships of a package
	async fn eval_package_relations(
		&mut self,
		pkg: &PkgRequest,
		input: &Self::EvalInput<'a>,
		common_input: &Self::CommonInput,
	) -> anyhow::Result<Self::EvalRelationsResult<'a>>;

	async fn get_package_properties<'b>(
		&'b mut self,
		pkg: &PkgRequest,
		common_input: &Self::CommonInput,
	) -> anyhow::Result<&'b PackageProperties>;
}
