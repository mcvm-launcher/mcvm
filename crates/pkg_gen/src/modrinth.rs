use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use anyhow::Context;
use mcvm_pkg::declarative::{
	DeclarativeAddon, DeclarativeAddonVersion, DeclarativeConditionSet, DeclarativePackage,
	DeclarativePackageRelations,
};
use mcvm_pkg::metadata::PackageMetadata;
use mcvm_pkg::properties::PackageProperties;
use mcvm_pkg::RecommendedPackage;
use mcvm_shared::addon::AddonKind;
use mcvm_shared::modifications::{ModloaderMatch, PluginLoaderMatch};
use mcvm_shared::pkg::PackageStability;
use mcvm_shared::util::DeserListOrSingle;
use mcvm_shared::versions::VersionPattern;

use mcvm_net::modrinth::{
	self, DependencyType, GalleryEntry, KnownLoader, License, Loader, Member, Project, ProjectType,
	ReleaseChannel, SideSupport, Version,
};
use mcvm_shared::Side;
use regex::{Regex, RegexBuilder};

use crate::relation_substitution::{substitute_multiple, RelationSubFunction};

/// Generates a Modrinth package from a Modrinth project ID
pub async fn gen_from_id(
	id: &str,
	relation_substitution: impl RelationSubFunction,
	force_extensions: &[String],
	make_fabriclike: bool,
	make_forgelike: bool,
) -> anyhow::Result<DeclarativePackage> {
	let client = mcvm_core::net::download::Client::new();
	let project = modrinth::get_project(id, &client)
		.await
		.expect("Failed to get Modrinth project");

	let versions = modrinth::get_multiple_versions(&project.versions, &client)
		.await
		.expect("Failed to get Modrinth project versions");

	let members = modrinth::get_project_team(id, &client)
		.await
		.expect("Failed to get project team members from Modrinth");

	gen(
		project,
		&versions,
		&members,
		relation_substitution,
		force_extensions,
		make_fabriclike,
		make_forgelike,
	)
	.await
}

