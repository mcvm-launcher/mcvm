use std::{
	collections::HashMap,
	fmt::Debug,
	path::{Path, PathBuf},
	sync::Arc,
};

use anyhow::{bail, Context};
use mcvm::config_crate::instance::get_addon_paths;
use mcvm_core::io::files::{create_leading_dirs_async, update_hardlink_async};
use mcvm_net::{
	download::{self, Client},
	modrinth::{self, Project, ProjectType, Version},
};
use mcvm_plugin::{api::CustomPlugin, hooks::OnInstanceSetupResult};
use mcvm_shared::{
	addon::AddonKind,
	modifications::{Modloader, ServerType},
	output::{MCVMOutput, MessageContents, MessageLevel},
	versions::{parse_versioned_string, VersionPattern},
	UpdateDepth,
};
use tokio::{
	sync::{mpsc::Sender, Mutex},
	task::JoinSet,
};

fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::from_manifest_file("modrinth", include_str!("plugin.json"))?;

	plugin.on_instance_setup(|mut ctx, arg| {
		if arg.update_depth < UpdateDepth::Full {
			return Ok(OnInstanceSetupResult::default());
		}

		let storage_dir = ctx
			.get_data_dir()
			.context("Failed to get data dir")?
			.join("internal/modrinth/projects");

		let game_dir = PathBuf::from(arg.game_dir);

		let mut all_dirs = Vec::new();
		let addon_kinds = [
			AddonKind::Datapack,
			AddonKind::Mod,
			AddonKind::Plugin,
			AddonKind::ResourcePack,
			AddonKind::Shader,
		];
		for addon_kind in &addon_kinds {
			let dirs = get_addon_paths(&arg.config, &game_dir, *addon_kind, &[], &arg.version_info)
				.context("Failed to get instance paths for addon")?;
			all_dirs.extend(dirs);
		}

		let requested_projects = arg.config.common.plugin_config.get("modrinth_projects");
		// If requested projects is not present, clear old projects immediately instead of later so that they still get removed
		// even if you just delete your modrinth_projects field
		if requested_projects.is_none() {
			for dir in &all_dirs {
				clear_dir(dir).context("Failed to clear existing projects in addon directory")?;
			}
		}

		let Some(requested_projects) = requested_projects else {
			return Ok(OnInstanceSetupResult::default());
		};
		let requested_projects: Vec<String> = serde_json::from_value(requested_projects.clone())
			.context("Requested Modrinth projects were not formatted correctly")?;

		// Figure out which Modrinth loader we are using
		let modloader =
			Modloader::from_client_and_server_type(arg.client_type, arg.server_type.clone());

		ctx.get_output().display(
			MessageContents::Header("Updating Modrinth projects".into()),
			MessageLevel::Important,
		);
		let mut section = ctx.get_output().get_section();

		let runtime = tokio::runtime::Runtime::new()?;

		let mut process = section.get_process();
		process.display(
			MessageContents::StartProcess("Getting project info and resolving dependencies".into()),
			MessageLevel::Important,
		);

		// Collect all the projects we need to download by walking through dependencies
		let projects = Arc::new(Mutex::new(HashMap::new()));
		let (to_evaluate_sender, mut to_evaluate_receiver) =
			tokio::sync::mpsc::channel::<OptionalProjectReference>(requested_projects.len() + 10);

		// Add the initial projectages
		for project in requested_projects {
			let (id, version) = parse_versioned_string(&project);
			let version = if version == VersionPattern::Any {
				None
			} else {
				Some(version)
			};
			to_evaluate_sender
				.blocking_send(OptionalProjectReference {
					id: id.to_string(),
					version,
				})
				.expect("Failed to send to channel");
		}

		let mut task_set = JoinSet::new();
		let client = Client::new();
		// Run through all the tasks
		runtime.block_on(async {
			loop {
				if task_set.is_empty() && to_evaluate_receiver.is_empty() {
					break;
				}

				let task = to_evaluate_receiver.try_recv();
				if let Ok(task) = task {
					eval_project(
						task.clone(),
						&projects,
						&to_evaluate_sender,
						arg.version_info.version.clone(),
						modloader.clone(),
						arg.server_type.clone(),
						&mut task_set,
						&client,
					)
					.with_context(|| format!("Failed to evaluate project '{}'", task.id))?;
				}

				if let Some(result) = task_set.try_join_next() {
					result??;
				}
			}

			Ok::<(), anyhow::Error>(())
		})?;

		process.display(
			MessageContents::Success("Dependencies resolved".into()),
			MessageLevel::Important,
		);
		std::mem::drop(process);

		let mut process = section.get_process();
		process.display(
			MessageContents::StartProcess("Downloading projects".into()),
			MessageLevel::Important,
		);

		// Now we actually download all of the projects

		// Clear the existing projects
		for dir in &all_dirs {
			clear_dir(dir).context("Failed to clear existing projects in directory")?;
		}

		let instance_config = Arc::new(arg.config);
		let version_info = Arc::new(arg.version_info);

		runtime.block_on(async move {
			let mut task_set = JoinSet::new();
			for (
				_,
				ProjectWithVersions {
					project,
					versions,
					available_versions,
				},
			) in Arc::try_unwrap(projects)
				.expect("All tasks should be done")
				.into_inner()
			{
				let game_dir = game_dir.clone();
				let storage_dir = storage_dir.clone();
				let client = client.clone();
				let instance_config = instance_config.clone();
				let version_info = version_info.clone();
				task_set.spawn(async move {
					let latest_version_name = available_versions
						.last()
						.context("Project versions empty")?;
					let latest_version = versions
						.iter()
						.find(|x| &x.id == latest_version_name)
						.context("Version does not exist")?;

					let addon_kind = match project.project_type {
						ProjectType::Mod => AddonKind::Mod,
						ProjectType::Datapack => AddonKind::Datapack,
						ProjectType::Modpack => bail!("Modpacks are not supported by this plugin"),
						ProjectType::Plugin => AddonKind::Plugin,
						ProjectType::ResourcePack => AddonKind::ResourcePack,
						ProjectType::Shader => AddonKind::Shader,
					};

					let download_url = &latest_version
						.get_primary_download()
						.context("No downloads available")?
						.url;

					let filename = format!("{latest_version_name}{}", addon_kind.get_extension());
					let path = storage_dir.join(&project.id).join(&filename);

					if !path.exists() {
						let _ = create_leading_dirs_async(&path).await;
						download::file(download_url, &path, &client)
							.await
							.with_context(|| {
								format!("Failed to download file for project '{}'", project.id)
							})?;
					}

					let dirs = get_addon_paths(
						&instance_config,
						&game_dir,
						addon_kind,
						&[],
						&version_info,
					)
					.context("Failed to get addon target directories")?;

					for target_path in dirs {
						let target_path = target_path.join(format!(
							"modrinth_mcvm_{}_{latest_version_name}{}",
							project.id,
							addon_kind.get_extension(),
						));
						let _ = create_leading_dirs_async(&target_path).await;
						update_hardlink_async(&path, &target_path)
							.await
							.context("Failed to update hardlink")?;
					}

					Ok::<(), anyhow::Error>(())
				});
			}

			while let Some(task) = task_set.join_next().await {
				task??;
			}

			Ok::<(), anyhow::Error>(())
		})?;

		process.display(
			MessageContents::Success("Projects downloaded".into()),
			MessageLevel::Important,
		);
		std::mem::drop(process);

		section.display(
			MessageContents::Success("Modrinth projects updated".into()),
			MessageLevel::Important,
		);

		Ok(OnInstanceSetupResult::default())
	})?;

	Ok(())
}

