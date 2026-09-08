use anyhow::{Context, bail};
use nitro_instance::lock::InstanceLockfile;
use nitro_shared::output::{MessageContents, NitroOutput};
use nitro_shared::pkg::ArcPkgReq;
use nitro_shared::translate;
use nitro_shared::versions::VersionInfo;
use reqwest::Client;

use crate::addon::{AddonExt, ResolvedPackageAddon};
use crate::io::paths::Paths;
use crate::pkg::eval::EvalData;

use super::Instance;
use crate::config::package::PackageConfig;

use std::collections::HashMap;
use std::future::Future;

impl Instance {
	/// Installs a package on this instance
	#[allow(clippy::too_many_arguments)]
	pub async fn install_package(
		&mut self,
		pkg: &ArcPkgReq,
		eval: &EvalData,
		mc_version: &str,
		paths: &Paths,
		inst_lock: &mut InstanceLockfile,
		force: bool,
		client: &Client,
		o: &mut impl NitroOutput,
	) -> anyhow::Result<()> {
		let version_info = VersionInfo {
			version: mc_version.to_string(),
			versions: eval.input.constants.version_list.clone(),
		};

		let tasks = self.get_package_addon_tasks(eval, paths, force, client);

		for task in tasks.into_values() {
			task.await.context("Failed to install addon")?;
		}

		self.install_eval_data(pkg, eval, &version_info, paths, inst_lock, o)
			.await
			.context("Failed to install evaluation data on instance")?;

		Ok(())
	}

	/// Gets the tasks for installing addons for a package
	pub fn get_package_addon_tasks(
		&mut self,
		eval: &EvalData,
		paths: &Paths,
		force: bool,
		client: &Client,
	) -> HashMap<String, impl Future<Output = anyhow::Result<()>> + Send + 'static> {
		let mut tasks = HashMap::new();
		for addon in eval.addon_reqs.iter() {
			if addon.addon.should_update(paths, &self.id) || force {
				let task = addon.get_acquire_task(paths, &self.id, client);
				tasks.insert(addon.get_unique_id(&self.id), task);
			}
		}

		tasks
	}

	/// Install the EvalData resulting from evaluating a package onto this instance
	#[allow(clippy::too_many_arguments)]
	pub async fn install_eval_data(
		&mut self,
		pkg: &ArcPkgReq,
		eval: &EvalData,
		version_info: &VersionInfo,
		paths: &Paths,
		inst_lock: &mut InstanceLockfile,
		o: &mut impl NitroOutput,
	) -> anyhow::Result<()> {
		// Get the configuration for the package or the default if it is not configured by the user
		let pkg_config = self
			.get_package_config(pkg)
			.cloned()
			.unwrap_or_else(|| PackageConfig::from_id(pkg.id.clone()));

		if eval.uses_custom_instructions {
			o.display(MessageContents::Warning(translate!(
				o,
				CustomInstructionsWarning
			)));
		}

		// Run commands
		run_package_commands(&eval.commands, o).context("Failed to run package commands")?;

		// Install addons

		let addons: Vec<_> = eval
			.addon_reqs
			.iter()
			.map(|x| {
				let mut addon = x.addon.addon(x.addon.get_path(paths, &self.id));
				self.get_addon_targets(&mut addon, &pkg_config.worlds, version_info);

				ResolvedPackageAddon {
					pkg_addon: x.addon.clone(),
					addon,
				}
			})
			.collect();

		let lockfile_addons: Vec<_> = addons.iter().map(|x| x.to_lockfile_addon()).collect();

		let files_to_remove =
			inst_lock.update_package(pkg, &lockfile_addons, eval.selected_content_version.clone());

		for addon in addons {
			self.create_addon(&addon.addon, &pkg_config.worlds, version_info)
				.with_context(|| format!("Failed to install addon '{}'", addon.pkg_addon.id))?;
		}

		for path in files_to_remove {
			if path.exists() {
				let _ = std::fs::remove_file(path);
			}
		}

		Ok(())
	}

	/// Gets all of the configured packages for this instance
	pub fn packages(&self) -> &Vec<PackageConfig> {
		&self.packages
	}

	/// Gets the configuration for a specific package on this instance
	pub fn get_package_config(&self, package: &ArcPkgReq) -> Option<&PackageConfig> {
		let configured_packages = self.packages();

		configured_packages.iter().find(|x| x.req == *package)
	}
}

/// Runs package commands
fn run_package_commands(commands: &[Vec<String>], o: &mut impl NitroOutput) -> anyhow::Result<()> {
	if !commands.is_empty() {
		o.display(MessageContents::StartProcess(translate!(
			o,
			StartRunningCommands
		)));

		for command_and_args in commands {
			let program = command_and_args
				.first()
				.expect("Command should contain at least the program");
			let mut command = std::process::Command::new(program);
			command.args(&command_and_args[1..]);
			let mut child = command
				.spawn()
				.context("Failed to spawn command {program}")?;
			let result = child.wait()?;
			if !result.success() {
				bail!("Command {program} returned a non-zero exit code");
			}
		}

		o.display(MessageContents::Success(translate!(
			o,
			FinishRunningCommands
		)));
	}

	Ok(())
}
