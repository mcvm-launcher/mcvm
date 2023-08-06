use std::collections::HashMap;

use mcvm_parse::{
	conditions::OSCondition, metadata::PackageMetadata, properties::PackageProperties,
};
use mcvm_shared::{
	instance::Side,
	lang::Language,
	modifications::{ModloaderMatch, PluginLoaderMatch},
	pkg::PackageStability,
	versions::VersionPattern, addon::AddonKind,
};
use serde::Deserialize;

/// Package relationships for declarative packages
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct DeclarativePackageRelations {
	pub dependencies: Vec<String>,
	pub explicit_dependencies: Vec<String>,
	pub conflicts: Vec<String>,
	pub extensions: Vec<String>,
	pub bundled: Vec<String>,
	pub compats: Vec<(String, String)>,
	pub recommendations: Vec<String>,
}

impl DeclarativePackageRelations {
	/// Merges this struct and another struct's relations
	pub fn merge(&mut self, other: Self) {
		self.dependencies.extend(other.dependencies);
		self.explicit_dependencies
			.extend(other.explicit_dependencies);
		self.conflicts.extend(other.conflicts);
		self.extensions.extend(other.extensions);
		self.bundled.extend(other.bundled);
		self.compats.extend(other.compats);
		self.recommendations.extend(other.recommendations);
	}
}

/// Properties that are used for choosing the best addon version from a declarative package
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct DeclarativeAddonVersionProperties {
	pub minecraft_versions: Option<Vec<VersionPattern>>,
	pub side: Option<Side>,
	pub modloaders: Option<Vec<ModloaderMatch>>,
	pub plugin_loaders: Option<Vec<PluginLoaderMatch>>,
	pub stability: Option<PackageStability>,
	pub features: Option<Vec<String>>,
	pub os: Option<OSCondition>,
	pub language: Option<Language>,
}

/// Properties for declarative addon versions that can be changed with patches
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct DeclarativeAddonVersionPatchProperties {
	pub relations: DeclarativePackageRelations,
	pub filename: String,
}

/// Properties that can be applied conditionally
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct DeclarativeConditionalProperties {
	pub relations: DeclarativePackageRelations,
}

/// Conditional rule to apply changes to a declarative package
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct DeclarativeConditionalRule {
	pub conditions: Vec<DeclarativeAddonVersionProperties>,
	pub properties: DeclarativeConditionalProperties,
}

/// Version for an addon in a declarative package
#[derive(Deserialize, Debug, Clone)]
pub struct DeclarativeAddonVersion {
	#[serde(flatten)]
	pub properties: DeclarativeAddonVersionProperties,
	#[serde(default)]
	pub relations: DeclarativePackageRelations,
	#[serde(default)]
	pub filename: Option<String>,
	#[serde(default)]
	pub path: Option<String>,
	#[serde(default)]
	pub url: Option<String>,
	#[serde(default)]
	pub version: Option<String>,
}

/// Addon in a declarative package
#[derive(Deserialize, Debug, Clone)]
pub struct DeclarativeAddon {
	pub kind: AddonKind,
	pub versions: Vec<DeclarativeAddonVersion>,
	#[serde(default)]
	pub conditions: Vec<DeclarativeAddonVersionProperties>,
}

/// Structure for a declarative / JSON package
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct DeclarativePackage {
	pub meta: PackageMetadata,
	pub properties: PackageProperties,
	pub addons: HashMap<String, DeclarativeAddon>,
	pub relations: DeclarativePackageRelations,
	pub conditional_rules: Vec<DeclarativeConditionalRule>,
}

/// Deserialize a declarative package
pub fn deserialize_declarative_package(text: &str) -> anyhow::Result<DeclarativePackage> {
	let out = serde_json::from_str(text)?;
	Ok(out)
}

#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn test_declarative_package_deser() {
		let contents = r#"
			{
				"meta": {
					"name": "Test Package",
					"long_description": "Blah blah blah"
				},
				"properties": {
					"modrinth_id": "2E4b7"
				},
				"addons": {
					"test": {
						"kind": "mod",
						"versions": [
							{
								"url": "example.com"
							}
						]
					}
				},
				"relations": {
					"compats": [[ "foo", "bar" ]]
				}
			}
		"#;

		let pkg = deserialize_declarative_package(contents).unwrap();

		assert_eq!(pkg.meta.name, Some(String::from("Test Package")));
	}
}
