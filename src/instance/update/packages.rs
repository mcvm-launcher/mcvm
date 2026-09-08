use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

use itertools::Itertools;
use nitro_config::package::EvalPermissions;
use nitro_core::io::files::create_leading_dirs;
use nitro_core::net::get_transfer_limit;
use nitro_instance::addon::get_addon_dirs;
use nitro_instance::lock::InstanceLockfile;
use nitro_pkg::repo::PackageFlag;
use nitro_pkg::{PkgRequest, PkgRequestSource};
use nitro_shared::minecraft::AddonKind;
use nitro_shared::output::{MessageContents, NitroOutput};
use nitro_shared::pkg::{ArcPkgReq, PackageDiff, PackageStability, merge_package_lists};
use nitro_shared::util::OS_STRING;
use nitro_shared::versions::{VersionInfo, VersionPattern};
use nitro_shared::{UpdateDepth, manual_files, translate};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::addon::{AddonExt, AddonRequest};
use crate::config::package::PackageConfig;
use crate::instance::Instance;
use crate::pkg::eval::{EvalConstants, EvalParameters, ResolutionAndEvalResult, resolve};
use crate::util::select_random_n_items_from_list;

use super::InstanceUpdateContext;

use anyhow::{Context, bail};

/// Install packages on an instance. Returns a set of all unique packages
pub async fn update_instance_packages<O: NitroOutput>(
	instance: &mut Instance,
	constants: &Arc<EvalConstants>,
	mc_version: String,
	depth: UpdateDepth,
	ctx: &mut InstanceUpdateContext<'_, O>,
) -> anyhow::Result<HashSet<ArcPkgReq>> {
	let mut inst_lock = instance.get_lockfile(ctx.paths)?;

	// Check for updates
	let Some(packages) = get_included_packages(instance, &inst_lock, depth) else {
		return Ok(HashSet::new());
	};

	// Resolve dependencies
	ctx.output.start_process();
	ctx.output.display(MessageContents::StartProcess(translate!(
		ctx.output,
		StartResolvingDependencies
	)));
	let resolution = resolve_instance(instance, &packages, constants, ctx)
		.await
		.context("Failed to resolve dependencies for instance")?;
	ctx.output.display(MessageContents::Success(translate!(
		ctx.output,
		FinishResolvingDependencies
	)));
	ctx.output.end_process();

	// Prompt to update the packages
	let current_packages = inst_lock.get_packages();
	let mut diffs = resolution.get_diffs(current_packages);

	// Make requests displayable
	for diff in &mut diffs {
		match diff {
			PackageDiff::Added(req)
			| PackageDiff::Removed(req)
			| PackageDiff::VersionChanged(req, ..) => {
				*req = ctx
					.packages
					.make_req_displayable(req, ctx.paths, ctx.client, ctx.output)
					.await
			}
			PackageDiff::ManyAdded(..) | PackageDiff::ManyRemoved(..) => {}
		}
	}

	if !diffs.is_empty() && !ctx.output.prompt_special_package_diffs(diffs).await? {
		bail!("Package update aborted");
	}

	let version_info = VersionInfo {
		version: mc_version.clone(),
		versions: constants.version_list.clone(),
	};

	remove_existing_addons(instance, &version_info)?;

	// Evaluate first to install all of the addons
	ctx.output.display(MessageContents::Header(translate!(
		ctx.output,
		StartAcquiringAddons
	)));
	let mut addons = Vec::new();
	for package in resolution.packages.iter().sorted_by_key(|x| x.req.clone()) {
		addons.extend(package.eval.addon_reqs.clone());

		// Check the package to display warnings
		check_package(ctx, &package.req)
			.await
			.with_context(|| format!("Failed to check package {}", package.req))?;

		// Display any notices from the installation
		for notice in &package.eval.notices {
			ctx.output.display(format_package_update_message(
				&package.req,
				MessageContents::Notice(notice.clone()),
			));
		}
	}

	// Run the acquire tasks
	install_addons(addons, &instance.id, depth, ctx)
		.await
		.context("Failed to install addons")?;

	ctx.output.display(MessageContents::Success(translate!(
		ctx.output,
		FinishAcquiringAddons
	)));

	// Install each package one after another onto all of its instances
	ctx.output.start_process();
	ctx.output.display(MessageContents::Header(translate!(
		ctx.output,
		StartInstallingPackages
	)));

	for package in &resolution.packages {
		instance
			.install_eval_data(
				&package.req,
				&package.eval,
				&version_info,
				ctx.paths,
				&mut inst_lock,
				ctx.output,
			)
			.await
			.context("Failed to install package on instance")?;
	}

	// Remove unused packages and addons
	let used_package_reqs = resolution
		.packages
		.iter()
		.map(|x| x.req.clone())
		.collect::<Vec<_>>();
	let addons_to_remove = inst_lock
		.remove_unused_packages(&used_package_reqs)
		.context("Failed to remove unused packages")?;
	for addon in addons_to_remove {
		let _ = addon.remove_from_instance();
	}

	inst_lock.update_configured_packages(
		instance
			.packages
			.iter()
			.map(|x| x.req.to_string())
			.collect(),
	);
	inst_lock.write()?;

	ctx.output.display(MessageContents::Success(translate!(
		ctx.output,
		FinishInstallingPackages,
		"count" = &resolution.packages.len().to_string()
	)));
	ctx.output.end_process();

	// Get the set of unique packages
	let out = HashSet::from_iter(resolution.packages.into_iter().map(|x| x.req));

	Ok(out)
}

