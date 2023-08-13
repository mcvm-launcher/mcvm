use std::collections::HashMap;

use anyhow::Context;
use mcvm_parse::{
	conditions::OSCondition, metadata::PackageMetadata, properties::PackageProperties,
};
use mcvm_shared::{
	addon::AddonKind,
	instance::Side,
	lang::Language,
	modifications::{ModloaderMatch, PluginLoaderMatch},
	pkg::PackageStability,
	util::DeserListOrSingle,
	versions::VersionPattern,
};
use serde::Deserialize;

/// Package relationships for declarative packages
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct DeclarativePackageRelations {
	pub dependencies: DeserListOrSingle<String>,
	pub explicit_dependencies: DeserListOrSingle<String>,
	pub conflicts: DeserListOrSingle<String>,
	pub extensions: DeserListOrSingle<String>,
	pub bundled: DeserListOrSingle<String>,
	pub compats: DeserListOrSingle<(String, String)>,
	pub recommendations: DeserListOrSingle<String>,
}

impl DeclarativePackageRelations {
	/// Merges this struct and another struct's relations
	pub fn merge(&mut self, other: Self) {
		self.dependencies.merge(other.dependencies);
		self.explicit_dependencies
			.merge(other.explicit_dependencies);
		self.conflicts.merge(other.conflicts);
		self.extensions.merge(other.extensions);
		self.bundled.merge(other.bundled);
		self.compats.merge(other.compats);
		self.recommendations.merge(other.recommendations);
	}
}

/// Properties that are used for choosing the best addon version
/// from a declarative package and conditional rules
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct DeclarativeConditionSet {
	pub minecraft_versions: Option<DeserListOrSingle<VersionPattern>>,
	pub side: Option<Side>,
	pub modloaders: Option<DeserListOrSingle<ModloaderMatch>>,
	pub plugin_loaders: Option<DeserListOrSingle<PluginLoaderMatch>>,
	pub stability: Option<PackageStability>,
	pub features: Option<DeserListOrSingle<String>>,
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
pub struct DeclarativeConditionalRuleProperties {
	pub relations: DeclarativePackageRelations,
	pub notices: DeserListOrSingle<String>,
}

/// Conditional rule to apply changes to a declarative package
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct DeclarativeConditionalRule {
	pub conditions: Vec<DeclarativeConditionSet>,
	pub properties: DeclarativeConditionalRuleProperties,
}

/// Version for an addon in a declarative package
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct DeclarativeAddonVersion {
	#[serde(flatten)]
	pub conditional_properties: DeclarativeConditionSet,
	pub relations: DeclarativePackageRelations,
	pub notices: DeserListOrSingle<String>,
	pub filename: Option<String>,
	pub path: Option<String>,
	pub url: Option<String>,
	pub version: Option<String>,
}

/// Addon in a declarative package
#[derive(Deserialize, Debug, Clone)]
pub struct DeclarativeAddon {
	pub kind: AddonKind,
	pub versions: Vec<DeclarativeAddonVersion>,
	#[serde(default)]
	pub conditions: Vec<DeclarativeConditionSet>,
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

/// Validate a declarative package
pub fn validate_declarative_package(pkg: &DeclarativePackage) -> anyhow::Result<()> {
	pkg.meta.check_validity().context("Metadata was invalid")?;
	pkg.properties
		.check_validity()
		.context("Properties were invalid")?;

	Ok(())
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
