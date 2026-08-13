use std::collections::{HashMap, HashSet};

use anyhow::Context;
use nitro_pkg::RecommendedPackage;
use nitro_pkg::declarative::{
	DeclarativeAddon, DeclarativeAddonVersion, DeclarativeConditionSet, DeclarativePackage,
	DeclarativePackageRelations,
};
use nitro_pkg::metadata::PackageMetadata;
use nitro_pkg::properties::PackageProperties;
use nitro_shared::loaders::{Loader, LoaderMatch};
use nitro_shared::pkg::{AddonHashes, PackageCategory, PackageKind, PackageStability};
use nitro_shared::util::DeserListOrSingle;
use nitro_shared::versions::VersionPattern;

use nitro_net::modrinth::{
	self, DependencyType, FullGalleryEntry, GalleryEntry, KnownLoader, License, Member,
	ModrinthLoader, Project, ProjectType, ReleaseChannel, SearchedProject, SideSupport, Version,
};
use nitro_shared::Side;

use crate::relation_substitution::{RelationSubFunction, substitute_multiple};

/// Generates a Modrinth package from a Modrinth project ID
pub async fn generate_from_id(
	id: &str,
	relation_substitution: impl RelationSubFunction,
	force_extensions: &[String],
	make_fabriclike: bool,
	make_forgelike: bool,
	repository: Option<&str>,
) -> anyhow::Result<DeclarativePackage> {
	let client = nitro_core::net::download::Client::new();
	let project = modrinth::get_project(id, &client)
		.await
		.expect("Failed to get Modrinth project");

	let versions = modrinth::get_multiple_versions(&project.versions, &client)
		.await
		.expect("Failed to get Modrinth project versions");

	let members = modrinth::get_project_team(id, &client)
		.await
		.expect("Failed to get project team members from Modrinth");

	generate(
		project,
		&versions,
		&members,
		relation_substitution,
		force_extensions,
		make_fabriclike,
		make_forgelike,
		repository,
	)
	.await
}

