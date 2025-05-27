use std::{
	collections::{HashMap, HashSet},
	path::{Path, PathBuf},
	sync::Arc,
};

use anyhow::{bail, Context};
use mcvm::config_crate::instance::get_addon_paths;
use mcvm_core::io::{
	files::{create_leading_dirs, create_leading_dirs_async, update_hardlink_async},
	json_from_file, json_to_file,
};
use mcvm_net::{
	download::{self, Client},
	smithed::{self, Pack},
};
use mcvm_pkg_gen::relation_substitution::RelationSubMethod;
use mcvm_plugin::{
	api::CustomPlugin,
	hooks::{CustomRepoQueryResult, OnInstanceSetupResult},
};
use mcvm_shared::{
	addon::AddonKind,
	output::{MCVMOutput, MessageContents, MessageLevel},
	versions::{parse_versioned_string, VersionPattern},
	UpdateDepth,
};
use tokio::{
	sync::{mpsc::Sender, Mutex},
	task::JoinSet,
};

fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::from_manifest_file("smithed", include_str!("plugin.json"))?;

	plugin.on_instance_setup(|mut ctx, arg| {
		if arg.update_depth < UpdateDepth::Full {
			return Ok(OnInstanceSetupResult::default());
		}

		let storage_dir = ctx
			.get_data_dir()
			.context("Failed to get data dir")?
			.join("internal/smithed/packs");

		let game_dir = PathBuf::from(arg.game_dir);

		let datapack_dirs = get_addon_paths(
			&arg.config,
			&game_dir,
			AddonKind::Datapack,
			&[],
			&arg.version_info,
		)
		.context("Failed to get instance paths for datapacks")?;
		let resource_pack_dirs = get_addon_paths(
			&arg.config,
			&game_dir,
			AddonKind::ResourcePack,
			&[],
			&arg.version_info,
		)
		.context("Failed to get instance paths for datapacks")?;

		let requested_packs = arg.config.common.plugin_config.get("smithed_packs");
		let requested_bundles = arg.config.common.plugin_config.get("smithed_bundles");
		// If requested packs is not present, clear old packs immediately instead of later so that they still get removed
		// even if you just delete your smithed_packs field
		if requested_packs.is_none() && requested_bundles.is_none() {
			for dir in &datapack_dirs {
				clear_dir(dir).context("Failed to clear existing packs in datapack directory")?;
			}
			for dir in &resource_pack_dirs {
				clear_dir(dir)
					.context("Failed to clear existing packs in resource pack directory")?;
			}
		}

		let client = Client::new();
		let runtime = tokio::runtime::Runtime::new()?;

		let mut all_requested_packs = Vec::new();
		if let Some(requested_packs) = requested_packs {
			let requested_packs: Vec<String> = serde_json::from_value(requested_packs.clone())
				.context("Requested Smithed packs were not formatted correctly")?;

			all_requested_packs.extend(requested_packs.into_iter().map(|x| {
				let (id, version) = parse_versioned_string(&x);
				let version = if version == VersionPattern::Any {
					None
				} else {
					Some(version)
				};

				OptionalPackReference {
					id: id.to_string(),
					version,
				}
			}));
		}
		// Add bundles
		if let Some(requested_bundles) = requested_bundles {
			let requested_bundles: Vec<String> = serde_json::from_value(requested_bundles.clone())
				.context("Requested Smithed packs were not formatted correctly")?;

			let result = runtime.block_on(async {
				let mut task_set = JoinSet::new();
				for bundle_id in requested_bundles {
					let client = client.clone();
					let minecraft_version = arg.version_info.version.clone();
					task_set.spawn(async move {
						let (bundle_id, version) = parse_versioned_string(&bundle_id);

						let bundle = smithed::get_bundle(&bundle_id, &client)
							.await
							.context("Failed to download bundle from Smithed API")?;

						let versions: Vec<_> = bundle
							.versions
							.iter()
							.filter_map(|x| {
								if x.supports.contains(&minecraft_version) {
									Some(x.name.clone())
								} else {
									None
								}
							})
							.collect();
						let versions = version.get_matches(&versions);
						let version = versions.last().with_context(|| {
							format!("Failed to find a valid version for the bundle '{bundle_id}'")
						})?;
						let version = bundle
							.versions
							.into_iter()
							.find(|x| x.name == *version)
							.expect("Should exist");

						Ok::<_, anyhow::Error>(version.packs)
					});
				}

				let mut out = Vec::new();

				while let Some(result) = task_set.join_next().await {
					let result = result??;
					out.extend(result.into_iter().map(|x| OptionalPackReference {
						id: x.id,
						version: Some(VersionPattern::Single(x.version)),
					}));
				}

				Ok::<_, anyhow::Error>(out)
			})?;

			all_requested_packs.extend(result);
		}

		ctx.get_output().display(
			MessageContents::Header("Updating Smithed packs".into()),
			MessageLevel::Important,
		);
		let mut section = ctx.get_output().get_section();

		let mut process = section.get_process();
		process.display(
			MessageContents::StartProcess("Getting pack info and resolving dependencies".into()),
			MessageLevel::Important,
		);

		// Collect all the packs we need to download by walking through dependencies
		let packs = Arc::new(Mutex::new(HashMap::new()));
		let (to_evaluate_sender, mut to_evaluate_receiver) =
			tokio::sync::mpsc::channel::<OptionalPackReference>(all_requested_packs.len() + 10);

		// Add the initial packages
		for pack in all_requested_packs {
			to_evaluate_sender
				.blocking_send(pack)
				.expect("Failed to send to channel");
		}

		let mut task_set = JoinSet::new();
		// Run through all the tasks
		runtime.block_on(async {
			loop {
				if task_set.is_empty() && to_evaluate_receiver.is_empty() {
					break;
				}

				let task = to_evaluate_receiver.try_recv();
				if let Ok(task) = task {
					eval_pack(
						task.clone(),
						&packs,
						&to_evaluate_sender,
						arg.version_info.version.clone(),
						&mut task_set,
						&client,
					)
					.with_context(|| format!("Failed to evaluate pack '{}'", task.id))?;
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
			MessageContents::StartProcess("Downloading packs".into()),
			MessageLevel::Important,
		);

		// Now we actually download all of the packs

		// Clear the existing packs
		for dir in &datapack_dirs {
			clear_dir(dir).context("Failed to clear existing packs in datapack directory")?;
		}
		for dir in &resource_pack_dirs {
			clear_dir(dir).context("Failed to clear existing packs in resource pack directory")?;
		}

		runtime.block_on(async move {
			let mut task_set = JoinSet::new();
			for (_, PackWithVersions { pack, versions }) in Arc::try_unwrap(packs)
				.expect("All tasks should be done")
				.into_inner()
			{
				let storage_dir = storage_dir.clone();
				let client = client.clone();
				let datapack_dirs = datapack_dirs.clone();
				let resource_pack_dirs = resource_pack_dirs.clone();
				task_set.spawn(async move {
					let latest_version_name = versions.last().context("Pack versions empty")?;
					let latest_version = pack
						.versions
						.iter()
						.find(|x| &x.name == latest_version_name)
						.context("Version does not exist")?;

					if let Some(datapack_url) = &latest_version.downloads.datapack {
						let filename = format!("{latest_version_name}_datapack.zip");
						let path = storage_dir.join(&pack.id).join(&filename);

						if !path.exists() {
							let _ = create_leading_dirs_async(&path).await;
							download::file(datapack_url, &path, &client)
								.await
								.with_context(|| {
									format!("Failed to download datapack for pack '{}'", pack.id)
								})?;
						}

						for target_path in datapack_dirs {
							let target_path = target_path.join(format!(
								"smithed_mcvm_{}_{latest_version_name}.zip",
								pack.id
							));
							let _ = create_leading_dirs_async(&target_path).await;
							update_hardlink_async(&path, &target_path)
								.await
								.context("Failed to update hardlink")?;
						}
					}
					if let Some(resource_pack_url) = &latest_version.downloads.resourcepack {
						let filename = format!("{latest_version_name}_resource_pack.zip");
						let path = storage_dir.join(&pack.id).join(&filename);

						if !path.exists() {
							let _ = create_leading_dirs_async(&path).await;
							download::file(resource_pack_url, &path, &client)
								.await
								.with_context(|| {
									format!(
										"Failed to download resource pack for pack '{}'",
										pack.id
									)
								})?;
						}

						for target_path in resource_pack_dirs {
							let target_path = target_path.join(format!(
								"smithed_mcvm_{}_{latest_version_name}.zip",
								pack.id
							));
							let _ = create_leading_dirs_async(&target_path).await;
							update_hardlink_async(&path, &target_path)
								.await
								.context("Failed to update hardlink")?;
						}
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
			MessageContents::Success("Packs downloaded".into()),
			MessageLevel::Important,
		);
		std::mem::drop(process);

		section.display(
			MessageContents::Success("Smithed packs updated".into()),
			MessageLevel::Important,
		);

		Ok(OnInstanceSetupResult::default())
	})?;

	plugin.query_custom_package_repository(|ctx, arg| {
		if arg.repository != "smithed" {
			return Ok(None);
		}

		let runtime = tokio::runtime::Runtime::new()?;
		let client = Client::new();

		let storage_dir = ctx
			.get_data_dir()
			.context("Failed to get data dir")?
			.join("internal/smithed/packs");
		let pack = get_cached_pack(&arg.package, &storage_dir, &client, &runtime)
			.context("Failed to get pack")?;
		let Some(pack) = pack else {
			return Ok(None);
		};

		let relation_sub_function = {
			let client = client.clone();
			let storage_dir = storage_dir.clone();
			let runtime = tokio::runtime::Runtime::new()?;

			move |relation: &str| {
				let pack = get_cached_pack(relation, &storage_dir, &client, &runtime)
					.context("Failed to get pack")?
					.context("Pack does not exist")?;

				Ok(pack.id)
			}
		};

		let package = mcvm_pkg_gen::smithed::gen(
			pack,
			RelationSubMethod::Function(Box::new(relation_sub_function)),
			&[],
		)
		.context("Failed to generate MCVM package")?;
		let package =
			serde_json::to_string_pretty(&package).context("Failed to serialized package")?;

		Ok(Some(CustomRepoQueryResult {
			contents: package,
			content_type: mcvm::pkg_crate::PackageContentType::Declarative,
			flags: HashSet::new(),
		}))
	})?;

	Ok(())
}

/// Read a pack and add it's dependencies to the list
fn eval_pack(
	pack: OptionalPackReference,
	packs: &Arc<Mutex<HashMap<String, PackWithVersions>>>,
	to_evaluate: &Sender<OptionalPackReference>,
	minecraft_version: String,
	task_set: &mut JoinSet<anyhow::Result<()>>,
	client: &Client,
) -> anyhow::Result<()> {
	let packs = packs.clone();
	let to_evaluate = to_evaluate.clone();
	let client = client.clone();
	task_set.spawn(async move {
		let mut lock = packs.lock().await;
		let mut is_first_download = false;
		let pack_data = if let Some(data) = lock.get_mut(&pack.id) {
			data
		} else {
			let pack_data = smithed::get_pack(&pack.id, &client)
				.await
				.with_context(|| format!("Failed to download pack '{}'", pack.id))?;

			let available_versions: Vec<_> = pack_data
				.versions
				.iter()
				.filter_map(|x| {
					if x.supports.contains(&minecraft_version) {
						Some(x.name.clone())
					} else {
						None
					}
				})
				.collect();

			lock.insert(
				pack.id.clone(),
				PackWithVersions {
					pack: pack_data,
					versions: available_versions,
				},
			);
			is_first_download = true;
			lock.get_mut(&pack.id).expect("Just inserted")
		};

		// Figure out which version we want to use

		// We can check this here because this will only be empty now if the Minecraft versions don't match
		if pack_data.versions.is_empty() {
			bail!(
				"No versions were found for pack '{}' that matched the Minecraft version",
				pack.id
			);
		}

		let old_best_version = pack_data
			.versions
			.last()
			.expect("Should not be empty")
			.clone();

		if let Some(requested_version) = pack.version {
			let new_versions = requested_version.get_matches(&pack_data.versions);
			// We have removed all possible versions
			if new_versions.is_empty() {
				bail!("No valid versions of pack '{}' could be found", pack.id);
			}
			pack_data.versions = new_versions;
		}

		let version_name = pack_data.versions.last().expect("Should not be empty");
		let version = pack_data
			.pack
			.versions
			.iter()
			.find(|x| &x.name == version_name)
			.expect("Should be one of the packs from the list");

		// Evaluate dependencies only if the version has changed
		if is_first_download || &old_best_version == version_name {
			for dep in &version.dependencies {
				to_evaluate
					.send(OptionalPackReference {
						id: dep.id.clone(),
						version: Some(VersionPattern::Single(dep.version.clone())),
					})
					.await
					.context("Failed to send value")?;
			}
		}

		Ok::<(), anyhow::Error>(())
	});

	Ok(())
}

/// A pack and its available versions
#[derive(Debug)]
struct PackWithVersions {
	pack: Pack,
	versions: Vec<String>,
}

/// Reference to a pack and optionally a version
#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct OptionalPackReference {
	pub id: String,
	pub version: Option<VersionPattern>,
}

/// Clears datapacks or resource packs from a directory that were downloaded by Smithed
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
			.starts_with("smithed_mcvm")
		{
			std::fs::remove_file(entry.path()).context("Failed to remove Smithed pack")?;
		}
	}

	Ok(())
}

/// Gets a cached Smithed pack or downloads it
fn get_cached_pack(
	pack: &str,
	storage_dir: &Path,
	client: &Client,
	runtime: &tokio::runtime::Runtime,
) -> anyhow::Result<Option<Pack>> {
	let pack_path = storage_dir.join(pack);
	if pack_path.exists() {
		json_from_file(&pack_path).context("Failed to read pack from file")
	} else {
		let result = runtime.block_on(smithed::get_pack_optional(pack, &client))?;

		let pack = match result {
			Some(result) => result,
			None => return Ok(None),
		};

		let _ = create_leading_dirs(&pack_path);
		// TODO: Store both the id and slug together, hardlinked to each other, to cache no matter which method is used to request
		let _ = json_to_file(&pack_path, &pack);

		Ok(Some(pack))
	}
}
