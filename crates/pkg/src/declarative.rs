use std::collections::HashMap;

use anyhow::Context;
use mcvm_parse::conditions::{ArchCondition, OSCondition};
use mcvm_shared::addon::AddonKind;
use mcvm_shared::lang::Language;
use mcvm_shared::modifications::{ModloaderMatch, PluginLoaderMatch};
use mcvm_shared::pkg::{PackageAddonOptionalHashes, PackageStability};
use mcvm_shared::util::DeserListOrSingle;
use mcvm_shared::versions::VersionPattern;
use mcvm_shared::Side;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::metadata::PackageMetadata;
use crate::properties::PackageProperties;
use crate::RecommendedPackage;

/// Structure for a declarative / JSON package
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct DeclarativePackage {
	/// Metadata for the package
	#[serde(skip_serializing_if = "PackageMetadata::is_empty")]
	pub meta: PackageMetadata,
	/// Properties for the package
	#[serde(skip_serializing_if = "PackageProperties::is_empty")]
	pub properties: PackageProperties,
	/// Addons that the package installs
	#[serde(skip_serializing_if = "HashMap::is_empty")]
	pub addons: HashMap<String, DeclarativeAddon>,
	/// Relationships with other packages
	#[serde(skip_serializing_if = "DeclarativePackageRelations::is_empty")]
	pub relations: DeclarativePackageRelations,
	/// Changes to conditionally apply to the package
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub conditional_rules: Vec<DeclarativeConditionalRule>,
}

/// Package relationships for declarative packages
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct DeclarativePackageRelations {
	/// Package dependencies
	#[serde(skip_serializing_if = "DeserListOrSingle::is_empty")]
	pub dependencies: DeserListOrSingle<String>,
	/// Explicit dependencies
	#[serde(skip_serializing_if = "DeserListOrSingle::is_empty")]
	pub explicit_dependencies: DeserListOrSingle<String>,
	/// Package conflicts
	#[serde(skip_serializing_if = "DeserListOrSingle::is_empty")]
	pub conflicts: DeserListOrSingle<String>,
	/// Package extensions
	#[serde(skip_serializing_if = "DeserListOrSingle::is_empty")]
	pub extensions: DeserListOrSingle<String>,
	/// Bundled packages
	#[serde(skip_serializing_if = "DeserListOrSingle::is_empty")]
	pub bundled: DeserListOrSingle<String>,
	/// Package compats
	#[serde(skip_serializing_if = "DeserListOrSingle::is_empty")]
	pub compats: DeserListOrSingle<(String, String)>,
	/// Package recommendations
	#[serde(skip_serializing_if = "DeserListOrSingle::is_empty")]
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

	/// Checks if the relations are empty
	pub fn is_empty(&self) -> bool {
		self.dependencies.is_empty()
			&& self.explicit_dependencies.is_empty()
			&& self.conflicts.is_empty()
			&& self.extensions.is_empty()
			&& self.bundled.is_empty()
			&& self.compats.is_empty()
			&& self.recommendations.is_empty()
	}
}

/// Properties that are used for choosing the best addon version
/// from a declarative package and conditional rules
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct DeclarativeConditionSet {
	/// Minecraft versions to allow
	#[serde(skip_serializing_if = "DeserListOrSingle::is_option_empty")]
	pub minecraft_versions: Option<DeserListOrSingle<VersionPattern>>,
	/// What side to allow
	#[serde(skip_serializing_if = "Option::is_none")]
	pub side: Option<Side>,
	/// What modloaders to allow
	#[serde(skip_serializing_if = "DeserListOrSingle::is_option_empty")]
	pub modloaders: Option<DeserListOrSingle<ModloaderMatch>>,
	/// What plugin loaders to allow
	#[serde(skip_serializing_if = "DeserListOrSingle::is_option_empty")]
	pub plugin_loaders: Option<DeserListOrSingle<PluginLoaderMatch>>,
	/// What stability setting to allow
	#[serde(skip_serializing_if = "Option::is_none")]
	pub stability: Option<PackageStability>,
	/// What features to allow
	#[serde(skip_serializing_if = "DeserListOrSingle::is_option_empty")]
	pub features: Option<DeserListOrSingle<String>>,
	/// What operating systems to allow
	#[serde(skip_serializing_if = "DeserListOrSingle::is_option_empty")]
	pub operating_systems: Option<DeserListOrSingle<OSCondition>>,
	/// What system architectures to allow
	#[serde(skip_serializing_if = "DeserListOrSingle::is_option_empty")]
	pub architectures: Option<DeserListOrSingle<ArchCondition>>,
	/// What languages to allow
	#[serde(skip_serializing_if = "DeserListOrSingle::is_option_empty")]
	pub languages: Option<DeserListOrSingle<Language>>,
}