/// Read a project and add it's dependencies to the list
fn eval_project(
	project: OptionalProjectReference,
	projects: &Arc<Mutex<HashMap<String, ProjectWithVersions>>>,
	to_evaluate: &Sender<OptionalProjectReference>,
	minecraft_version: String,
	modloader: Option<Modloader>,
	server_type: ServerType,
	task_set: &mut JoinSet<anyhow::Result<()>>,
	client: &Client,
) -> anyhow::Result<()> {
	let projects = projects.clone();
	let to_evaluate = to_evaluate.clone();
	let client = client.clone();
	task_set.spawn(async move {
		let mut lock = projects.lock().await;
		let mut is_first_download = false;
		let project_data = if let Some(data) = lock.get_mut(&project.id) {
			data
		} else {
			let project_data = modrinth::get_project(&project.id, &client)
				.await
				.with_context(|| format!("Failed to download project '{}'", project.id))?;
			let versions = modrinth::get_multiple_versions(&project_data.versions, &client)
				.await
				.with_context(|| {
					format!("Failed to download versions for project '{}'", project.id)
				})?;

			let available_versions: Vec<_> = versions
				.iter()
				.filter_map(|x| {
					if !x.game_versions.contains(&minecraft_version) {
						return None;
					}
					if !x.loaders.iter().any(|x| {
						let matches_modloader = if let Some(modloader) = &modloader {
							x.matches_modloader(modloader)
						} else {
							true
						};
						matches_modloader || x.matches_plugin_loader(&server_type)
					}) {
						return None;
					}
					Some(x.name.clone())
				})
				.collect();

			lock.insert(
				project.id.clone(),
				ProjectWithVersions {
					project: project_data,
					versions,
					available_versions,
				},
			);
			is_first_download = true;
			lock.get_mut(&project.id).expect("Just inserted")
		};

		// Figure out which version we want to use

		// We can check this here because this will only be empty now if the Minecraft versions don't match
		if project_data.available_versions.is_empty() {
			bail!(
				"No versions were found for project '{}' that matched the Minecraft version",
				project.id
			);
		}

		let old_best_version = project_data
			.available_versions
			.last()
			.expect("Should not be empty")
			.clone();

		if let Some(requested_version) = project.version {
			let new_versions = requested_version.get_matches(&project_data.available_versions);
			// We have removed all possible versions
			if new_versions.is_empty() {
				bail!(
					"No valid versions of project '{}' could be found",
					project.id
				);
			}
			project_data.available_versions = new_versions;
		}

		let version_name = project_data
			.available_versions
			.last()
			.expect("Should not be empty");
		let version = project_data
			.versions
			.iter()
			.find(|x| &x.id == version_name)
			.expect("Should be one of the versions from the list");

		// Evaluate dependencies only if the version has changed
		if is_first_download || &old_best_version == version_name {
			for dep in &version.dependencies {
				to_evaluate
					.send(OptionalProjectReference {
						id: dep.project_id.clone(),
						version: dep.version_id.clone().map(VersionPattern::Single),
					})
					.await
					.context("Failed to send value")?;
			}
		}

		Ok::<(), anyhow::Error>(())
	});

	Ok(())
}

/// A project and its available versions
struct ProjectWithVersions {
	project: Project,
	versions: Vec<Version>,
	available_versions: Vec<String>,
}

impl Debug for ProjectWithVersions {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "ProjectWithVersions")
	}
}

/// Reference to a project and optionally a version
#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct OptionalProjectReference {
	pub id: String,
	pub version: Option<VersionPattern>,
}

/// Clears datapacks or resource packs from a directory that were downloaded by Modrinth
fn clear_dir(dir: &Path) -> anyhow::Result<()> {
	for entry in dir.read_dir().context("Failed to read directory")? {
		let entry = entry?;
		if entry.file_type()?.is_dir() {
			continue;
		}
		if entry
			.file_name()
			.to_string_lossy()
			.to_string()
			.starts_with("modrinth_mcvm")
		{
			std::fs::remove_file(entry.path()).context("Failed to remove Modrinth project")?;
		}
	}

	Ok(())
}
