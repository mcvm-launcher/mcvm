use std::collections::HashMap;

use anyhow::Context;
use nitro_parse::conditions::{ArchCondition, OSCondition};
use nitro_shared::Side;
use nitro_shared::lang::Language;
use nitro_shared::loaders::LoaderMatch;
use nitro_shared::pkg::{AddonOptionalHashes, PackageKind, PackageStability};
use nitro_shared::util::DefaultExt;
use nitro_shared::util::DeserListOrSingle;
use nitro_shared::versions::VersionPattern;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::RecommendedPackage;
use crate::metadata::PackageMetadata;
use crate::properties::PackageProperties;

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
	/// Included packages
	#[serde(skip_serializing_if = "DeserListOrSingle::is_empty")]
	pub inclusions: DeserListOrSingle<String>,
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
		self.inclusions.merge(other.inclusions);
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
			&& self.inclusions.is_empty()
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
	/// What loaders to allow
	#[serde(skip_serializing_if = "DeserListOrSingle::is_option_empty")]
	pub loaders: Option<DeserListOrSingle<LoaderMatch>>,
	/// What stability setting to allow
	#[serde(skip_serializing_if = "Option::is_none")]
	pub stability: Option<PackageStability>,
	/// What features to allow
	#[serde(skip_serializing_if = "DeserListOrSingle::is_option_empty")]
	pub features: Option<DeserListOrSingle<String>>,
	/// What content versions to allow
	#[serde(skip_serializing_if = "DeserListOrSingle::is_option_empty")]
	pub content_versions: Option<DeserListOrSingle<String>>,
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
	pub kind: PackageKind,
	/// Modpack format if this addon is a modpack
	#[serde(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub modpack_format: Option<String>,
	/// The available versions of this addon
	#[serde(default)]
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
	#[serde(skip_serializing_if = "AddonOptionalHashes::is_empty")]
	pub hashes: AddonOptionalHashes,
	/// Whether this URL is actually a manual download page
	#[serde(default)]
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub is_manual: bool,
}

impl DeclarativeAddonVersion {
	/// Gets whether one of the given content versions matches this version's content versions or version ID
	pub fn content_versions_match(&self, content_versions: &[String]) -> bool {
		if let Some(id) = &self.version
			&& content_versions.contains(id)
		{
			return true;
		}

		if let Some(own_content_versions) = &self.conditional_properties.content_versions {
			own_content_versions
				.iter()
				.any(|x| content_versions.contains(x))
		} else {
			false
		}
	}
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
	// SAFETY: The modified, possibly invalid string is a copy that is never used again
	let out = unsafe {
		let mut text = text.to_string();
		let text = text.as_bytes_mut();
		simd_json::from_slice(text)?
	};
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
	/// Gets the real name of a content version given either an actual content version,
	/// or a version ID
	pub fn get_content_version_name<'a>(&'a self, version: &'a String) -> &'a String {
		if let Some(content_versions) = &self.properties.content_versions
			&& content_versions.contains(version)
		{
			return version;
		}

		for addon in self.addons.values() {
			for addon_version in &addon.versions {
				if addon_version.version.as_ref().is_none_or(|x| x != version) {
					continue;
				}

				let Some(content_versions) = &addon_version.conditional_properties.content_versions
				else {
					continue;
				};

				if let Some(content_version) = content_versions.first() {
					return content_version;
				}
			}
		}

		version
	}

	/// Improve a generated package by inferring certain fields
	pub fn improve_generation(&mut self) {
		// Infer issues link from a GitHub source link
		if self.meta.issues.is_none()
			&& let Some(source) = &self.meta.source
			&& source.contains("://github.com/")
		{
			let issues = source.clone();
			let issues = issues.trim_end_matches('/');
			self.meta.issues = Some(issues.to_string() + "issues");
		}
	}

	/// Optimize the package by removing redundancies. This might break some packages
	/// so it is recommended to use it only on simple generated ones
	pub fn optimize(&mut self) {
		// Move common relations in every version of an addon to the package scope
		for addon in self.addons.values_mut() {
			if addon.versions.is_empty() {
				return;
			}
			let mut first = None;
			let mut all_same = true;
			for version in &addon.versions {
				if let Some(first) = first {
					if &version.relations.dependencies != first {
						all_same = false;
						break;
					}
				} else {
					first = Some(&version.relations.dependencies);
				}
			}

			if all_same && addon.conditions.is_empty() && !addon.optional {
				self.relations
					.dependencies
					.extend(first.expect("Length of versions is > 0").iter().cloned());

				for version in &mut addon.versions {
					version.relations.dependencies = DeserListOrSingle::List(Vec::new());
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

	#[test]
	fn test_declarative_content_version_name() {
		let mut addons = HashMap::new();
		addons.insert(
			"foo".into(),
			DeclarativeAddon {
				kind: PackageKind::Mod,
				modpack_format: None,
				versions: vec![DeclarativeAddonVersion {
					version: Some("a".into()),
					conditional_properties: DeclarativeConditionSet {
						content_versions: Some(DeserListOrSingle::Single("1".into())),
						..Default::default()
					},
					..Default::default()
				}],
				conditions: Vec::new(),
				optional: false,
			},
		);

		let pkg = DeclarativePackage {
			properties: PackageProperties {
				content_versions: Some(vec!["1".into(), "2".into()]),
				..Default::default()
			},
			addons,
			..Default::default()
		};

		assert_eq!(pkg.get_content_version_name(&"1".into()), "1");
		assert_eq!(pkg.get_content_version_name(&"2".into()), "2");
		assert_eq!(pkg.get_content_version_name(&"a".into()), "1");
	}
}