/// Conditional rule to apply changes to a declarative package
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct DeclarativeConditionalRule {
	/// Conditions for this rule
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub conditions: Vec<DeclarativeConditionSet>,
	/// Properties to apply if this rule succeeds
	pub properties: DeclarativeConditionalRuleProperties,
}

/// Properties that can be applied conditionally
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct DeclarativeConditionalRuleProperties {
	/// Relations to append
	#[serde(skip_serializing_if = "DeclarativePackageRelations::is_empty")]
	pub relations: DeclarativePackageRelations,
	/// Notices to raise
	#[serde(skip_serializing_if = "DeserListOrSingle::is_empty")]
	pub notices: DeserListOrSingle<String>,
}

/// Addon in a declarative package
#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct DeclarativeAddon {
	/// What kind of addon this is
	pub kind: AddonKind,
	/// The available versions of this addon
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub versions: Vec<DeclarativeAddonVersion>,
	/// Conditions for this addon to be considered
	#[serde(default)]
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub conditions: Vec<DeclarativeConditionSet>,
	/// Whether this addon should be considered optional and not throw an error if it
	/// does not match any versions
	#[serde(default)]
	#[serde(skip_serializing_if = "is_false")]
	pub optional: bool,
}

fn is_false(v: &bool) -> bool {
	!v
}

/// Version for an addon in a declarative package
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct DeclarativeAddonVersion {
	/// Conditional properties for this version
	#[serde(flatten)]
	pub conditional_properties: DeclarativeConditionSet,
	/// Additional relations that this version imposes
	#[serde(skip_serializing_if = "DeclarativePackageRelations::is_empty")]
	pub relations: DeclarativePackageRelations,
	/// Notices that this version raises
	#[serde(skip_serializing_if = "DeserListOrSingle::is_empty")]
	pub notices: DeserListOrSingle<String>,
	/// Filename for the addon file
	#[serde(skip_serializing_if = "Option::is_none")]
	pub filename: Option<String>,
	/// Path to the version file
	#[serde(skip_serializing_if = "Option::is_none")]
	pub path: Option<String>,
	/// URL to the version file
	#[serde(skip_serializing_if = "Option::is_none")]
	pub url: Option<String>,
	/// Version identifier for this version
	#[serde(skip_serializing_if = "Option::is_none")]
	pub version: Option<String>,
	/// Hashes for this version file
	#[serde(skip_serializing_if = "PackageAddonOptionalHashes::is_empty")]
	pub hashes: PackageAddonOptionalHashes,
}

/// Properties for declarative addon versions that can be changed with patches
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct DeclarativeAddonVersionPatchProperties {
	/// Relations to append
	#[serde(skip_serializing_if = "DeclarativePackageRelations::is_empty")]
	pub relations: DeclarativePackageRelations,
	/// A filename to change
	pub filename: Option<String>,
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

impl DeclarativePackage {
	/// Improve a generated package by inferring certain fields
	pub fn improve_generation(&mut self) {
		// Infer issues link from a GitHub source link
		if self.meta.issues.is_none() {
			if let Some(source) = &self.meta.source {
				if source.contains("://github.com/") {
					let issues = source.clone();
					let issues = issues.trim_end_matches('/');
					self.meta.issues = Some(issues.to_string() + "issues");
				}
			}
		}
	}
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

		assert_eq!(pkg.meta.name, Some("Test Package".into()));
	}
}