/// Generates a Modrinth package from a Modrinth project
pub async fn generate(
	project: Project,
	versions: &[Version],
	members: &[Member],
	mut relation_substitution: impl RelationSubFunction,
	force_extensions: &[String],
	make_fabriclike: bool,
	make_forgelike: bool,
	repository: Option<&str>,
) -> anyhow::Result<DeclarativePackage> {
	// Get supported sides
	let supported_sides = get_supported_sides(&project);

	// Fill out metadata
	let mut meta = PackageMetadata {
		slug: Some(project.slug),
		name: Some(project.title),
		description: Some(project.description),
		downloads: Some(project.downloads),
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
		} else {
			meta.banner = gallery.first().map(|x| x.get_url().to_string());
		}
		meta.gallery = Some(
			gallery
				.into_iter()
				.map(|x| x.get_url().to_string())
				.collect(),
		);
	}

	meta.categories = Some(
		project
			.categories
			.into_iter()
			.flat_map(|x| convert_category(&x).into_iter())
			.collect(),
	);

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
	// I hate Modrinth
	let package_type = if (project.project_type == ProjectType::Mod && project.loaders.is_empty())
		|| (project.loaders.len() == 1
			&& project.loaders[0] == ModrinthLoader::Known(KnownLoader::Datapack))
	{
		PackageKind::Datapack
	} else {
		match project.project_type {
			ProjectType::Mod => PackageKind::Mod,
			ProjectType::Datapack => PackageKind::Datapack,
			ProjectType::Plugin => PackageKind::Plugin,
			ProjectType::ResourcePack => PackageKind::ResourcePack,
			ProjectType::Shader => PackageKind::Shader,
			ProjectType::Modpack => PackageKind::Modpack,
		}
	};

	let modpack_format = if package_type == PackageKind::Modpack {
		Some("mrpack".into())
	} else {
		None
	};

	let mut addon = DeclarativeAddon {
		kind: package_type,
		modpack_format,
		versions: Vec::new(),
		conditions: Vec::new(),
		optional: false,
	};

	props.kinds = vec![package_type];

	let oops_all_datapacks = versions.iter().all(|x| {
		x.loaders
			.contains(&ModrinthLoader::Known(KnownLoader::Datapack))
	});

	// Make substitutions
	let mut substitutions = HashSet::new();
	let mut substitutions_without_versions = HashSet::new();
	for version in versions {
		for dependency in &version.dependencies {
			if let Some(project_id) = &dependency.project_id {
				substitutions.insert((project_id.clone(), dependency.version_id.clone()));
				substitutions_without_versions.insert(project_id.clone());
			}
		}
	}

	relation_substitution
		.preload_substitutions(
			&substitutions_without_versions
				.into_iter()
				.collect::<Vec<_>>(),
		)
		.await
		.context("Failed to preload substitutions")?;

	let substitutions = substitute_multiple(substitutions.iter(), relation_substitution)
		.await
		.context("Failed to substitute relations")?;

	let mut content_versions = Vec::with_capacity(versions.len());
	let mut all_loaders = HashSet::new();
	let mut needs_datapack_feature = false;

	for version in versions {
		let version_name = version.id.clone();
		// Collect Minecraft versions
		let mc_versions: Vec<VersionPattern> = version
			.game_versions
			.iter()
			.map(|x| VersionPattern::Single(x.clone()))
			.collect();

		// Look at loaders
		let mut loaders = Vec::new();
		let mut skip = false;
		let mut is_datapack_version = false;
		for loader in &version.loaders {
			match loader {
				ModrinthLoader::Known(loader) => match loader {
					KnownLoader::Fabric => loaders.push(if make_fabriclike {
						LoaderMatch::FabricLike
					} else {
						LoaderMatch::Loader(Loader::Fabric)
					}),
					KnownLoader::Quilt => loaders.push(LoaderMatch::Loader(Loader::Quilt)),
					KnownLoader::Forge => loaders.push(if make_forgelike {
						LoaderMatch::ForgeLike
					} else {
						LoaderMatch::Loader(Loader::Forge)
					}),
					KnownLoader::NeoForged => loaders.push(LoaderMatch::Loader(Loader::NeoForged)),
					KnownLoader::Rift => loaders.push(LoaderMatch::Loader(Loader::Rift)),
					KnownLoader::Liteloader => {
						loaders.push(LoaderMatch::Loader(Loader::LiteLoader))
					}
					KnownLoader::Risugamis => loaders.push(LoaderMatch::Loader(Loader::Risugamis)),
					KnownLoader::Bukkit => loaders.push(LoaderMatch::Bukkit),
					KnownLoader::Folia => loaders.push(LoaderMatch::Loader(Loader::Folia)),
					KnownLoader::Spigot => loaders.push(LoaderMatch::Loader(Loader::Spigot)),
					KnownLoader::Sponge => loaders.push(LoaderMatch::Loader(Loader::Sponge)),
					KnownLoader::Paper => loaders.push(LoaderMatch::Loader(Loader::Paper)),
					KnownLoader::Purpur => loaders.push(LoaderMatch::Loader(Loader::Purpur)),
					KnownLoader::Datapack => {
						is_datapack_version = true;
						if addon.kind == PackageKind::Mod && !oops_all_datapacks {
							needs_datapack_feature = true;
						}
					}
					// Skip over these versions for now
					KnownLoader::BungeeCord | KnownLoader::Velocity | KnownLoader::Waterfall => {
						skip = true
					}
					// We don't care about these
					KnownLoader::Iris | KnownLoader::Optifine | KnownLoader::Minecraft => {}
				},
				ModrinthLoader::Unknown(..) => {}
			}
		}
		if skip {
			continue;
		}

		all_loaders.extend(loaders.clone());

		// Get stability
		let stability = match version.version_type {
			ReleaseChannel::Release => PackageStability::Stable,
			ReleaseChannel::Alpha | ReleaseChannel::Beta => PackageStability::Latest,
		};

		let mut deps = Vec::new();
		let mut recommendations = Vec::new();
		let mut extensions = Vec::new();
		let mut bundled = Vec::new();
		let mut conflicts = Vec::new();

		for dep in &version.dependencies {
			let Some(project_id) = &dep.project_id else {
				continue;
			};
			let (pkg_id, pkg_version) = substitutions
				.get(&(project_id.clone(), dep.version_id.clone()))
				.expect("Should have errored already")
				.clone();

			// Don't count none relations
			if pkg_id == "none" {
				continue;
			}

			let req = if let Some(repo) = &repository {
				format!("{repo}:{pkg_id}")
			} else {
				pkg_id
			};
			let req = if let Some(pkg_version) = pkg_version {
				format!("{req}@~{pkg_version}")
			} else {
				req
			};

			match dep.dependency_type {
				DependencyType::Required => {
					if addon.kind == PackageKind::Bundle || addon.kind == PackageKind::Modpack {
						bundled.push(req);
					} else if force_extensions.contains(&req) {
						extensions.push(req);
					} else {
						deps.push(req)
					}
				}
				DependencyType::Optional => recommendations.push(RecommendedPackage {
					value: req.into(),
					invert: false,
				}),
				DependencyType::Incompatible => conflicts.push(req),
				DependencyType::Embedded => {
					if addon.kind == PackageKind::Bundle || addon.kind == PackageKind::Modpack {
						bundled.push(req);
					}
				}
			}
		}

		// Sort relations
		deps.sort();
		recommendations.sort();
		extensions.sort();
		bundled.sort();
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
				loaders: Some(DeserListOrSingle::List(loaders)),
				stability: Some(stability),
				content_versions: Some(DeserListOrSingle::Single(content_version)),
				features: if needs_datapack_feature && is_datapack_version {
					Some(DeserListOrSingle::Single("datapack".into()))
				} else {
					None
				},
				..Default::default()
			},
			relations: DeclarativePackageRelations {
				dependencies: DeserListOrSingle::List(deps),
				recommendations: DeserListOrSingle::List(recommendations),
				extensions: DeserListOrSingle::List(extensions),
				bundled: DeserListOrSingle::List(bundled),
				conflicts: DeserListOrSingle::List(conflicts),
				..Default::default()
			},

			..Default::default()
		};

		// Select download for non-bundle kinds
		if addon.kind != PackageKind::Bundle {
			let Ok(download) = version.get_primary_download() else {
				continue;
			};
			pkg_version.url = Some(download.url.clone());
			pkg_version.filename = Some(download.filename.clone());
			pkg_version.hashes = AddonHashes {
				sha256: None,
				sha512: Some(download.hashes.sha512.clone()),
			}
		}

		addon.versions.push(pkg_version);
	}

	content_versions.reverse();
	props.content_versions = Some(content_versions);
	props.supported_loaders = Some(all_loaders.into_iter().collect());
	if needs_datapack_feature {
		props.features = Some(vec!["datapack".into()]);
	}

	let mut addon_map = HashMap::new();
	addon_map.insert("addon".into(), addon);

	Ok(DeclarativePackage {
		meta,
		properties: props,
		addons: addon_map,
		..Default::default()
	})
}

