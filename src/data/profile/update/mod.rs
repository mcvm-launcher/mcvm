/// UpdateManager
pub mod manager;
/// Updating packages on a profile
pub mod packages;

use crate::data::config::plugin::PluginManager;
#[cfg(not(feature = "disable_profile_update_packages"))]
use crate::pkg::eval::EvalConstants;
use mcvm_shared::translate;
#[cfg(not(feature = "disable_profile_update_packages"))]
use packages::{print_package_support_messages, update_profile_packages};
#[cfg(not(feature = "disable_profile_update_packages"))]
use std::collections::HashSet;

use anyhow::{anyhow, Context};
use mcvm_mods::paper;
use mcvm_shared::modifications::ServerType;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use reqwest::Client;

use crate::data::config::Config;
use crate::io::files::paths::Paths;
use crate::io::lock::Lockfile;
use crate::pkg::reg::PkgRegistry;
use mcvm_shared::id::ProfileID;

use manager::UpdateManager;

use super::Profile;

/// Shared objects for profile updating functions
pub struct ProfileUpdateContext<'a, O: MCVMOutput> {
	/// The package registry
	pub packages: &'a mut PkgRegistry,
	/// The shared paths
	pub paths: &'a Paths,
	/// The lockfile
	pub lock: &'a mut Lockfile,
	/// The reqwest client
	pub client: &'a Client,
	/// The plugins
	pub plugins: &'a PluginManager,
	/// The output object
	pub output: &'a mut O,
}

/// Update a list of profiles
pub async fn update_profiles(
	paths: &Paths,
	config: &mut Config,
	ids: &[ProfileID],
	force: bool,
	update_packages: bool,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<()> {
	#[cfg(feature = "disable_profile_update_packages")]
	let _update_packages = update_packages;

	let client = Client::new();
	let mut lock = Lockfile::open(paths).context("Failed to open lockfile")?;

	let mut ctx = ProfileUpdateContext {
		packages: &mut config.packages,
		paths,
		lock: &mut lock,
		client: &client,
		output: o,
		plugins: &config.plugins,
	};

	let mut manager = UpdateManager::new(force, false);

	for id in ids {
		let profile = config
			.profiles
			.get_mut(id)
			.ok_or(anyhow!("Unknown profile '{id}'"))?;

		ctx.output.display(
			MessageContents::Header(translate!(ctx.output, StartUpdatingProfile, "prof" = id)),
			MessageLevel::Important,
		);

		manager.set_version(&profile.version);
		manager
			.fulfill_requirements(
				&config.users,
				&config.plugins,
				paths,
				ctx.client,
				ctx.output,
			)
			.await
			.context("Failed to fulfill update manager")?;
		let mc_version = manager.version_info.get().version.clone();

		let paper_properties = get_paper_properties(profile, &mc_version, &mut ctx)
			.await
			.context("Failed to get Paper build number and filename")?;

		check_profile_version_change(profile, &mc_version, paper_properties.clone(), &mut ctx)
			.await
			.context("Failed to check for a profile version update")?;

		check_profile_paper_update(profile, paper_properties, &mut ctx)
			.await
			.context("Failed to check for Paper updates")?;

		ctx.lock
			.finish(paths)
			.await
			.context("Failed to finish using lockfile")?;

		if !update_packages {
			continue;
		}

		#[cfg(not(feature = "disable_profile_update_packages"))]
		{
			let mut all_packages = HashSet::new();

			if !profile.instances.is_empty() {
				profile
					.create(
						paths,
						&mut manager,
						&config.users,
						&config.plugins,
						ctx.client,
						ctx.output,
					)
					.await
					.context("Failed to create profile instances")?;

				ctx.output.display(
					MessageContents::Header(translate!(ctx.output, StartUpdatingPackages)),
					MessageLevel::Important,
				);

				let constants = EvalConstants {
					version: mc_version.to_string(),
					modifications: profile.modifications.clone(),
					version_list: manager.version_info.get().versions.clone(),
					language: config.prefs.language,
					profile_stability: profile.default_stability,
				};

				let packages =
					update_profile_packages(profile, &constants, &mut ctx, force).await?;

				ctx.output.display(
					MessageContents::Success(translate!(ctx.output, FinishUpdatingPackages)),
					MessageLevel::Important,
				);

				all_packages.extend(packages);
			}

			ctx.lock
				.finish(paths)
				.await
				.context("Failed to finish using lockfile")?;

			let all_packages = Vec::from_iter(all_packages);
			print_package_support_messages(&all_packages, &mut ctx)
				.await
				.context("Failed to print support messages")?;
		}
	}

	Ok(())
}

/// Update a profile when the Minecraft version has changed
async fn check_profile_version_change<'a, O: MCVMOutput>(
	profile: &mut Profile,
	mc_version: &str,
	paper_properties: Option<(u16, String)>,
	ctx: &mut ProfileUpdateContext<'a, O>,
) -> anyhow::Result<()> {
	if ctx.lock.update_profile_version(&profile.id, mc_version) {
		ctx.output.start_process();
		ctx.output.display(
			MessageContents::StartProcess(translate!(ctx.output, StartUpdatingProfileVersion)),
			MessageLevel::Important,
		);

		for instance in profile.instances.values_mut() {
			instance
				.teardown(ctx.paths, paper_properties.clone())
				.context("Failed to remove old files when updating Minecraft version")?;
		}

		ctx.output.display(
			MessageContents::Success(translate!(ctx.output, FinishUpdatingProfileVersion)),
			MessageLevel::Important,
		);
		ctx.output.end_process();
	}
	Ok(())
}

/// Get the updated Paper file name and build number for a profile that uses it
async fn get_paper_properties<'a, O: MCVMOutput>(
	profile: &Profile,
	mc_version: &str,
	ctx: &mut ProfileUpdateContext<'a, O>,
) -> anyhow::Result<Option<(u16, String)>> {
	let out = if let ServerType::Paper = profile.modifications.server_type {
		let build_num = paper::get_newest_build(paper::Mode::Paper, mc_version, ctx.client)
			.await
			.context("Failed to get the newest Paper build number")?;
		let paper_file_name =
			paper::get_jar_file_name(paper::Mode::Paper, mc_version, build_num, ctx.client)
				.await
				.context("Failed to get the name of the Paper Jar file")?;
		Some((build_num, paper_file_name))
	} else {
		None
	};

	Ok(out)
}

// TODO: Make this work with Folia
/// Remove the old Paper files for a profile if they have updated
async fn check_profile_paper_update<'a, O: MCVMOutput>(
	profile: &mut Profile,
	paper_properties: Option<(u16, String)>,
	ctx: &mut ProfileUpdateContext<'a, O>,
) -> anyhow::Result<()> {
	if let Some((build_num, file_name)) = paper_properties {
		if ctx.lock.update_profile_paper_build(&profile.id, build_num) {
			for inst in profile.instances.values_mut() {
				inst.remove_paper(ctx.paths, file_name.clone())
					.context("Failed to remove Paper")?;
			}
		}
	}

	Ok(())
}
