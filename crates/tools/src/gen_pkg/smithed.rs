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

use crate::smithed_api;

pub async fn gen(id: &str, dep_substitutions: Option<Vec<String>>) {
	let mut dep_subs = HashMap::new();
	if let Some(dep_substitutions) = dep_substitutions {
		for dep in dep_substitutions {
			let mut items = dep.split('=');
			let key = items.next().expect("Key in dep sub is missing");
			let val = items.next().expect("Val in dep sub is missing");
			if key.is_empty() {
				panic!("Dep sub key is empty");
			}
			if val.is_empty() {
				panic!("Dep sub value is empty");
			}
			dep_subs.insert(key.to_string(), val.to_string());
		}
	}

	let pack = smithed_api::get_pack(id).await.expect("Failed to get pack");

	let meta = PackageMetadata {
		name: Some(pack.display.name),
		description: Some(pack.display.description),
		icon: Some(pack.display.icon),
		website: pack.display.web_page,
		..Default::default()
	};

	let props = PackageProperties {
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

	for version in pack.versions {
		let version_name_sanitized = version.name.replace('.', "-");
		let version_name = format!("smithed-version-{version_name_sanitized}");
		let mc_versions: Vec<VersionPattern> = version
			.supports
			.iter()
			.map(|x| VersionPattern::Single(x.clone()))
			.collect();

		let deps: Vec<String> = version
			.dependencies
			.iter()
			.map(|dep| {
				if let Some(dep_id) = dep_subs.get(&dep.id) {
					dep_id.clone()
				} else {
					panic!("Dependency {} was not substituted", dep.id)
				}
			})
			.collect();

		let mut pkg_version = DeclarativeAddonVersion {
			version: Some(version_name),
			conditional_properties: DeclarativeConditionSet {
				minecraft_versions: Some(DeserListOrSingle::List(mc_versions)),
				..Default::default()
			},
			relations: DeclarativePackageRelations {
				dependencies: DeserListOrSingle::List(deps),
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

	let mut addon_map = HashMap::new();
	addon_map.insert("datapack".into(), datapack);
	addon_map.insert("resourcepack".into(), resourcepack);

	let pkg = DeclarativePackage {
		meta,
		properties: props,
		addons: addon_map,
		..Default::default()
	};

	println!(
		"{}",
		serde_json::to_string_pretty(&pkg).expect("Failed to format package")
	);
}