/// Generates a Modrinth package from a Modrinth project
pub async fn gen(
	project: Project,
	versions: &[Version],
	members: &[Member],
	relation_substitution: impl RelationSubFunction,
	force_extensions: &[String],
	make_fabriclike: bool,
	make_forgelike: bool,
) -> anyhow::Result<DeclarativePackage> {
	// Get supported sides
	let supported_sides = get_supported_sides(&project);

	// Fill out metadata
	let mut meta = PackageMetadata {
		name: Some(project.title),
		description: Some(project.description),
		..Default::default()
	};
	if let Some(body) = project.body {
		meta.long_description = Some(body);
	}
	if let Some(icon_url) = project.icon_url {
		meta.icon = Some(icon_url);
	}
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
	// Sort donation URLs as their order does not seem to be deterministic
	let mut donation_urls = project.donation_urls;
	donation_urls.sort_by_key(|x| x.url.clone());
	if let Some(support_link) = donation_urls.first() {
		meta.support_link = Some(support_link.url.clone());
	}
	if let Some(gallery) = project.gallery {
		// Get the banner image from the featured gallery image
		if let Some(banner) = gallery
			.iter()
			.find(|x| matches!(x, GalleryEntry::Full(entry) if entry.featured))
		{
			meta.banner = Some(banner.get_url().to_string());
		}
		meta.gallery = Some(
			gallery
				.into_iter()
				.map(|x| x.get_url().to_string())
				.collect(),
		);
	}

	// Handle custom licenses
	meta.license = Some(match project.license {
		License::Short(license) => license,
		License::Long(license) => {
			if license.id == "LicenseRef-Custom" {
				if let Some(url) = license.url {
					url
				} else {
					"Custom".into()
				}
			} else {
				license.id
			}
		}
	});

	// Get team members and use them to fill out the authors field
	let mut members = members.to_vec();
	members.sort();
	meta.authors = Some(members.into_iter().map(|x| x.user.username).collect());

	// Create properties
	let mut props = PackageProperties {
		modrinth_id: Some(project.id),
		supported_sides: Some(supported_sides),
		supported_versions: Some(
			project
				.game_versions
				.into_iter()
				.map(|x| VersionPattern::from(&x))
				.collect(),
		),
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
		optional: false,
	};

	let mut content_versions = Vec::with_capacity(versions.len());

	// Make substitutions
	let mut substitutions = HashSet::new();
	for version in versions {
		for dependency in &version.dependencies {
			if let Some(project_id) = &dependency.project_id {
				substitutions.insert(project_id);
			}
		}
	}
	let substitutions = substitute_multiple(substitutions.into_iter(), relation_substitution)
		.await
		.context("Failed to substitute relations")?;

	for version in versions {
		let version_name = version.id.clone();
		// Collect Minecraft versions
		let mc_versions: Vec<VersionPattern> = version
			.game_versions
			.iter()
			.map(|x| VersionPattern::Single(x.clone()))
			.collect();

		// Look at loaders
		let mut modloaders = Vec::new();
		let mut plugin_loaders = Vec::new();
		let mut skip = false;
		for loader in &version.loaders {
			match loader {
				Loader::Known(loader) => match loader {
					KnownLoader::Fabric => modloaders.push(if make_fabriclike {
						ModloaderMatch::FabricLike
					} else {
						ModloaderMatch::Fabric
					}),
					KnownLoader::Quilt => modloaders.push(ModloaderMatch::Quilt),
					KnownLoader::Forge => modloaders.push(if make_forgelike {
						ModloaderMatch::ForgeLike
					} else {
						ModloaderMatch::Forge
					}),
					KnownLoader::NeoForged => modloaders.push(ModloaderMatch::NeoForged),
					KnownLoader::Rift => modloaders.push(ModloaderMatch::Rift),
					KnownLoader::Liteloader => modloaders.push(ModloaderMatch::LiteLoader),
					KnownLoader::Risugamis => modloaders.push(ModloaderMatch::Risugamis),
					KnownLoader::Bukkit => plugin_loaders.push(PluginLoaderMatch::Bukkit),
					KnownLoader::Folia => plugin_loaders.push(PluginLoaderMatch::Folia),
					KnownLoader::Spigot => plugin_loaders.push(PluginLoaderMatch::Spigot),
					KnownLoader::Sponge => plugin_loaders.push(PluginLoaderMatch::Sponge),
					KnownLoader::Paper => plugin_loaders.push(PluginLoaderMatch::Paper),
					KnownLoader::Purpur => plugin_loaders.push(PluginLoaderMatch::Purpur),
					// Skip over these versions for now
					KnownLoader::Datapack
					| KnownLoader::BungeeCord
					| KnownLoader::Velocity
					| KnownLoader::Waterfall => skip = true,
					// We don't care about these
					KnownLoader::Iris | KnownLoader::Optifine | KnownLoader::Minecraft => {}
				},
				Loader::Unknown(other) => panic!("Unknown loader {other}"),
			}
		}
		if skip {
			continue;
		}

		// Get stability
		let stability = match version.version_type {
			ReleaseChannel::Release => PackageStability::Stable,
			ReleaseChannel::Alpha | ReleaseChannel::Beta => PackageStability::Latest,
		};

		let mut deps = Vec::new();
		let mut recommendations = Vec::new();
		let mut extensions = Vec::new();
		let mut conflicts = Vec::new();

		for dep in &version.dependencies {
			let Some(project_id) = &dep.project_id else {
				continue;
			};
			let pkg_id = substitutions
				.get(project_id)
				.expect("Should have errored already")
				.clone();
			// Don't count none relations
			if pkg_id == "none" {
				continue;
			}
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
				DependencyType::Embedded => continue,
			}
		}

		// Sort relations
		deps.sort();
		recommendations.sort();
		extensions.sort();
		conflicts.sort();

		// Content versions
		let content_version = cleanup_version_name(&version.version_number);
		if !content_versions.contains(&content_version) {
			content_versions.push(content_version.clone());
		}

		let mut pkg_version = DeclarativeAddonVersion {
			version: Some(version_name),
			conditional_properties: DeclarativeConditionSet {
				minecraft_versions: Some(DeserListOrSingle::List(mc_versions)),
				modloaders: Some(DeserListOrSingle::List(modloaders)),
				plugin_loaders: Some(DeserListOrSingle::List(plugin_loaders)),
				stability: Some(stability),
				content_versions: Some(DeserListOrSingle::Single(content_version)),
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

	props.content_versions = Some(content_versions);

	let mut addon_map = HashMap::new();
	addon_map.insert("addon".into(), addon);

	Ok(DeclarativePackage {
		meta,
		properties: props,
		addons: addon_map,
		..Default::default()
	})
}

/// Gets the list of supported sides from the project
fn get_supported_sides(project: &Project) -> Vec<Side> {
	let mut out = Vec::with_capacity(2);
	if let SideSupport::Required | SideSupport::Optional = &project.client_side {
		out.push(Side::Client);
	}
	if let SideSupport::Required | SideSupport::Optional = &project.server_side {
		out.push(Side::Server);
	}
	out
}

/// Cleanup a version name to remove things like modloaders
fn cleanup_version_name(version: &str) -> String {
	static MODLOADER_REGEX: OnceLock<Regex> = OnceLock::new();
	let regex = MODLOADER_REGEX.get_or_init(|| {
		RegexBuilder::new("(-|_|\\+)?(fabric|forge|quilt)")
			.case_insensitive(true)
			.build()
			.expect("Failed to create regex")
	});
	let version = regex.replace_all(version, "");
	let version = version.replace("+", "-");

	version
}
