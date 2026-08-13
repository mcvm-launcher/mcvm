use std::collections::HashSet;

use anyhow::{Context, anyhow};
use nitro_net::curseforge::{
	CurseFile, CurseGameVersion, CurseMod, parse_class_id, parse_release_type,
};
use nitro_pkg::{
	declarative::{
		DeclarativeAddon, DeclarativeAddonVersion, DeclarativeConditionSet, DeclarativePackage,
		DeclarativePackageRelations,
	},
	metadata::PackageMetadata,
	properties::PackageProperties,
};
use nitro_shared::{
	Side,
	loaders::{Loader, LoaderMatch},
	pkg::PackageKind,
	util::DeserListOrSingle,
	versions::VersionPattern,
};

/// Generates a CurseForge package from a CurseForge project
pub async fn generate(
	m: CurseMod,
	body: Option<String>,
	files: Vec<CurseFile>,
	repository: Option<&str>,
) -> anyhow::Result<DeclarativePackage> {
	let mut meta = PackageMetadata {
		slug: Some(m.slug),
		name: Some(m.name),
		icon: m.logo.map(|x| x.url),
		description: Some(m.summary),
		downloads: Some(m.download_count),
		long_description: body,
		website: m.links.website_url,
		issues: m.links.issues_url,
		source: m.links.source_url,
		documentation: m.links.wiki_url,
		..Default::default()
	};
	meta.authors = Some(m.authors.into_iter().map(|x| x.name).collect());
	meta.gallery = Some(m.screenshots.into_iter().map(|x| x.url).collect());

	let mut props = PackageProperties {
		curseforge_id: Some(m.id.to_string()),
		..Default::default()
	};

	let pkg_ty = parse_class_id(m.class_id).context("Unsupported CurseForge project type")?;
	let modpack_format = if pkg_ty == PackageKind::Modpack {
		Some("cfpack".into())
	} else {
		None
	};
	props.kinds = vec![pkg_ty];

	let mut addon = DeclarativeAddon {
		kind: pkg_ty,
		modpack_format,
		versions: Vec::new(),
		conditions: Vec::new(),
		optional: false,
	};

	let mut content_versions = Vec::with_capacity(files.len());
	let mut all_sides = HashSet::new();
	let mut all_loaders = HashSet::new();
	let mut all_mc_versions = HashSet::new();

	for file in files {
		let mut sides = HashSet::new();
		let mut loaders = HashSet::new();
		let mut mc_versions = HashSet::new();

		let mut skip = false;
		for v in file.game_versions {
			match v {
				CurseGameVersion::Client => {
					sides.insert(Side::Client);
				}
				CurseGameVersion::Server => {
					sides.insert(Side::Server);
					sides.insert(Side::Client);
				}
				CurseGameVersion::Forge | CurseGameVersion::NeoForge => {
					loaders.insert(LoaderMatch::ForgeLike);
				}
				CurseGameVersion::Fabric | CurseGameVersion::Quilt => {
					loaders.insert(LoaderMatch::FabricLike);
				}
				CurseGameVersion::LiteLoader => {
					loaders.insert(LoaderMatch::Loader(Loader::LiteLoader));
				}
				CurseGameVersion::Minecraft(version) => {
					mc_versions.insert(VersionPattern::Single(version));
				}
				CurseGameVersion::Cauldron => {
					skip = true;
				}
			}
		}
		if skip {
			continue;
		}

		all_sides.extend(sides.clone());
		all_loaders.extend(loaders.clone());
		all_mc_versions.extend(mc_versions.clone());

		let content_version = file.display_name.clone();
		let content_version = cleanup_version_name(&content_version);
		content_versions.push(content_version.clone());

		let stability = parse_release_type(file.release_type);

		let mut deps = Vec::new();
		let mut conflicts = Vec::new();
		let mut inclusions = Vec::new();
		for dep in file.dependencies {
			let pkg = if let Some(repo) = repository {
				format!("{repo}:{}", dep.mod_id)
			} else {
				dep.mod_id.to_string()
			};

			match dep.relation_type {
				1 | 6 => inclusions.push(pkg),
				3 => deps.push(pkg),
				5 => conflicts.push(pkg),
				_ => {}
			}
		}

		let pkg_version = DeclarativeAddonVersion {
			version: Some(file.id.to_string()),
			url: Some(file.download_url),
			filename: Some(file.file_name),
			conditional_properties: DeclarativeConditionSet {
				minecraft_versions: Some(DeserListOrSingle::List(
					mc_versions.into_iter().collect(),
				)),
				loaders: Some(DeserListOrSingle::List(loaders.into_iter().collect())),
				stability: Some(stability),
				content_versions: Some(DeserListOrSingle::Single(content_version)),
				..Default::default()
			},
			relations: DeclarativePackageRelations {
				dependencies: DeserListOrSingle::List(deps),
				conflicts: DeserListOrSingle::List(conflicts),
				inclusions: DeserListOrSingle::List(inclusions),
				..Default::default()
			},
			..Default::default()
		};

		addon.versions.push(pkg_version);
	}

	Err(anyhow!("Die"))
}

/// Cleanup a version name to remove things like loaders
pub fn cleanup_version_name(version: &str) -> String {
	version.replace("+", "-")
}
