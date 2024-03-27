use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

/// Modrinth package generation
pub mod modrinth;
/// Smithed package generation
pub mod smithed;

/// Different types of package generation
#[derive(Copy, Clone, Debug, clap::ValueEnum)]
pub enum PackageSource {
	Smithed,
	Modrinth,
}

/// Configuration for generating the package from whatever source
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct PackageGenerationConfig {
	/// JSON structure to be merged with the output package
	pub merge: serde_json::Map<String, serde_json::Value>,
	/// Substitutions for relations
	pub relation_substitutions: HashMap<String, String>,
}

/// Generates a package from a source and config
pub async fn gen(source: PackageSource, config: Option<PackageGenerationConfig>, id: &str) {
	let config = config.unwrap_or_default();
	let pkg = match source {
		PackageSource::Smithed => smithed::gen(id, config.relation_substitutions).await,
		PackageSource::Modrinth => modrinth::gen(id, config.relation_substitutions).await,
	};

	// Merge with config
	let mut pkg = serde_json::value::to_value(pkg).expect("Failed to convert package to value");
	let merge = serde_json::value::to_value(config.merge)
		.expect("Failed to convert merged config to value");
	json_merge(&mut pkg, merge);

	println!(
		"{}",
		serde_json::to_string_pretty(&pkg).expect("Failed to format package")
	);
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
