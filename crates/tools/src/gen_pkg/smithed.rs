use std::collections::HashMap;

use mcvm::pkg_crate::declarative::{
	DeclarativeAddon, DeclarativeAddonVersion, DeclarativeConditionSet, DeclarativePackage,
	DeclarativePackageRelations,
};
use mcvm::pkg_crate::metadata::PackageMetadata;
use mcvm::pkg_crate::properties::PackageProperties;
use mcvm::shared::addon::AddonKind;
use mcvm::shared::util::DeserListOrSingle;
use mcvm::shared::versions::VersionPattern;
use reqwest::Client;

use crate::smithed_api::{self, Pack};

pub async fn gen(
	id: &str,
	relation_substitutions: HashMap<String, String>,
	force_extensions: &[String],
) -> DeclarativePackage {
	let pack = smithed_api::get_pack(id, &Client::new())
		.await
		.expect("Failed to get pack");

	gen_raw(pack, relation_substitutions, force_extensions).await
}

pub async fn gen_raw(
	pack: Pack,
	relation_substitutions: HashMap<String, String>,
	force_extensions: &[String],
) -> DeclarativePackage {
	let meta = PackageMetadata {
		name: Some(pack.display.name),
		description: Some(pack.display.description),
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
	};

	let mut resourcepack = DeclarativeAddon {
		kind: AddonKind::ResourcePack,
		versions: Vec::new(),
		conditions: Vec::new(),
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
			if let Some(dep_id) = relation_substitutions.get(&dep.id) {
				if force_extensions.contains(dep_id) {
					extensions.push(dep_id.clone());
				} else {
					deps.push(dep_id.clone());
				}
			} else {
				panic!("Dependency {} was not substituted", dep.id);
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

	let pkg = DeclarativePackage {
		meta,
		properties: props,
		addons: addon_map,
		..Default::default()
	};

	pkg
}
