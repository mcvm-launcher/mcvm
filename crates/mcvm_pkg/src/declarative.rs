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
	pkg::{PackageAddonOptionalHashes, PackageStability},
	util::DeserListOrSingle,
	versions::VersionPattern,
};
use serde::Deserialize;

use crate::RecommendedPackage;

/// Package relationships for declarative packages
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct DeclarativePackageRelations {
	/// Package dependencies
	pub dependencies: DeserListOrSingle<String>,
	/// Explicit dependencies
	pub explicit_dependencies: DeserListOrSingle<String>,
	/// Package conflicts
	pub conflicts: DeserListOrSingle<String>,
	/// Package extensions
	pub extensions: DeserListOrSingle<String>,
	/// Bundled packages
	pub bundled: DeserListOrSingle<String>,
	/// Package compats
	pub compats: DeserListOrSingle<(String, String)>,
	/// Package recommendations
	pub recommendations: DeserListOrSingle<RecommendedPackage>,
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
	/// Minecraft versions to allow
	pub minecraft_versions: Option<DeserListOrSingle<VersionPattern>>,
	/// What side to allow
	pub side: Option<Side>,
	/// What modloaders to allow
	pub modloaders: Option<DeserListOrSingle<ModloaderMatch>>,
	/// What plugin loaders to allow
	pub plugin_loaders: Option<DeserListOrSingle<PluginLoaderMatch>>,
	/// What stability setting to allow
	pub stability: Option<PackageStability>,
	/// What features to allow
	pub features: Option<DeserListOrSingle<String>>,
	/// What operating system to allow
	pub os: Option<OSCondition>,
	/// What language to allow
	pub language: Option<Language>,
}

/// Properties for declarative addon versions that can be changed with patches
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct DeclarativeAddonVersionPatchProperties {
	/// Relations to append
	pub relations: DeclarativePackageRelations,
	// TODO: This should be an option
	/// A filename to change
	pub filename: String,
}

/// Properties that can be applied conditionally
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct DeclarativeConditionalRuleProperties {
	/// Relations to append
	pub relations: DeclarativePackageRelations,
	/// Notices to raise
	pub notices: DeserListOrSingle<String>,
}

/// Conditional rule to apply changes to a declarative package
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct DeclarativeConditionalRule {
	/// Conditions for this rule
	pub conditions: Vec<DeclarativeConditionSet>,
	/// Properties to apply if this rule succeeds
	pub properties: DeclarativeConditionalRuleProperties,
}

/// Version for an addon in a declarative package
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct DeclarativeAddonVersion {
	/// Conditional properties for this version
	#[serde(flatten)]
	pub conditional_properties: DeclarativeConditionSet,
	/// Additional relations that this version imposes
	pub relations: DeclarativePackageRelations,
	/// Notices that this version raises
	pub notices: DeserListOrSingle<String>,
	/// Filename for the addon file
	pub filename: Option<String>,
	/// Path to the version file
	pub path: Option<String>,
	/// URL to the version file
	pub url: Option<String>,
	/// Version identifier for this version
	pub version: Option<String>,
	/// Hashes for this version file
	pub hashes: PackageAddonOptionalHashes,
}

/// Addon in a declarative package
#[derive(Deserialize, Debug, Clone)]
pub struct DeclarativeAddon {
	/// What kind of addon this is
	pub kind: AddonKind,
	/// The available versions of this addon
	pub versions: Vec<DeclarativeAddonVersion>,
	/// Conditions for this addon to be considered
	#[serde(default)]
	pub conditions: Vec<DeclarativeConditionSet>,
}

/// Structure for a declarative / JSON package
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct DeclarativePackage {
	/// Metadata for the package
	pub meta: PackageMetadata,
	/// Properties for the package
	pub properties: PackageProperties,
	/// Addons that the package installs
	pub addons: HashMap<String, DeclarativeAddon>,
	/// Relationships with other packages
	pub relations: DeclarativePackageRelations,
	/// Changes to conditionally apply to the package
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
