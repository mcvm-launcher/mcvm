use std::{
	collections::{HashMap, HashSet},
	fs::File,
	io::BufReader,
	path::{Path, PathBuf},
	sync::Arc,
	time::SystemTime,
};

use anyhow::{Context, bail};
use nitro_core::io::{files::create_leading_dirs, json_from_file, json_to_file};
use nitro_instance::addon::modpack::mrpack::{ModrinthIndex, ModrinthPack};
use nitro_net::{
	download::Client,
	modrinth::{self, Member, Project, SearchResults, Version},
};
use nitro_pkg::{PackageMetaAndProps, PackageSearchResults, PkgRequest, PkgRequestSource};
use nitro_pkg_gen::{modrinth::get_preview, relation_substitution::RelationSubNone};
use nitro_plugin::{
	api::{executable::ExecutablePlugin, utils::PackageSearchCache},
	hook::hooks::{CustomRepoQueryResult, ImportInstanceResult, InstallModpackResult},
};
use nitro_shared::{
	Side,
	io::update_link,
	output::{MessageContents, NitroOutput},
	versions::{MinecraftVersionDeser, VersionPattern},
};
use nitrolaunch::config_crate::instance::InstanceConfig;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

const PROJECT_CACHE_TIME_SECS: u64 = 3600;