/// Evaluates addon acquire tasks and manual downloads efficiently
async fn install_addons<O: NitroOutput>(
	addons: Vec<AddonRequest>,
	instance_id: &str,
	depth: UpdateDepth,
	ctx: &mut InstanceUpdateContext<'_, O>,
) -> anyhow::Result<()> {
	let mut tasks = HashMap::new();
	let mut manual_addons = Vec::new();
	for addon in addons {
		if !addon.addon.should_update(ctx.paths, instance_id) && depth != UpdateDepth::Force {
			continue;
		}

		if addon.addon.is_manual {
			manual_addons.push(addon);
		} else {
			let task = addon.get_acquire_task(ctx.paths, instance_id, ctx.client);
			tasks.insert(addon.get_unique_id(instance_id), task);
		}
	}

	let total_count = tasks.len();
	let mut task_set = JoinSet::new();

	let sem = Arc::new(Semaphore::new(get_transfer_limit()));
	for task in tasks.into_values() {
		let permit = sem.clone().acquire_owned().await;
		let task = async move {
			let _permit = permit?;

			task.await
		};
		task_set.spawn(task);
	}

	if !task_set.is_empty() {
		let mut process = ctx.output.get_process();
		while let Some(result) = task_set.join_next().await {
			result
				.context("Failed to run addon acquire task")?
				.context("Failed to acquire addon")?;

			// Update progress bar
			let progress = MessageContents::Progress {
				current: (total_count - task_set.len()) as u32,
				total: total_count as u32,
			};

			process.display(progress);
		}
	}

	if !manual_addons.is_empty() {
		let manual_files = manual_addons
			.iter()
			.filter_map(|x| x.manual_file())
			.collect();
		ctx.output.prompt_special_manual_files(manual_files).await?;

		let manual_dir = manual_files::get_scan_dir_from_os(OS_STRING);
		for addon in manual_addons {
			let src = manual_dir.join(&addon.addon.file_name);
			let dest = addon.addon.get_path(ctx.paths, instance_id);
			let _ = create_leading_dirs(&dest);
			std::fs::rename(src, dest).context("Failed to move manual file")?;
		}
	}

	Ok(())
}

/// Resolve packages on an instance
async fn resolve_instance<O: NitroOutput>(
	instance: &mut Instance,
	packages: &[PackageConfig],
	constants: &Arc<EvalConstants>,
	ctx: &mut InstanceUpdateContext<'_, O>,
) -> anyhow::Result<ResolutionAndEvalResult> {
	let mut params = EvalParameters::new(instance.kind.to_side());
	params.stability = instance.config.package_stability.unwrap_or_default();

	let mut overrides = instance.config.overrides.clone();
	overrides.suppress = merge_package_lists(overrides.suppress.into_iter(), &constants.suppress);

	let resolution = resolve(
		packages,
		&instance.id,
		constants.clone(),
		params,
		overrides,
		ctx.paths,
		ctx.packages,
		ctx.client,
		ctx.output,
	)
	.await
	.with_context(|| {
		format!(
			"Failed to resolve package dependencies for instance '{}'",
			instance.id
		)
	})?;

	Ok(resolution)
}

/// Gets the inputs for the package resolver, based on whether we are doing a full or partial update
fn get_included_packages(
	instance: &Instance,
	inst_lock: &InstanceLockfile,
	depth: UpdateDepth,
) -> Option<Vec<PackageConfig>> {
	match depth {
		UpdateDepth::Full | UpdateDepth::Force => Some(instance.packages.clone()),
		UpdateDepth::Shallow => {
			// We only want to update new or removed packages, and only if we have to
			if inst_lock.get_configured_packages()
				== &instance
					.packages
					.iter()
					.map(|x| x.req.to_string())
					.collect::<HashSet<_>>()
			{
				return None;
			}

			// Add locked packages as soft requirements
			let mut out = Vec::new();
			for (pkg, data) in inst_lock.get_packages() {
				let mut req = PkgRequest::parse(pkg, PkgRequestSource::UserRequire);
				let mut pkg = instance
					.packages
					.iter()
					.find(|x| *x.req == req)
					.cloned()
					.unwrap_or_else(|| PackageConfig {
						req: req.clone().arc(),
						features: Vec::new(),
						use_default_features: true,
						permissions: EvalPermissions::default(),
						stability: PackageStability::default(),
						worlds: Vec::new(),
						content_version: None,
						optional: false,
					});

				req.content_version = data
					.content_version
					.clone()
					.map(VersionPattern::Prefer)
					.unwrap_or_default();
				pkg.req = req.clone().arc();
				out.push(pkg);
			}

			// Add any newly configured packages
			for pkg in &instance.packages {
				out.retain(|x| x.req != pkg.req);
				out.push(pkg.clone());
			}

			// Remove any soft constraints for packages that are no longer configured
			for pkg_id in inst_lock.get_configured_packages() {
				let req = PkgRequest::parse(pkg_id, PkgRequestSource::UserRequire);
				if !instance.packages.iter().any(|x| *x.req == req) {
					out.retain(|x| *x.req != req);
				}
			}

			Some(out)
		}
	}
}