/// Gets the project for use in generating search result previews for this project
pub fn get_preview(result: SearchedProject) -> Project {
	Project {
		id: result.id,
		slug: result.slug,
		project_type: result.project_type,
		title: result.title,
		description: result.description,
		categories: result.display_categories,
		game_versions: result.versions,
		icon_url: result.icon_url,
		gallery: result.gallery.map(|x| {
			x.into_iter()
				.map(|x| {
					if result
						.featured_gallery
						.as_ref()
						.is_some_and(|featured| &x == featured)
					{
						GalleryEntry::Full(FullGalleryEntry {
							url: x.clone(),
							raw_url: x,
							featured: true,
						})
					} else {
						GalleryEntry::Simple(x)
					}
				})
				.collect()
		}),
		downloads: result.downloads,
		..Default::default()
	}
}

/// Gets the list of supported sides from the project
fn get_supported_sides(project: &Project) -> Vec<Side> {
	let mut out = Vec::with_capacity(2);
	if let SideSupport::Required | SideSupport::Optional = &project.client_side {
		out.push(Side::Client);
	}
	if let SideSupport::Required | SideSupport::Optional = &project.server_side {
		out.push(Side::Server);
		out.push(Side::Client);
	}
	out
}

/// Cleanup a version name to remove things like loaders
pub fn cleanup_version_name(version: &str) -> String {
	version.replace("+", "-")
}

fn convert_category(category: &str) -> Vec<PackageCategory> {
	match category {
		"adventure" => vec![PackageCategory::Adventure],
		"atmosphere" => vec![PackageCategory::Atmosphere],
		"audio" => vec![PackageCategory::Audio],
		"blocks" => vec![PackageCategory::Blocks, PackageCategory::Building],
		"cartoon" => vec![PackageCategory::Cartoon],
		"challenging" => vec![PackageCategory::Challenge],
		"combat" => vec![PackageCategory::Combat],
		"decoration" => vec![PackageCategory::Decoration, PackageCategory::Building],
		"economy" => vec![PackageCategory::Economy],
		"entities" => vec![PackageCategory::Entities],
		"equipment" => vec![PackageCategory::Equipment],
		"fantasy" => vec![PackageCategory::Fantasy],
		"fonts" => vec![PackageCategory::Fonts],
		"food" => vec![PackageCategory::Food],
		"game-mechanics" => vec![PackageCategory::GameMechanics],
		"gui" => vec![PackageCategory::Gui],
		"items" => vec![PackageCategory::Items],
		"kitchen-sink" => vec![PackageCategory::Extensive],
		"library" => vec![PackageCategory::Library],
		"lightweight" => vec![PackageCategory::Lightweight],
		"locale" => vec![PackageCategory::Language],
		"magic" => vec![PackageCategory::Magic],
		"minigame" => vec![PackageCategory::Minigame],
		"mobs" => vec![PackageCategory::Mobs],
		"multiplayer" => vec![PackageCategory::Multiplayer],
		"optimization" => vec![PackageCategory::Optimization],
		"realistic" => vec![PackageCategory::Realistic],
		"simplistic" => vec![PackageCategory::Simplistic],
		"social" => vec![PackageCategory::Social],
		"storage" => vec![PackageCategory::Storage],
		"technology" => vec![PackageCategory::Technology],
		"transportation" => vec![PackageCategory::Transportation],
		"tweaks" => vec![PackageCategory::Tweaks],
		"utility" => vec![PackageCategory::Utility],
		"vanilla-like" => vec![PackageCategory::VanillaPlus],
		"worldgen" => vec![PackageCategory::Worldgen, PackageCategory::Exploration],
		_ => Vec::new(),
	}
}
