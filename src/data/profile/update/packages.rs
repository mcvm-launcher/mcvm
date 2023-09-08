use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use mcvm_pkg::PkgRequest;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::pkg::PackageStability;

use crate::data::id::InstanceID;
use crate::data::profile::Profile;
use crate::package::eval::{resolve, EvalConstants, EvalInput, EvalParameters, EvalPermissions};
use crate::util::select_random_n_items_from_list;

use super::ProfileUpdateContext;

use anyhow::{anyhow, Context};

/// Install packages on a profile. Returns a set of all unique packages
pub async fn update_profile_packages<'a, O: MCVMOutput>(
	profile: &Profile,
	constants: &EvalConstants,
	ctx: &mut ProfileUpdateContext<'a, O>,
	force: bool,
) -> anyhow::Result<HashSet<PkgRequest>> {
	ctx.output.display(
		MessageContents::StartProcess("Resolving package dependencies".into()),
		MessageLevel::Important,
	);
	let (batched, resolved) = resolve_and_batch(profile, constants, ctx)
		.await
		.context("Failed to resolve dependencies for profile")?;

	ctx.output.display(
		MessageContents::StartProcess("Installing packages".into()),
		MessageLevel::Important,
	);
	for (package, package_instances) in batched.iter().sorted_by_key(|x| x.0) {
		ctx.output.start_process();

		let pkg_version = ctx
			.packages
			.get_version(package, ctx.paths, ctx.client)
			.await
			.context("Failed to get version for package")?;
		let mut notices = Vec::new();
		for instance_id in package_instances {
			let instance = ctx.instances.get(instance_id).ok_or(anyhow!(
				"Instance '{instance_id}' does not exist in the registry"
			))?;
			let params = EvalParameters {
				side: instance.kind.to_side(),
				features: Vec::new(),
				perms: EvalPermissions::Standard,
				stability: PackageStability::Stable,
			};
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
					pkg_version,
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
	for (instance_id, packages) in resolved {
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
					.collect::<Vec<String>>(),
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

	let mut out = HashSet::new();
	out.extend(batched.keys().cloned());

	Ok(out)
}

/// Resolve packages and create a mapping of packages to a list of instances.
/// This allows us to update packages in a reasonable order to the user.
/// It also returns a map of instances to packages so that unused packages can be removed
async fn resolve_and_batch<'a, O: MCVMOutput>(
	profile: &Profile,
	constants: &EvalConstants,
	ctx: &mut ProfileUpdateContext<'a, O>,
) -> anyhow::Result<(
	HashMap<PkgRequest, Vec<InstanceID>>,
	HashMap<InstanceID, Vec<PkgRequest>>,
)> {
	let mut batched: HashMap<PkgRequest, Vec<InstanceID>> = HashMap::new();
	let mut resolved = HashMap::new();
	for instance_id in &profile.instances {
		let instance = ctx.instances.get(instance_id).ok_or(anyhow!(
			"Instance '{instance_id}' does not exist in the registry"
		))?;
		let params = EvalParameters {
			side: instance.kind.to_side(),
			features: Vec::new(),
			perms: EvalPermissions::Standard,
			stability: PackageStability::Stable,
		};
		let instance_resolved = resolve(
			&profile.packages,
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

	Ok((batched, resolved))
}

/// Prints support messages about installed packages when updating
pub async fn print_package_support_messages<'a, O: MCVMOutput>(
	packages: &[PkgRequest],
	ctx: &mut ProfileUpdateContext<'a, O>,
) -> anyhow::Result<()> {
	let package_count = 5;
	let packages = select_random_n_items_from_list(packages, package_count);
	let mut links = Vec::new();
	for package in packages {
		if let Some(link) = ctx
			.packages
			.get_metadata(package, ctx.paths, ctx.client)
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
				instance.to_string(),
				Box::new(message),
			)),
		)
	} else {
		MessageContents::Package(pkg.to_owned(), Box::new(message))
	};

	MessageContents::ListItem(Box::new(msg))
}
