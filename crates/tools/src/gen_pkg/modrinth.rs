use std::collections::HashMap;

use mcvm::pkg_crate::declarative::{
	DeclarativeAddon, DeclarativeAddonVersion, DeclarativeConditionSet, DeclarativePackage,
	DeclarativePackageRelations,
};
use mcvm::pkg_crate::metadata::PackageMetadata;
use mcvm::pkg_crate::properties::PackageProperties;
use mcvm::pkg_crate::RecommendedPackage;
use mcvm::shared::addon::AddonKind;
use mcvm::shared::modifications::{ModloaderMatch, PluginLoaderMatch};
use mcvm::shared::util::DeserListOrSingle;
use mcvm::shared::versions::VersionPattern;

use mcvm::net::modrinth::{self, DependencyType, KnownLoader, Loader, ProjectType};

pub async fn gen(
	id: &str,
	relation_substitutions: HashMap<String, String>,
	force_extensions: &[String],
) -> DeclarativePackage {
	let client = reqwest::Client::new();
	let project = modrinth::get_project(id, &client)
		.await
		.expect("Failed to get Modrinth project");

	let mut meta = PackageMetadata {
		name: Some(project.title),
		description: Some(project.description),
		icon: Some(project.icon_url),
		..Default::default()
	};
	if let Some(issues_url) = project.issues_url {
		meta.issues = Some(issues_url);
	}
	if let Some(source_url) = project.source_url {
		meta.source = Some(source_url);
	}
	if let Some(wiki_url) = project.wiki_url {
		meta.documentation = Some(wiki_url);
	}
	if let Some(discord_url) = project.discord_url {
		meta.community = Some(discord_url);
	}
	if let Some(support_link) = project.donation_urls.first() {
		meta.support_link = Some(support_link.url.clone());
	}

	meta.license = Some(project.license.id);

	// Get team members and use them to fill out the authors field
	let mut members = modrinth::get_project_team(id, &client)
		.await
		.expect("Failed to get project team members from Modrinth");
	members.sort_by_key(|x| x.ordering);
	meta.authors = Some(members.into_iter().map(|x| x.user.username).collect());

	let mut props = PackageProperties {
		modrinth_id: Some(project.id),
		..Default::default()
	};

	// Generate addons
	let addon_kind = match project.project_type {
		ProjectType::Mod => AddonKind::Mod,
		ProjectType::Datapack => AddonKind::Datapack,
		ProjectType::Plugin => AddonKind::Plugin,
		ProjectType::ResourcePack => AddonKind::ResourcePack,
		ProjectType::Shader => AddonKind::Shader,
		ProjectType::Modpack => panic!("Modpack projects are unsupported"),
	};
	let mut addon = DeclarativeAddon {
		kind: addon_kind,
		versions: Vec::new(),
		conditions: Vec::new(),
	};

	let mut all_mc_versions = Vec::new();

	let versions = modrinth::get_multiple_versions(&project.versions, &client)
		.await
		.expect("Failed to get Modrinth project versions");

	for version in versions {
		let version_name = version.id.clone();
		// Collect Minecraft versions
		let mc_versions: Vec<VersionPattern> = version
			.game_versions
			.iter()
			.map(|x| VersionPattern::Single(x.clone()))
			.collect();

		// Add to all Minecraft versions
		for version in mc_versions.clone() {
			if !all_mc_versions.contains(&version) {
				all_mc_versions.push(version);
			}
		}

		// Look at loaders
		let mut modloaders = Vec::new();
		let mut plugin_loaders = Vec::new();
		for loader in &version.loaders {
			match loader {
				Loader::Known(loader) => match loader {
					KnownLoader::Fabric => modloaders.push(ModloaderMatch::Fabric),
					KnownLoader::Quilt => modloaders.push(ModloaderMatch::Quilt),
					KnownLoader::Forge => modloaders.push(ModloaderMatch::Forge),
					KnownLoader::NeoForged => modloaders.push(ModloaderMatch::NeoForged),
					KnownLoader::Bukkit => plugin_loaders.push(PluginLoaderMatch::Bukkit),
					KnownLoader::Folia => plugin_loaders.push(PluginLoaderMatch::Folia),
					KnownLoader::Spigot => plugin_loaders.push(PluginLoaderMatch::Spigot),
					KnownLoader::Sponge => plugin_loaders.push(PluginLoaderMatch::Sponge),
					KnownLoader::Paper => plugin_loaders.push(PluginLoaderMatch::Paper),
					KnownLoader::Purpur => plugin_loaders.push(PluginLoaderMatch::Purpur),
				},
				Loader::Unknown(other) => panic!("Unknown loader {other}"),
			}
		}

		let mut deps = Vec::new();
		let mut recommendations = Vec::new();
		let mut extensions = Vec::new();
		let mut conflicts = Vec::new();

		for dep in &version.dependencies {
			let pkg_id = if let Some(dep_id) = relation_substitutions.get(&dep.project_id) {
				dep_id.clone()
			} else {
				panic!("Dependency {} was not substituted", dep.project_id)
			};
			match dep.dependency_type {
				DependencyType::Required => {
					if force_extensions.contains(&pkg_id) {
						extensions.push(pkg_id);
					} else {
						deps.push(pkg_id)
					}
				}
				DependencyType::Optional => recommendations.push(RecommendedPackage {
					value: pkg_id.into(),
					invert: false,
				}),
				DependencyType::Incompatible => conflicts.push(pkg_id),
				// We don't need to do anything with embedded dependencies yet
				DependencyType::Embedded => {}
			}
		}

		let mut pkg_version = DeclarativeAddonVersion {
			version: Some(version_name),
			conditional_properties: DeclarativeConditionSet {
				minecraft_versions: Some(DeserListOrSingle::List(mc_versions)),
				modloaders: Some(DeserListOrSingle::List(modloaders)),
				plugin_loaders: Some(DeserListOrSingle::List(plugin_loaders)),
				..Default::default()
			},
			relations: DeclarativePackageRelations {
				dependencies: DeserListOrSingle::List(deps),
				recommendations: DeserListOrSingle::List(recommendations),
				extensions: DeserListOrSingle::List(extensions),
				conflicts: DeserListOrSingle::List(conflicts),
				..Default::default()
			},
			..Default::default()
		};

		// Select download
		let download = version
			.get_primary_download()
			.expect("Version has no available downloads");
		pkg_version.url = Some(download.url.clone());

		addon.versions.push(pkg_version);
	}

	props.supported_versions = Some(all_mc_versions);

	let mut addon_map = HashMap::new();
	addon_map.insert("addon".into(), addon);

	let pkg = DeclarativePackage {
		meta,
		properties: props,
		addons: addon_map,
		..Default::default()
	};

	pkg
}
