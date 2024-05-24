use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::sync::Arc;

use itertools::Itertools;
use mcvm_core::net::download::get_transfer_limit;
use mcvm_pkg::repo::PackageFlag;
use mcvm_pkg::PkgRequest;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::pkg::{ArcPkgReq, PackageID};
use mcvm_shared::translate;
use mcvm_shared::versions::VersionInfo;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::data::profile::Profile;
use crate::pkg::eval::{resolve, EvalConstants, EvalInput, EvalParameters};
use crate::util::select_random_n_items_from_list;
use mcvm_shared::id::InstanceID;

use super::ProfileUpdateContext;

use anyhow::{anyhow, Context};

/// Install packages on a profile. Returns a set of all unique packages
pub async fn update_profile_packages<'a, O: MCVMOutput>(
	profile: &mut Profile,
	constants: &EvalConstants,
	ctx: &mut ProfileUpdateContext<'a, O>,
	force: bool,
) -> anyhow::Result<HashSet<ArcPkgReq>> {
	// Resolve dependencies
	ctx.output.start_process();
	ctx.output.display(
		MessageContents::StartProcess(translate!(ctx.output, StartResolvingDependencies)),
		MessageLevel::Important,
	);
	let resolved_packages = resolve_and_batch(profile, constants, ctx)
		.await
		.context("Failed to resolve dependencies for profile")?;
	ctx.output.display(
		MessageContents::Success(translate!(ctx.output, FinishResolvingDependencies)),
		MessageLevel::Important,
	);
	ctx.output.end_process();

	// Evaluate first to install all of the addons
	ctx.output.display(
		MessageContents::StartProcess(translate!(ctx.output, StartAcquiringAddons)),
		MessageLevel::Important,
	);
	let mut tasks = HashMap::new();
	let mut evals = HashMap::new();
	for (package, package_instances) in resolved_packages
		.package_to_instances
		.iter()
		.sorted_by_key(|x| x.0)
	{
		// Check the package to display warnings
		check_package(ctx, package)
			.await
			.with_context(|| format!("Failed to check package {package}"))?;

		// Install the package on it's instances
		let mut notices = Vec::new();
		for instance_id in package_instances {
			let instance = profile.instances.get_mut(instance_id).ok_or(anyhow!(
				"Instance '{instance_id}' does not exist in the registry"
			))?;

			let mut params = EvalParameters::new(instance.kind.to_side());
			params.stability = profile.default_stability;

			let input = EvalInput { constants, params };
			let (eval, new_tasks) = instance
				.get_package_addon_tasks(
					package,
					input,
					ctx.packages,
					ctx.paths,
					force,
					ctx.client,
					ctx.plugins,
					ctx.output,
				)
				.await
				.with_context(|| {
					format!("Failed to get addon install tasks for package '{package}' on instance")
				})?;
			tasks.extend(new_tasks);

			// Add any notices to the list
			notices.extend(
				eval.notices
					.iter()
					.map(|x| (instance_id.clone(), x.to_owned())),
			);

			// Add the eval to the map
			evals.insert((package, instance_id), eval);
		}

		// Display any accumulated notices from the installation
		for (instance, notice) in notices {
			ctx.output.display(
				format_package_update_message(
					package,
					Some(&instance),
					MessageContents::Notice(notice),
				),
				MessageLevel::Important,
			);
		}
	}

	// Run the acquire tasks
	run_addon_tasks(tasks, ctx.output)
		.await
		.context("Failed to acquire addons")?;

	ctx.output.display(
		MessageContents::Success(translate!(ctx.output, FinishAcquiringAddons)),
		MessageLevel::Important,
	);

	// Install each package one after another onto all of its instances
	ctx.output.display(
		MessageContents::StartProcess(translate!(ctx.output, StartInstallingPackages)),
		MessageLevel::Important,
	);
	for (package, package_instances) in resolved_packages
		.package_to_instances
		.iter()
		.sorted_by_key(|x| x.0)
	{
		ctx.output.start_process();

		for instance_id in package_instances {
			let instance = profile.instances.get_mut(instance_id).ok_or(anyhow!(
				"Instance '{instance_id}' does not exist in the registry"
			))?;

			let version_info = VersionInfo {
				version: constants.version.clone(),
				versions: constants.version_list.clone(),
			};
			let eval = evals
				.get(&(package, instance_id))
				.expect("Evaluation should be in map");
			instance
				.install_eval_data(
					package,
					eval,
					&version_info,
					ctx.paths,
					ctx.lock,
					ctx.output,
				)
				.await
				.context("Failed to install package on instance")?;
		}

		ctx.output.display(
			format_package_update_message(
				package,
				None,
				MessageContents::Success(translate!(ctx.output, FinishInstallingPackage)),
			),
			MessageLevel::Important,
		);
		ctx.output.end_process();
	}

	// Use the instance-package map to remove unused packages and addons
	for (instance_id, packages) in resolved_packages.instance_to_packages {
		let instance = profile.instances.get(&instance_id).ok_or(anyhow!(
			"Instance '{instance_id}' does not exist in the registry"
		))?;
		let inst_ref = instance.get_inst_ref().to_string();
		let files_to_remove = ctx
			.lock
			.remove_unused_packages(
				&inst_ref,
				&packages
					.iter()
					.map(|x| x.id.clone())
					.collect::<Vec<PackageID>>(),
			)
			.context("Failed to remove unused packages")?;
		for file in files_to_remove {
			instance
				.remove_addon_file(&file, ctx.paths)
				.with_context(|| {
					format!(
						"Failed to remove addon file {} for instance {}",
						file.display(),
						instance_id
					)
				})?;
		}
	}

	// Get the set of unique packages
	let mut out = HashSet::new();
	out.extend(resolved_packages.package_to_instances.keys().cloned());

	Ok(out)
}