fn main() -> anyhow::Result<()> {
	let mut plugin = ExecutablePlugin::from_manifest_file("modrinth", include_str!("plugin.json"))?;

	plugin.query_custom_package_repository(|ctx, arg| {
		if arg.repository != "modrinth" {
			return Ok(None);
		}

		let data_dir = ctx.get_data_dir()?;
		let storage_dirs = StorageDirs::new(&data_dir);

		let runtime = tokio::runtime::Runtime::new()?;
		let client = Client::new();

		runtime.block_on(query_package(&arg.package, &client, &storage_dirs))
	})?;

	plugin.preload_packages(|ctx, arg| {
		if arg.repository != "modrinth" {
			return Ok(());
		}

		let data_dir = ctx.get_data_dir()?;
		let storage_dirs = StorageDirs::new(&data_dir);

		let runtime = tokio::runtime::Runtime::new()?;
		let client = Client::new();

		runtime.block_on(async move {
			if arg.packages.len() > 3 {
				let _ =
					download_multiple_projects(&arg.packages, &storage_dirs, &client, false).await;
			} else {
				let mut tasks = tokio::task::JoinSet::new();
				for package in arg.packages {
					let client = client.clone();
					let storage_dirs = storage_dirs.clone();

					tasks.spawn(
						async move { query_package(&package, &client, &storage_dirs).await },
					);
				}

				while let Some(task) = tasks.join_next().await {
					let _ = task??;
				}
			}

			Ok::<(), anyhow::Error>(())
		})?;

		Ok(())
	})?;

	plugin.search_custom_package_repository(|ctx, arg| {
		if arg.repository != "modrinth" {
			return Ok(PackageSearchResults::default());
		}

		let client = Client::new();
		let runtime = tokio::runtime::Runtime::new()?;

		let data_dir = ctx.get_data_dir()?;

		let (projects, previews, total_results) = runtime.block_on(async move {
			let cache_path = data_dir.join("internal/modrinth/search_cache.json");
			create_leading_dirs(&cache_path)?;
			let mut search_cache =
				PackageSearchCache::open(cache_path, 250).context("Failed to open search cache")?;

			let results =
				if let Some(results) = search_cache.check::<SearchResults>(&arg.parameters) {
					results
				} else {
					let results = modrinth::search_projects(arg.parameters.clone(), &client)
						.await
						.context("Failed to search projects from the API")?;

					let _ = search_cache.write(&arg.parameters, results.clone());
					results
				};

			let mut previews = HashMap::with_capacity(results.hits.len());
			let mut projects = Vec::with_capacity(results.hits.len());
			for result in results.hits {
				let req = PkgRequest {
					source: PkgRequestSource::UserRequire,
					id: result.id.clone().into(),
					content_version: VersionPattern::Any,
					repository: Some("modrinth".into()),
					slug: Some(result.slug.clone()),
				};
				let req_str = req.to_string();

				projects.push(req_str.clone());
				let package = nitro_pkg_gen::modrinth::generate(
					get_preview(result),
					&[],
					&[],
					RelationSubNone,
					&[],
					true,
					true,
					Some("modrinth"),
				)
				.await;
				if let Ok(package) = package {
					previews.insert(
						req_str,
						PackageMetaAndProps {
							meta: Arc::new(package.meta),
							props: Arc::new(package.properties),
						},
					);
				}
			}

			Ok::<_, anyhow::Error>((projects, previews, results.total_hits))
		})?;

		Ok(PackageSearchResults {
			results: projects,
			total_results,
			previews,
		})
	})?;

	plugin.sync_custom_package_repository(|ctx, arg| {
		if arg.repository != "modrinth" {
			return Ok(());
		}

		let storage_dirs = StorageDirs::new(&ctx.get_data_dir()?);

		if storage_dirs.packages.exists() {
			std::fs::remove_dir_all(storage_dirs.packages)
				.context("Failed to remove cached packages")?;
		}
		if storage_dirs.projects.exists() {
			std::fs::remove_dir_all(storage_dirs.projects)
				.context("Failed to remove cached projects")?;
		}

		Ok(())
	})?;

	plugin.install_modpack(|mut ctx, arg| {
		let mut old_pack = if let Some(old_path) = arg.old_path {
			let file = BufReader::new(File::open(old_path).context("Failed to open old modpack")?);
			Some(ModrinthPack::from_stream(file).context("Failed to open old modpack")?)
		} else {
			None
		};

		let data_dir = ctx.get_data_dir()?;
		let addons_dir = data_dir.join("internal/addons");

		let file = BufReader::new(File::open(arg.path).context("Failed to open modpack")?);
		let mut pack = ModrinthPack::from_stream(file).context("Failed to open modpack")?;

		let mut process = ctx.get_output().get_process();
		process.display(MessageContents::StartProcess(
			"Downloading modpack files".into(),
		));

		let client = Client::new();
		let runtime = tokio::runtime::Runtime::new()?;

		runtime
			.block_on(pack.download(&addons_dir, &client))
			.context("Failed to download modpack files")?;

		process.display(MessageContents::Success("Modpack files downloaded".into()));
		process.finish();

		let mut process = ctx.get_output().get_process();
		process.display(MessageContents::StartProcess("Installing modpack".into()));

		pack.apply(
			Path::new(&arg.target_path),
			&addons_dir,
			arg.side,
			old_pack.as_mut(),
		)
		.context("Failed to apply modpack")?;

		let addons = pack.get_addons(Path::new(&arg.target_path), &addons_dir)?;

		process.display(MessageContents::Success("Modrinth pack installed".into()));

		let mut packages = Vec::new();
		for file in &pack.index().files {
			let (Some(project_id), _) = file.get_modrinth_info() else {
				continue;
			};

			packages.push(format!("modrinth:{project_id}"));
		}

		Ok(InstallModpackResult {
			name: pack.index().name.clone(),
			packages,
			addons,
		})
	})?;

	plugin.import_instance(|mut ctx, arg| {
		if arg.format != "mrpack" {
			bail!("Invalid format");
		}

		let source_path = PathBuf::from(arg.source_path);
		let target_path = PathBuf::from(arg.result_path);

		let addons_dir = ctx.get_data_dir()?.join("internal/addons");

		let output = ctx.get_output();

		let side = arg.side.context("Side not specified")?;

		let file = File::open(source_path).context("Failed to open pack file")?;
		let mut modpack = ModrinthPack::from_stream(file).context("Failed to open mrpack")?;

		// Download files
		let mut process = output.get_process();
		process.display(MessageContents::StartProcess("Downloading mods".into()));

		let runtime = tokio::runtime::Runtime::new()?;
		let client = Client::new();
		runtime
			.block_on(modpack.download(&addons_dir, &client))
			.context("Failed to download modpack files")?;

		process.display(MessageContents::Success("Mods downloaded".into()));
		process.finish();

		let target_path = match side {
			Side::Client => target_path.join(".minecraft"),
			Side::Server => target_path,
		};

		let mut process = output.get_process();
		process.display(MessageContents::StartProcess("Installing modpack".into()));
		modpack
			.apply(&target_path, &addons_dir, side, None)
			.context("Failed to install modpack")?;
		process.display(MessageContents::Success("Modpack installed".into()));
		process.finish();

		let config = mrpack_index_to_config(modpack.index(), side);

		Ok(ImportInstanceResult {
			format: arg.format,
			config,
		})
	})?;

	Ok(())
}

