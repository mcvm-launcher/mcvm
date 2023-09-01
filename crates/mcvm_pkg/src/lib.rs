#![warn(missing_docs)]

//! mcvm_pkg is a library for dealing with MCVM packages

/// Standard declarative package format
pub mod declarative;
/// Standard repository format
pub mod repo;
/// Standardized package dependency resolution
pub mod resolve;

use anyhow::Context;
use async_trait::async_trait;
use declarative::{deserialize_declarative_package, validate_declarative_package};
use parse::properties::PackageProperties;
use serde::{Deserialize, Serialize};

// Re-export
pub use mcvm_parse as parse;
pub use mcvm_shared::pkg::{PkgRequest, PkgRequestSource};

/// Content type of a package
#[derive(Deserialize, Serialize, Debug, Copy, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum PackageContentType {
	/// A package script
	#[default]
	Script,
	/// A declarative / JSON package
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
	/// Type passed to your evaluation functions
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
	/// Get the evaluated dependencies
	fn get_deps(&self) -> Vec<Vec<RequiredPackage>>;
	/// Get the evaluated conflicts
	fn get_conflicts(&self) -> Vec<String>;
	/// Get the evaluated recommendations
	fn get_recommendations(&self) -> Vec<RecommendedPackage>;
	/// Get the evaluated bundled packages
	fn get_bundled(&self) -> Vec<String>;
	/// Get the evaluated compats
	fn get_compats(&self) -> Vec<(String, String)>;
	/// Get the evaluated extensions
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

	/// Get the properties of a package
	async fn get_package_properties<'b>(
		&'b mut self,
		pkg: &PkgRequest,
		common_input: &Self::CommonInput,
	) -> anyhow::Result<&'b PackageProperties>;
}