/// Evaluates addon acquire tasks efficiently with a progress display to the user
async fn run_addon_tasks(
	tasks: HashMap<String, impl Future<Output = anyhow::Result<()>> + Send + 'static>,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<()> {
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

	o.start_process();
	while let Some(result) = task_set.join_next().await {
		result
			.context("Failed to run addon acquire task")?
			.context("Failed to acquire addon")?;

		// Update progress bar
		let progress = MessageContents::Progress {
			current: (total_count - task_set.len()) as u32,
			total: total_count as u32,
		};

		o.display(progress, MessageLevel::Important);
	}

	o.end_process();

	Ok(())
}

/// Resolve packages and create a mapping of packages to a list of instances.
/// This allows us to update packages in a reasonable order to the user.
/// It also returns a map of instances to packages so that unused packages can be removed
async fn resolve_and_batch<'a, O: MCVMOutput>(
	profile: &Profile,
	constants: &EvalConstants,
	ctx: &mut ProfileUpdateContext<'a, O>,
) -> anyhow::Result<ResolvedPackages> {
	let mut batched: HashMap<ArcPkgReq, Vec<InstanceID>> = HashMap::new();
	let mut resolved = HashMap::new();

	for (instance_id, instance) in &profile.instances {
		let mut params = EvalParameters::new(instance.kind.to_side());
		params.stability = profile.default_stability;

		let instance_pkgs = instance.get_configured_packages();
		let instance_resolved = resolve(
			instance_pkgs,
			constants,
			params,
			ctx.paths,
			ctx.packages,
			ctx.client,
			ctx.plugins,
			ctx.output,
		)
		.await
		.with_context(|| {
			format!("Failed to resolve package dependencies for instance '{instance_id}'")
		})?;
		for package in &instance_resolved.packages {
			if let Some(entry) = batched.get_mut(package) {
				entry.push(instance_id.clone());
			} else {
				batched.insert(package.clone(), vec![instance_id.clone()]);
			}
		}
		resolved.insert(instance_id.clone(), instance_resolved.packages);
	}

	Ok(ResolvedPackages {
		package_to_instances: batched,
		instance_to_packages: resolved,
	})
}

struct ResolvedPackages {
	/// A mapping of package IDs to all of the instances they are installed on
	pub package_to_instances: HashMap<ArcPkgReq, Vec<InstanceID>>,
	/// A reverse mapping of instance IDs to all of the packages they have resolved
	pub instance_to_packages: HashMap<InstanceID, Vec<ArcPkgReq>>,
}

/// Checks a package with the registry to report any warnings about it
async fn check_package<'a, O: MCVMOutput>(
	ctx: &mut ProfileUpdateContext<'a, O>,
	pkg: &ArcPkgReq,
) -> anyhow::Result<()> {
	let flags = ctx
		.packages
		.flags(pkg, ctx.paths, ctx.client, ctx.output)
		.await
		.context("Failed to get flags for package")?;
	if flags.contains(&PackageFlag::OutOfDate) {
		ctx.output.display(
			MessageContents::Warning(translate!(ctx.output, PackageOutOfDate, "pkg" = &pkg.id)),
			MessageLevel::Important,
		);
	}

	if flags.contains(&PackageFlag::Deprecated) {
		ctx.output.display(
			MessageContents::Warning(translate!(ctx.output, PackageDeprecated, "pkg" = &pkg.id)),
			MessageLevel::Important,
		);
	}

	if flags.contains(&PackageFlag::Insecure) {
		ctx.output.display(
			MessageContents::Error(translate!(ctx.output, PackageInsecure, "pkg" = &pkg.id)),
			MessageLevel::Important,
		);
	}

	if flags.contains(&PackageFlag::Malicious) {
		ctx.output.display(
			MessageContents::Error(translate!(ctx.output, PackageMalicious, "pkg" = &pkg.id)),
			MessageLevel::Important,
		);
	}

	Ok(())
}

/// Prints support messages about installed packages when updating
pub async fn print_package_support_messages<'a, O: MCVMOutput>(
	packages: &[ArcPkgReq],
	ctx: &mut ProfileUpdateContext<'a, O>,
) -> anyhow::Result<()> {
	let package_count = 5;
	let packages = select_random_n_items_from_list(packages, package_count);
	let mut links = Vec::new();
	for package in packages {
		if let Some(link) = ctx
			.packages
			.get_metadata(package, ctx.paths, ctx.client, ctx.output)
			.await?
			.support_link
			.clone()
		{
			links.push((package, link))
		}
	}
	if !links.is_empty() {
		ctx.output.display(
			MessageContents::Header(translate!(ctx.output, PackageSupportHeader)),
			MessageLevel::Important,
		);
		for (req, link) in links {
			let msg = format_package_update_message(req, None, MessageContents::Hyperlink(link));
			ctx.output.display(msg, MessageLevel::Important);
		}
	}

	Ok(())
}

/// Creates the output message for package installation when updating profiles
fn format_package_update_message(
	pkg: &PkgRequest,
	instance: Option<&str>,
	message: MessageContents,
) -> MessageContents {
	let msg = if let Some(instance) = instance {
		MessageContents::Package(
			pkg.to_owned(),
			Box::new(MessageContents::Associated(
				Box::new(MessageContents::Simple(instance.to_string())),
				Box::new(message),
			)),
		)
	} else {
		MessageContents::Package(pkg.to_owned(), Box::new(message))
	};

	MessageContents::ListItem(Box::new(msg))
}