/// Queries for a Modrinth package
async fn query_package(
	id: &str,
	client: &Client,
	storage_dirs: &StorageDirs,
) -> anyhow::Result<Option<CustomRepoQueryResult>> {
	let project_info = get_cached_project(id, storage_dirs, client)
		.await
		.with_context(|| format!("Failed to get cached project '{id}'"))?;
	let Some(project_info) = project_info else {
		return Ok(None);
	};

	let mut package = nitro_pkg_gen::modrinth::generate(
		project_info.project,
		&project_info.versions,
		&project_info.members,
		RelationSubNone,
		&[],
		true,
		true,
		Some("modrinth"),
	)
	.await
	.context("Failed to generate Nitrolaunch package")?;

	package.improve_generation();
	package.optimize();

	let package = serde_json::to_string_pretty(&package).context("Failed to serialized package")?;

	Ok(Some(CustomRepoQueryResult {
		contents: package,
		content_type: nitrolaunch::pkg_crate::PackageContentType::Declarative,
		flags: HashSet::new(),
	}))
}

/// Gets a cached Modrinth project and it's versions or downloads it
async fn get_cached_project(
	project_id: &str,
	storage_dirs: &StorageDirs,
	client: &Client,
) -> anyhow::Result<Option<ProjectInfo>> {
	let project_path = storage_dirs.projects.join(project_id);
	// If a project does not exist, we create a dummy file so that we know not to fetch it again
	let does_not_exist_path = storage_dirs.get_missing_path(project_id);
	if does_not_exist_path.exists() {
		return Ok(None);
	}

	let project_info = if project_path.exists()
		&& !project_needs_update(&project_path).unwrap_or(true)
	{
		// TODO: Add versions to partially loaded project infos
		json_from_file(&project_path).context("Failed to read project info from file")?
	} else {
		let project_task = {
			let project = project_id.to_string();
			let client = client.clone();
			tokio::spawn(async move { modrinth::get_project_optional(&project, &client).await })
		};

		let members_task = {
			let project = project_id.to_string();
			let client = client.clone();
			tokio::spawn(async move { modrinth::get_project_team(&project, &client).await })
		};

		let versions_task = {
			let project = project_id.to_string();
			let client = client.clone();
			tokio::spawn(async move { modrinth::get_project_versions(&project, &client).await })
		};

		let (project, members, versions) = tokio::join!(project_task, members_task, versions_task);
		let project = project
			.context("Failed to get project")?
			.context("Failed to get project")?;
		let project = match project {
			Some(project) => project,
			None => {
				let file = std::fs::File::create(does_not_exist_path);
				std::mem::drop(file);
				return Ok(None);
			}
		};

		let members = members
			.context("Failed to get project members")?
			.context("Failed to get project members")?;
		let versions = versions
			.context("Failed to get project versions")?
			.context("Failed to get project versions")?;

		let project_info = ProjectInfo {
			project,
			versions,
			members,
		};

		let _ = save_project_info(&project_info, storage_dirs);

		project_info
	};

	Ok(Some(project_info))
}

/// Gets a cached Modrinth project and it's versions, but never downloads it
fn get_known_cached_project(
	project_id: &str,
	storage_dirs: &StorageDirs,
) -> anyhow::Result<Option<ProjectInfo>> {
	let project_path = storage_dirs.projects.join(project_id);
	// If a project does not exist, we create a dummy file so that we know not to fetch it again
	let does_not_exist_path = storage_dirs.get_missing_path(project_id);
	if does_not_exist_path.exists() {
		return Ok(None);
	}

	Ok(Some(
		json_from_file(&project_path).context("Failed to read project info from file")?,
	))
}