/// Removes existing addons on an instance just in case there are lockfile issues
fn remove_existing_addons(
	instance: &mut Instance,
	version_info: &VersionInfo,
) -> anyhow::Result<()> {
	let addon_kinds = [
		AddonKind::Datapack,
		AddonKind::Mod,
		AddonKind::Plugin,
		AddonKind::ResourcePack,
		AddonKind::Shader,
	];

	instance.ensure_dir()?;

	for adddon_kind in addon_kinds {
		let Some(inst_dir) = &instance.dir else {
			continue;
		};

		let dirs = get_addon_dirs(
			adddon_kind,
			instance.side(),
			inst_dir,
			&[],
			instance.config.datapack_folder.as_ref().map(Path::new),
			version_info,
		);
		for dir in dirs {
			remove_nitro_addons(&dir);
		}
	}

	Ok(())
}

/// Checks a package with the registry to report any warnings about it
async fn check_package<O: NitroOutput>(
	ctx: &mut InstanceUpdateContext<'_, O>,
	pkg: &ArcPkgReq,
) -> anyhow::Result<()> {
	let package = ctx
		.packages
		.get(pkg, ctx.paths, ctx.client, ctx.output)
		.await?;

	if package.flags.contains(&PackageFlag::OutOfDate) {
		ctx.output.display(MessageContents::Warning(translate!(
			ctx.output,
			PackageOutOfDate,
			"pkg" = &pkg.id
		)));
	}

	if package.flags.contains(&PackageFlag::Deprecated) {
		ctx.output.display(MessageContents::Warning(translate!(
			ctx.output,
			PackageDeprecated,
			"pkg" = &pkg.id
		)));
	}

	if package.flags.contains(&PackageFlag::Insecure) {
		ctx.output.display(MessageContents::Error(translate!(
			ctx.output,
			PackageInsecure,
			"pkg" = &pkg.id
		)));
	}

	if package.flags.contains(&PackageFlag::Malicious) {
		ctx.output.display(MessageContents::Error(translate!(
			ctx.output,
			PackageMalicious,
			"pkg" = &pkg.id
		)));
	}

	Ok(())
}

/// Prints support messages about installed packages when updating
pub async fn print_package_support_messages<O: NitroOutput>(
	packages: &[ArcPkgReq],
	ctx: &mut InstanceUpdateContext<'_, O>,
) -> anyhow::Result<()> {
	let package_count = 5;
	let packages = select_random_n_items_from_list(packages, package_count);
	let mut links = Vec::new();
	for req in packages {
		let package = ctx
			.packages
			.get(req, ctx.paths, ctx.client, ctx.output)
			.await?;
		if let Some(link) = package
			.get_metadata(ctx.paths, ctx.client)
			.await?
			.support_link
			.clone()
		{
			links.push((req, link))
		}
	}
	if !links.is_empty() {
		ctx.output.display(MessageContents::Header(translate!(
			ctx.output,
			PackageSupportHeader
		)));
		for (req, link) in links {
			let msg = format_package_update_message(req, MessageContents::Hyperlink(link));
			ctx.output.display(msg);
		}
	}

	Ok(())
}

/// Creates the output message for package installation when updating an instance
fn format_package_update_message(pkg: &PkgRequest, message: MessageContents) -> MessageContents {
	MessageContents::ListItem(Box::new(MessageContents::Package(
		pkg.to_owned(),
		Box::new(message),
	)))
}

/// Removes Nitrolaunch-like addons from a directory
fn remove_nitro_addons(dir: &Path) {
	let Ok(dir) = dir.read_dir() else {
		return;
	};

	for entry in dir {
		let Ok(entry) = entry else {
			continue;
		};

		let filename = entry.file_name().to_string_lossy().to_string();
		if filename.starts_with("nitro_") && filename.contains("addon") {
			let _ = std::fs::remove_file(entry.path());
		}
	}
}
