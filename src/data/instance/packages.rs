use anyhow::{bail, Context};
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::pkg::ArcPkgReq;
use mcvm_shared::translate;
use mcvm_shared::versions::VersionInfo;
use reqwest::Client;

use crate::data::addon::AddonExt;
use crate::data::config::plugin::PluginManager;
use crate::io::files::paths::Paths;
use crate::io::lock::{Lockfile, LockfileAddon};
use crate::pkg::eval::{EvalData, EvalInput, Routine};
use crate::pkg::reg::PkgRegistry;

use super::Instance;
use crate::data::config::package::PackageConfig;

use std::collections::HashMap;
use std::future::Future;

impl Instance {
	/// Installs a package on this instance
	#[allow(clippy::too_many_arguments)]
	pub async fn install_package<'a>(
		&mut self,
		pkg: &ArcPkgReq,
		eval_input: EvalInput<'a>,
		reg: &mut PkgRegistry,
		paths: &'a Paths,
		lock: &mut Lockfile,
		force: bool,
		client: &Client,
		plugins: &'a PluginManager,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<EvalData<'a>> {
		let version_info = VersionInfo {
			version: eval_input.constants.version.clone(),
			versions: eval_input.constants.version_list.clone(),
		};

		let (eval, tasks) = self
			.get_package_addon_tasks(pkg, eval_input, reg, paths, force, client, plugins, o)
			.await
			.context("Failed to get download tasks for installing package")?;

		for task in tasks.into_values() {
			task.await.context("Failed to install addon")?;
		}

		self.install_eval_data(pkg, &eval, &version_info, paths, lock, o)
			.await
			.context("Failed to install evaluation data on instance")?;

		Ok(eval)
	}

	/// Gets the tasks for installing addons for a package by evaluating it
	#[allow(clippy::too_many_arguments)]
	pub async fn get_package_addon_tasks<'a>(
		&mut self,
		pkg: &ArcPkgReq,
		eval_input: EvalInput<'a>,
		reg: &mut PkgRegistry,
		paths: &'a Paths,
		force: bool,
		client: &Client,
		plugins: &'a PluginManager,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<(
		EvalData<'a>,
		HashMap<String, impl Future<Output = anyhow::Result<()>> + Send + 'static>,
	)> {
		let eval = reg
			.eval(pkg, paths, Routine::Install, eval_input, client, plugins, o)
			.await
			.context("Failed to evaluate package")?;

		let mut tasks = HashMap::new();
		for addon in eval.addon_reqs.iter() {
			if addon.addon.should_update(paths, &self.id) || force {
				let task = addon
					.get_acquire_task(paths, &self.id, client)
					.context("Failed to get task for acquiring addon")?;
				tasks.insert(addon.get_unique_id(&self.id), task);
			}
		}

		Ok((eval, tasks))
	}

	/// Install the EvalData resulting from evaluating a package onto this instance
	#[allow(clippy::too_many_arguments)]
	pub async fn install_eval_data<'a>(
		&mut self,
		pkg: &ArcPkgReq,
		eval: &EvalData<'a>,
		version_info: &VersionInfo,
		paths: &Paths,
		lock: &mut Lockfile,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		// Get the configuration for the package or the default if it is not configured by the user
		let pkg_config = self
			.get_package_config(&pkg.id)
			.cloned()
			.unwrap_or_else(|| PackageConfig::from_id(pkg.id.clone()));

		if eval.uses_custom_instructions {
			o.display(
				MessageContents::Warning(translate!(o, CustomInstructionsWarning)),
				MessageLevel::Important,
			);
		}

		// Run commands
		run_package_commands(&eval.commands, o).context("Failed to run package commands")?;

		let lockfile_addons = eval
			.addon_reqs
			.iter()
			.map(|x| {
				Ok(LockfileAddon::from_addon(
					&x.addon,
					self.get_linked_addon_paths(&x.addon, &pkg_config.worlds, paths, version_info)?
						.iter()
						.map(|y| y.join(x.addon.file_name.clone()))
						.collect(),
				))
			})
			.collect::<anyhow::Result<Vec<LockfileAddon>>>()
			.context("Failed to convert addons to the lockfile format")?;

		let files_to_remove = lock
			.update_package(
				&pkg.id,
				&self.get_inst_ref().to_string(),
				&lockfile_addons,
				o,
			)
			.context("Failed to update package in lockfile")?;

		for addon in eval.addon_reqs.iter() {
			self.create_addon(&addon.addon, &pkg_config.worlds, paths, version_info)
				.with_context(|| format!("Failed to install addon '{}'", addon.addon.id))?;
		}

		for path in files_to_remove {
			self.remove_addon_file(&path, paths)
				.context("Failed to remove addon file from instance")?;
		}

		Ok(())
	}

	/// Gets all of the configured packages for this instance
	pub fn get_configured_packages(&self) -> &Vec<PackageConfig> {
		&self.config.packages
	}

	/// Gets the configuration for a specific package on this instance
	pub fn get_package_config(&self, package: &str) -> Option<&PackageConfig> {
		let configured_packages = self.get_configured_packages();

		configured_packages.iter().find(|x| x.id == package.into())
	}
}

/// Runs package commands
fn run_package_commands(commands: &[Vec<String>], o: &mut impl MCVMOutput) -> anyhow::Result<()> {
	if !commands.is_empty() {
		o.display(
			MessageContents::StartProcess(translate!(o, StartRunningCommands)),
			MessageLevel::Important,
		);

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

		o.display(
			MessageContents::Success(translate!(o, FinishRunningCommands)),
			MessageLevel::Important,
		);
	}

	Ok(())
}
