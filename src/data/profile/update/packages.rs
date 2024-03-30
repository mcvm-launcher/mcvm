use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use mcvm_pkg::repo::PackageFlag;
use mcvm_pkg::PkgRequest;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::pkg::{ArcPkgReq, PackageID};

use crate::data::config::package::PackageConfig;
use crate::data::id::InstanceID;
use crate::data::profile::Profile;
use crate::pkg::eval::{resolve, EvalConstants, EvalInput, EvalParameters};
use crate::util::select_random_n_items_from_list;

use super::ProfileUpdateContext;

use anyhow::{anyhow, Context};

/// Install packages on a profile. Returns a set of all unique packages
pub async fn update_profile_packages<'a, O: MCVMOutput>(
	profile: &Profile,
	constants: &EvalConstants,
	ctx: &mut ProfileUpdateContext<'a, O>,
	force: bool,
) -> anyhow::Result<HashSet<ArcPkgReq>> {
	// Resolve dependencies
	ctx.output.display(
		MessageContents::StartProcess("Resolving package dependencies".into()),
		MessageLevel::Important,
	);
	let resolved_packages = resolve_and_batch(profile, constants, ctx)
		.await
		.context("Failed to resolve dependencies for profile")?;

	// Install each package one after another onto all of it's instances
	ctx.output.display(
		MessageContents::StartProcess("Installing packages".into()),
		MessageLevel::Important,
	);
	for (package, package_instances) in resolved_packages
		.package_to_instances
		.iter()
		.sorted_by_key(|x| x.0)
	{
		// Check the package to display warnings
		check_package(ctx, package)
			.await
			.with_context(|| format!("Failed to check package {package}"))?;

		ctx.output.start_process();

		// Install the package on it's instances
		let mut notices = Vec::new();
		for instance_id in package_instances {
			let instance = ctx.instances.get_mut(instance_id).ok_or(anyhow!(
				"Instance '{instance_id}' does not exist in the registry"
			))?;

			// Get the configuration for the package or the default if it is not configured by the user
			let package_config = instance
				.get_package_config(&package.id)
				.cloned()
				.unwrap_or_else(|| PackageConfig::Basic(package.id.clone()));

			let params = EvalParameters::new(instance.kind.to_side());

			ctx.output.display(
				format_package_update_message(
					package,
					Some(instance_id),
					MessageContents::StartProcess("Installing".into()),
				),
				MessageLevel::Important,
			);

			let input = EvalInput { constants, params };
			let result = instance
				.install_package(
					package,
					&package_config,
					input,
					ctx.packages,
					ctx.paths,
					ctx.lock,
					force,
					ctx.client,
					ctx.output,
				)
				.await
				.with_context(|| {
					format!("Failed to install package '{package}' for instance '{instance_id}'")
				})?;

			// Add any notices to the list
			notices.extend(
				result
					.notices
					.iter()
					.map(|x| (instance_id.clone(), x.to_owned())),
			);
		}
		ctx.output.display(
			format_package_update_message(
				package,
				None,
				MessageContents::Success("Installed".into()),
			),
			MessageLevel::Important,
		);
		ctx.output.end_process();

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

	// Use the instance-package map to remove unused packages and addons
	for (instance_id, packages) in resolved_packages.instance_to_packages {
		let instance = ctx.instances.get(&instance_id).ok_or(anyhow!(
			"Instance '{instance_id}' does not exist in the registry"
		))?;
		let files_to_remove = ctx
			.lock
			.remove_unused_packages(
				&instance_id,
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

	for instance_id in &profile.instances {
		let instance = ctx.instances.get(instance_id).ok_or(anyhow!(
			"Instance '{instance_id}' does not exist in the registry"
		))?;
		let params = EvalParameters::new(instance.kind.to_side());
		let instance_pkgs = instance.get_configured_packages();
		let instance_resolved = resolve(
			&instance_pkgs,
			constants,
			params,
			ctx.paths,
			ctx.packages,
			ctx.client,
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
			MessageContents::Warning(format!("Package {pkg} has been flagged as out of date")),
			MessageLevel::Important,
		);
	}

	if flags.contains(&PackageFlag::Deprecated) {
		ctx.output.display(
			MessageContents::Warning(format!("Package {pkg} has been flagged as deprecated")),
			MessageLevel::Important,
		);
	}

	if flags.contains(&PackageFlag::Insecure) {
		ctx.output.display(
			MessageContents::Error(format!("Package {pkg} has been flagged as insecure")),
			MessageLevel::Important,
		);
	}

	if flags.contains(&PackageFlag::Malicious) {
		ctx.output.display(
			MessageContents::Error(format!("Package {pkg} has been flagged as malicious")),
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
			MessageContents::Header("Packages to consider supporting:".into()),
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
