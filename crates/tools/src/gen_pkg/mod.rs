use std::{collections::HashMap, io::stdout};

use serde::{Deserialize, Serialize};
use serde_json::{ser::PrettyFormatter, Serializer, Value};

/// Generation of many packages
pub mod batched;
/// Modrinth package generation
pub mod modrinth;
/// Smithed package generation
pub mod smithed;

/// Different types of package generation
#[derive(Copy, Clone, Debug, clap::ValueEnum, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PackageSource {
	Smithed,
	Modrinth,
}

/// Configuration for generating the package from whatever source
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct PackageGenerationConfig {
	/// JSON structure to be merged with the output package
	pub merge: serde_json::Value,
	/// Substitutions for relations
	pub relation_substitutions: HashMap<String, String>,
	/// Dependencies to force into extensions
	pub force_extensions: Vec<String>,
}

impl PackageGenerationConfig {
	/// Merge this config with another one to be placed over top of it
	#[must_use]
	pub fn merge(mut self, other: Self) -> Self {
		json_merge(&mut self.merge, other.merge);
		self.relation_substitutions
			.extend(other.relation_substitutions);
		self.force_extensions.extend(other.force_extensions);
		self
	}
}

/// Generates a package from a source and config
pub async fn gen(source: PackageSource, config: Option<PackageGenerationConfig>, id: &str) {
	let config = config.unwrap_or_default();
	let mut pkg = match source {
		PackageSource::Smithed => {
			smithed::gen(id, config.relation_substitutions, &config.force_extensions).await
		}
		PackageSource::Modrinth => {
			modrinth::gen(id, config.relation_substitutions, &config.force_extensions).await
		}
	};

	// Improve the generated package
	pkg.improve_generation();

	// Merge with config
	let mut pkg = serde_json::value::to_value(pkg).expect("Failed to convert package to value");
	json_merge(&mut pkg, config.merge);

	let mut serializer = Serializer::with_formatter(stdout(), PrettyFormatter::with_indent(b"\t"));
	pkg.serialize(&mut serializer)
		.expect("Failed to output package");
}

/// Utility function to merge serde_json values
fn json_merge(a: &mut Value, b: Value) {
	if let Value::Object(a) = a {
		if let Value::Object(b) = b {
			for (k, v) in b {
				if v.is_null() {
					a.remove(&k);
				} else {
					json_merge(a.entry(k).or_insert(Value::Null), v);
				}
			}

			return;
		}
	}

	*a = b;
}
