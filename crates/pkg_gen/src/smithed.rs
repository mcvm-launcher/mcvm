use std::collections::HashMap;

use anyhow::Context;
use mcvm_core::net::download::Client;
use mcvm_pkg::declarative::{
	DeclarativeAddon, DeclarativeAddonVersion, DeclarativeConditionSet, DeclarativePackage,
	DeclarativePackageRelations,
};
use mcvm_pkg::metadata::PackageMetadata;
use mcvm_pkg::properties::PackageProperties;
use mcvm_shared::addon::AddonKind;
use mcvm_shared::util::DeserListOrSingle;
use mcvm_shared::versions::VersionPattern;

use mcvm_net::smithed::Pack;

use crate::relation_substitution::RelationSubMethod;

/// Generates a Smithed package from a Smithed pack ID
pub async fn gen_from_id(
	id: &str,
	body: Option<String>,
	relation_substitution: RelationSubMethod,
	force_extensions: &[String],
) -> anyhow::Result<DeclarativePackage> {
	let pack = mcvm_net::smithed::get_pack(id, &Client::new())
		.await
		.expect("Failed to get pack");

	gen(pack, body, relation_substitution, force_extensions)
}

/// Generates a Smithed package from a Smithed pack
pub fn gen(
	pack: Pack,
	body: Option<String>,
	relation_substitution: RelationSubMethod,
	force_extensions: &[String],
) -> anyhow::Result<DeclarativePackage> {
	let meta = PackageMetadata {
		name: Some(pack.display.name),
		description: Some(pack.display.description),
		long_description: body,
		icon: Some(pack.display.icon),
		website: pack.display.web_page,
		..Default::default()
	};

	let mut props = PackageProperties {
		smithed_id: Some(pack.id),
		tags: Some(vec!["datapack".into()]),
		..Default::default()
	};

	// Generate addons

	let mut datapack = DeclarativeAddon {
		kind: AddonKind::Datapack,
		versions: Vec::new(),
		conditions: Vec::new(),
		optional: false,
	};

	let mut resourcepack = DeclarativeAddon {
		kind: AddonKind::ResourcePack,
		versions: Vec::new(),
		conditions: Vec::new(),
		optional: false,
	};

	let mut all_mc_versions = Vec::new();

	for version in pack.versions.into_iter().rev() {
		// Get the sanitized version name
		let version_name_sanitized = version.name.replace('.', "-");
		let version_name = format!("smithed-version-{version_name_sanitized}");
		// Collect Minecraft versions
		let mc_versions: Vec<VersionPattern> = version
			.supports
			.iter()
			.map(|x| VersionPattern::Single(x.clone()))
			.collect();

		// Add to all Minecraft versions
		for version in mc_versions.clone() {
			if !all_mc_versions.contains(&version) {
				all_mc_versions.push(version);
			}
		}

		let mut deps = Vec::new();
		let mut extensions = Vec::new();

		for dep in version.dependencies {
			let dep = relation_substitution
				.substitute(&dep.id)
				.context("Failed to substitute dependency")?;
			if force_extensions.contains(&dep) {
				extensions.push(dep);
			} else {
				deps.push(dep);
			}
		}

		let mut pkg_version = DeclarativeAddonVersion {
			version: Some(version_name),
			conditional_properties: DeclarativeConditionSet {
				minecraft_versions: Some(DeserListOrSingle::List(mc_versions)),
				..Default::default()
			},
			relations: DeclarativePackageRelations {
				dependencies: DeserListOrSingle::List(deps),
				extensions: DeserListOrSingle::List(extensions),
				..Default::default()
			},
			..Default::default()
		};

		if let Some(url) = version.downloads.datapack {
			pkg_version.url = Some(url);
			datapack.versions.push(pkg_version.clone());
		}

		if let Some(url) = version.downloads.resourcepack {
			pkg_version.url = Some(url);
			resourcepack.versions.push(pkg_version.clone());
		}
	}

	props.supported_versions = Some(all_mc_versions);

	let mut addon_map = HashMap::new();
	addon_map.insert("datapack".into(), datapack);
	addon_map.insert("resourcepack".into(), resourcepack);

	Ok(DeclarativePackage {
		meta,
		properties: props,
		addons: addon_map,
		..Default::default()
	})
}