/// Downloads multiple projects at once to save on API requests. Will have much higher latency, but is better for
/// downloading lots of projects as we won't get ratelimited
async fn download_multiple_projects(
	projects: &[String],
	storage_dirs: &StorageDirs,
	client: &Client,
	download_dependencies: bool,
) -> anyhow::Result<Vec<ProjectInfo>> {
	// Filter out projects that are already cached and don't need updated
	let project_ids: Vec<_> = projects
		.iter()
		.filter(|x| {
			let path = storage_dirs.projects.join(x);
			if !path.exists() {
				return true;
			}

			project_needs_update(&path).unwrap_or(true)
		})
		.cloned()
		.collect();

	if project_ids.is_empty() {
		return Ok(Vec::new());
	}

	// Download the new projects
	let projects = modrinth::get_multiple_projects(&project_ids, client)
		.await
		.context("Failed to download projects")?;

	// List of existing and new projects to have new data applied to them
	let mut project_needed_versions = Vec::new();
	let projects: Vec<_> = projects
		.into_iter()
		.filter_map(|project| {
			let path = storage_dirs.projects.join(&project.id);
			if !path.exists() {
				project_needed_versions.extend(project.versions.clone());
				Some(ProjectInfo {
					project,
					versions: Vec::new(),
					members: Vec::new(),
				})
			} else {
				let mut existing_project = get_known_cached_project(&project.id, storage_dirs)
					.ok()
					.flatten();
				if let Some(existing_project) = &mut existing_project {
					let missing_versions = existing_project
						.get_missing_versions(&project.versions)
						.into_iter()
						.map(|x| x.to_string());

					project_needed_versions.extend(missing_versions);
					existing_project.project = project;
				}

				existing_project
			}
		})
		.collect();

	// Collect Modrinth project versions. We have to batch these into multiple requests because there becomes
	// just too many parameters for the URL to handle
	let batch_limit = 215;
	let version_ids: Vec<_> = project_needed_versions;

	let chunks = version_ids.chunks(batch_limit);

	// Download each chunk
	let all_versions = Arc::new(Mutex::new(Vec::new()));
	let mut tasks = tokio::task::JoinSet::new();
	for chunk in chunks {
		let chunk = chunk.to_vec();
		let client = client.clone();
		let all_versions = all_versions.clone();
		let task = async move {
			let versions = modrinth::get_multiple_versions(&chunk, &client)
				.await
				.context("Failed to get Modrinth versions")?;

			let mut lock = all_versions.lock().await;
			lock.extend(versions);

			Ok::<(), anyhow::Error>(())
		};
		tasks.spawn(task);
	}

	// Download teams at the same time
	let mut team_ids = Vec::new();
	for project in &projects {
		team_ids.push(project.project.team.clone());
	}
	let all_teams = Arc::new(Mutex::new(Vec::new()));
	{
		let client = client.clone();
		let all_teams = all_teams.clone();
		let task = async move {
			let teams = modrinth::get_multiple_teams(&team_ids, &client)
				.await
				.context("Failed to get Modrinth teams")?;
			let mut lock = all_teams.lock().await;
			*lock = teams;

			Ok::<(), anyhow::Error>(())
		};
		tasks.spawn(task);
	}

	// Run the tasks
	while let Some(result) = tasks.join_next().await {
		result?.context("Task failed")?;
	}
	let all_versions = all_versions.lock().await;
	let all_teams = all_teams.lock().await;

	// Collect the versions into a HashMap so that we can look them up when ordering them correctly
	let mut all_versions = all_versions
		.iter()
		.map(|x| (x.id.clone(), x.clone()))
		.collect::<HashMap<_, _>>();

	// Create missing placeholder files for projects that weren't in the response
	for project in project_ids {
		if !projects
			.iter()
			.any(|x| x.project.id == project || x.project.slug == project)
		{
			let path = storage_dirs.get_missing_path(&project);
			if !path.exists() {
				let file = std::fs::File::create(path);
				std::mem::drop(file);
			}
		}
	}

	// Apply new versions, and team members to the list of projects
	let project_infos: anyhow::Result<Vec<_>> = projects
		.into_iter()
		.map(|project| {
			// Combine existing and new versions
			let versions: Vec<_> = project
				.project
				.versions
				.iter()
				.filter_map(|x| {
					all_versions
						.remove(x)
						.or_else(|| project.versions.iter().find(|y| &y.id == x).cloned())
				})
				.rev()
				.collect();

			let team = all_teams
				.iter()
				.find(|x| x.iter().any(|x| x.team_id == project.project.team))
				.cloned()
				.unwrap_or_default();

			Ok(ProjectInfo {
				project: project.project,
				versions,
				members: team,
			})
		})
		.collect();

	let project_infos = project_infos?;

	// Save to cache
	for project_info in &project_infos {
		let _ = save_project_info(project_info, storage_dirs);
	}

	if download_dependencies {
		let mut all_dependencies = Vec::new();
		for project in &project_infos {
			for version in &project.versions {
				for dep in &version.dependencies {
					if let Some(project_id) = &dep.project_id
						&& !all_dependencies.contains(project_id)
					{
						all_dependencies.push(project_id.clone());
					}
				}
			}
		}

		let _ = Box::pin(download_multiple_projects(
			&all_dependencies,
			storage_dirs,
			client,
			false,
		))
		.await;
	}

	Ok(project_infos)
}

