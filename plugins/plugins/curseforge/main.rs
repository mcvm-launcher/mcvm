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
use nitro_instance::addon::modpack::cfpack::{CurseForgeManifest, CurseForgePack};
use nitro_net::{
	curseforge::{self, CurseFile, CurseMod, SearchModsResponse},
	download::Client,
};
use nitro_pkg::{PackageMetaAndProps, PackageSearchResults, PkgRequest, PkgRequestSource};
use nitro_plugin::{
	api::{executable::ExecutablePlugin, utils::PackageSearchCache},
	hook::hooks::{CustomRepoQueryResult, ImportInstanceResult, InstallModpackResult},
};
use nitro_shared::{
	Side,
	io::{config::IO_CONFIG, update_link},
	output::{MessageContents, NitroOutput},
	versions::{MinecraftVersionDeser, VersionPattern},
};
use nitrolaunch::config_crate::instance::InstanceConfig;
use serde::{Deserialize, Serialize};

const PROJECT_CACHE_TIME_SECS: u64 = 3600;

fn main() -> anyhow::Result<()> {
	let mut plugin =
		ExecutablePlugin::from_manifest_file("curseforge", include_str!("plugin.json"))?;

	plugin.query_custom_package_repository(|ctx, arg| {
		if arg.repository != "curse" {
			return Ok(None);
		}

		let data_dir = ctx.get_data_dir()?;
		let storage_dirs = StorageDirs::new(&data_dir);

		let runtime = tokio::runtime::Runtime::new()?;
		let client = Client::new();

		runtime.block_on(query_package(&arg.package, &client, &storage_dirs))
	})?;

	plugin.preload_packages(|ctx, arg| {
		if arg.repository != "curse" {
			return Ok(());
		}

		let data_dir = ctx.get_data_dir()?;
		let storage_dirs = StorageDirs::new(&data_dir);

		let runtime = tokio::runtime::Runtime::new()?;
		let client = Client::new();

		runtime.block_on(async move {
			let mut tasks = tokio::task::JoinSet::new();
			for package in arg.packages {
				let client = client.clone();
				let storage_dirs = storage_dirs.clone();

				tasks.spawn(async move { query_package(&package, &client, &storage_dirs).await });
			}

			while let Some(task) = tasks.join_next().await {
				let _ = task??;
			}

			Ok::<(), anyhow::Error>(())
		})?;

		Ok(())
	})?;

	plugin.search_custom_package_repository(|ctx, arg| {
		if arg.repository != "curse" {
			return Ok(PackageSearchResults::default());
		}

		let api_key = get_api_key()?;
		let client = Client::new();
		let runtime = tokio::runtime::Runtime::new()?;

		let data_dir = ctx.get_data_dir()?;

		let (projects, previews, total_results) = runtime.block_on(async move {
			let cache_path = data_dir.join("internal/curseforge/search_cache.json");
			create_leading_dirs(&cache_path)?;
			let mut search_cache =
				PackageSearchCache::open(cache_path, 250).context("Failed to open search cache")?;

			let results = if let Some(results) =
				search_cache.check::<SearchModsResponse>(&arg.parameters)
			{
				results
			} else {
				let results = curseforge::search_mods(arg.parameters.clone(), &api_key, &client)
					.await
					.context("Failed to search projects from the API")?;

				let _ = search_cache.write(&arg.parameters, results.clone());
				results
			};

			let mut previews = HashMap::with_capacity(results.data.len());
			let mut projects = Vec::with_capacity(results.data.len());
			for result in results.data {
				let req = PkgRequest {
					source: PkgRequestSource::UserRequire,
					id: result.id.to_string().into(),
					content_version: VersionPattern::Any,
					repository: Some("curse".into()),
					slug: Some(result.slug.clone()),
				};
				let req_str = req.to_string();

				projects.push(req_str.clone());
				let package =
					nitro_pkg_gen::curse::generate(result, None, Vec::new(), Some("curse")).await;
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

			Ok::<_, anyhow::Error>((projects, previews, results.pagination.total_count as usize))
		})?;

		Ok(PackageSearchResults {
			results: projects,
			total_results,
			previews,
		})
	})?;

	plugin.sync_custom_package_repository(|ctx, arg| {
		if arg.repository != "curse" {
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
			Some(CurseForgePack::from_stream(file).context("Failed to open old modpack")?)
		} else {
			None
		};

		let data_dir = ctx.get_data_dir()?;
		let addons_dir = data_dir.join("internal/addons");
		let api_key = get_api_key().context("Failed to get CurseForge API key")?;

		let file = BufReader::new(File::open(arg.path).context("Failed to open modpack")?);
		let mut pack = CurseForgePack::from_stream(file).context("Failed to open modpack")?;

		let mut process = ctx.get_output().get_process();
		process.display(MessageContents::StartProcess(
			"Downloading modpack files".into(),
		));

		let client = Client::new();
		let runtime = tokio::runtime::Runtime::new()?;

		let (projects, files) = runtime
			.block_on(pack.download(&addons_dir, &client, &api_key))
			.context("Failed to download modpack files")?;

		process.display(MessageContents::Success("Modpack files downloaded".into()));
		process.finish();

		let mut process = ctx.get_output().get_process();
		process.display(MessageContents::StartProcess("Installing modpack".into()));

		let addons = pack
			.apply(
				Path::new(&arg.target_path),
				&files,
				&projects,
				&addons_dir,
				arg.side,
				&arg.minecraft_versions,
				old_pack.as_mut(),
			)
			.context("Failed to apply modpack")?;

		process.display(MessageContents::Success("CurseForge pack installed".into()));

		let mut packages = Vec::new();
		for file in &pack.manifest().files {
			packages.push(format!("curse:{}@{}", file.project_id, file.file_id));
		}

		Ok(InstallModpackResult {
			name: pack.manifest().name.clone(),
			packages,
			addons,
		})
	})?;

	plugin.import_instance(|mut ctx, arg| {
		if arg.format != "cfpack" {
			bail!("Invalid format");
		}

		let source_path = PathBuf::from(arg.source_path);
		let target_path = PathBuf::from(arg.result_path);

		let addons_dir = ctx.get_data_dir()?.join("internal/addons");
		let api_key = get_api_key().context("Failed to get CurseForge API key")?;

		let output = ctx.get_output();

		let side = arg.side.context("Side not specified")?;

		let file = File::open(source_path).context("Failed to open pack file")?;
		let mut modpack = CurseForgePack::from_stream(file).context("Failed to open cfpack")?;

		// Download files
		let mut process = output.get_process();
		process.display(MessageContents::StartProcess("Downloading addons".into()));

		let runtime = tokio::runtime::Runtime::new()?;
		let client = Client::new();
		let (projects, files) = runtime
			.block_on(modpack.download(&addons_dir, &client, &api_key))
			.context("Failed to download modpack files")?;

		process.display(MessageContents::Success("Addons downloaded".into()));
		process.finish();

		let target_path = match side {
			Side::Client => target_path.join(".minecraft"),
			Side::Server => target_path,
		};

		let mut process = output.get_process();
		process.display(MessageContents::StartProcess("Installing modpack".into()));
		modpack
			.apply(
				&target_path,
				&files,
				&projects,
				&addons_dir,
				side,
				&arg.minecraft_versions,
				None,
			)
			.context("Failed to install modpack")?;
		process.display(MessageContents::Success("Modpack installed".into()));
		process.finish();

		let config = cfpack_manifest_to_config(modpack.manifest(), side);

		Ok(ImportInstanceResult {
			format: arg.format,
			config,
		})
	})?;

	Ok(())
}

/// Queries for a CurseForge package
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

	let mut package = nitro_pkg_gen::curse::generate(
		project_info.project,
		Some(project_info.body.clone()),
		project_info.files.clone(),
		Some("curse"),
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

/// Gets a cached CurseForge project and it's versions or downloads it
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

	let api_key = get_api_key().context("Failed to get CurseForge API key")?;

	let project_info =
		if project_path.exists() && !project_needs_update(&project_path).unwrap_or(true) {
			json_from_file(&project_path).context("Failed to read project info from file")?
		} else {
			let project_task = {
				let project = project_id.to_string();
				let client = client.clone();
				let api_key = api_key.clone();
				tokio::spawn(async move {
					curseforge::get_mod_optional(&project, &api_key, &client).await
				})
			};

			let body_task = {
				let project = project_id.to_string();
				let client = client.clone();
				let api_key = api_key.clone();
				tokio::spawn(async move {
					curseforge::get_mod_description(&project, &api_key, &client).await
				})
			};

			let files_task = {
				let project = project_id.to_string();
				let client = client.clone();
				let api_key = api_key.clone();
				tokio::spawn(
					async move { curseforge::get_mod_files(&project, &api_key, &client).await },
				)
			};

			let (project, body, files) = tokio::join!(project_task, body_task, files_task);
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

			let body = body
				.context("Failed to get project body")?
				.context("Failed to get project body")?;
			let files = files
				.context("Failed to get project files")?
				.context("Failed to get project files")?;

			let project_info = ProjectInfo {
				project,
				files,
				body,
			};

			let _ = save_project_info(&project_info, storage_dirs);

			project_info
		};

	Ok(Some(project_info))
}

/// Saves info for a project to cache
fn save_project_info(project_info: &ProjectInfo, storage_dirs: &StorageDirs) -> anyhow::Result<()> {
	let id_path = storage_dirs
		.projects
		.join(project_info.project.id.to_string());
	let slug_path = storage_dirs.projects.join(&project_info.project.slug);
	create_leading_dirs(&id_path)?;
	json_to_file(&id_path, &project_info)?;
	update_link(&id_path, &slug_path)?;

	Ok(())
}

/// Project data, files, and body for a CurseForge project
#[derive(Serialize, Deserialize)]
struct ProjectInfo {
	project: CurseMod,
	files: Vec<CurseFile>,
	body: String,
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
		let curseforge_dir = data_dir.join("internal/curseforge");
		Self {
			projects: curseforge_dir.join("projects"),
			packages: curseforge_dir.join("packages"),
		}
	}

	/// Get the placeholder path for a project that does not exist
	fn get_missing_path(&self, project_id: &str) -> PathBuf {
		self.projects.join(format!("__missing__{project_id}"))
	}
}

/// Creates InstanceConfig from a cfpack manifest
fn cfpack_manifest_to_config(manifest: &CurseForgeManifest, side: Side) -> InstanceConfig {
	// Suppress mods that this pack provides
	let mut suppress = Vec::new();
	for file in &manifest.files {
		suppress.push(format!("curse:{}", file.project_id));
	}

	let loader = if let Some(loader) = manifest.minecraft.mod_loaders.first() {
		let (id, version) = match loader.id.split_once("-") {
			Some((id, version)) => (id, Some(version)),
			None => (loader.id.as_str(), None),
		};

		let id = match id {
			"neoforge" => "neoforged",
			other => other,
		};

		if let Some(version) = version {
			Some(format!("{id}@{version}"))
		} else {
			Some(id.to_string())
		}
	} else {
		None
	};

	InstanceConfig {
		side: Some(side),
		name: Some(manifest.name.clone()),
		version: Some(MinecraftVersionDeser::Version(
			manifest.minecraft.version.clone().into(),
		)),
		loader,
		..Default::default()
	}
}

fn get_api_key() -> anyhow::Result<String> {
	IO_CONFIG
		.get_string("curseforge_api_key")
		.context("API key missing")
}
