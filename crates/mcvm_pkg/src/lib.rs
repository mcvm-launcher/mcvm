pub mod declarative;
pub mod repo;

use anyhow::Context;
use declarative::{deserialize_declarative_package, validate_declarative_package};
// Re-export
pub use mcvm_parse as parse;
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