/// Saves info for a project to cache
fn save_project_info(project_info: &ProjectInfo, storage_dirs: &StorageDirs) -> anyhow::Result<()> {
	let id_path = storage_dirs.projects.join(&project_info.project.id);
	let slug_path = storage_dirs.projects.join(&project_info.project.slug);
	create_leading_dirs(&id_path)?;
	json_to_file(&id_path, &project_info)?;
	update_link(&id_path, &slug_path)?;

	Ok(())
}

/// Project data, versions, and team members for a Modrinth project
#[derive(Serialize, Deserialize)]
struct ProjectInfo {
	project: Project,
	/// Key is the version ID, value is a Version struct, serialized as a string
	/// to prevent extra serde_json time
	versions: Vec<Version>,
	members: Vec<Member>,
}

impl ProjectInfo {
	/// Gets the versions of this project that are in the given version list but not the versions map
	/// (haven't been downloaded yet)
	pub fn get_missing_versions<'a>(&self, needed_versions: &'a [String]) -> Vec<&'a str> {
		needed_versions
			.iter()
			.filter(|x| !self.versions.iter().any(|y| &&y.id == x))
			.map(|x| x.as_str())
			.collect()
	}
}

fn project_needs_update(path: &Path) -> anyhow::Result<bool> {
	let meta = path.metadata()?;
	let last_update = meta.modified()?;
	let now = SystemTime::now();

	if now < last_update {
		Ok(true)
	} else {
		Ok(now.duration_since(last_update)?.as_secs() >= PROJECT_CACHE_TIME_SECS)
	}
}

/// Storage directories
#[derive(Clone)]
struct StorageDirs {
	projects: PathBuf,
	packages: PathBuf,
}

impl StorageDirs {
	fn new(data_dir: &Path) -> Self {
		let modrinth_dir = data_dir.join("internal/modrinth");
		Self {
			projects: modrinth_dir.join("projects"),
			packages: modrinth_dir.join("packages"),
		}
	}

	/// Get the placeholder path for a project that does not exist
	fn get_missing_path(&self, project_id: &str) -> PathBuf {
		self.projects.join(format!("__missing__{project_id}"))
	}
}

/// Creates InstanceConfig from an mrpack index
fn mrpack_index_to_config(index: &ModrinthIndex, side: Side) -> InstanceConfig {
	// Suppress mods that this pack provides
	let mut suppress = Vec::new();
	for file in &index.files {
		let (Some(project_id), _) = file.get_modrinth_info() else {
			continue;
		};

		suppress.push(format!("modrinth:{project_id}"));
	}

	let loader = if let Some(version) = &index.dependencies.forge {
		Some(format!("forge@{version}"))
	} else if let Some(version) = &index.dependencies.neoforge {
		Some(format!("neoforged@{version}"))
	} else if let Some(version) = &index.dependencies.fabric_loader {
		Some(format!("fabric@{version}"))
	} else {
		index
			.dependencies
			.quilt_loader
			.as_ref()
			.map(|version| format!("quilt@{version}"))
	};

	InstanceConfig {
		side: Some(side),
		name: Some(index.name.clone()),
		version: Some(MinecraftVersionDeser::Version(
			index.dependencies.minecraft.clone().into(),
		)),
		loader,
		..Default::default()
	}
}
