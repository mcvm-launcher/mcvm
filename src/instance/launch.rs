use std::collections::HashMap;

use anyhow::Context;
use mcvm_config::instance::{QuickPlay, WrapperCommand};
use mcvm_core::auth_crate::mc::ClientId;
use mcvm_core::io::java::args::MemoryNum;
use mcvm_core::io::java::install::JavaInstallationKind;
use mcvm_core::user::UserManager;
use mcvm_plugin::hook_call::HookHandle;
use mcvm_plugin::hooks::{
	InstanceLaunchArg, OnInstanceLaunch, OnInstanceStop, WhileInstanceLaunch,
};
use mcvm_shared::id::InstanceID;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::{translate, UpdateDepth};
use reqwest::Client;

use super::tracking::RunningInstanceRegistry;
use super::update::manager::UpdateManager;
use crate::io::lock::Lockfile;
use crate::io::paths::Paths;
use crate::plugin::PluginManager;

use super::Instance;

impl Instance {
	/// Launch the instance process
	pub async fn launch(
		&mut self,
		paths: &Paths,
		users: &mut UserManager,
		plugins: &PluginManager,
		settings: LaunchSettings,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<InstanceHandle> {
		o.display(
			MessageContents::StartProcess(translate!(o, StartUpdatingInstance, "inst" = &self.id)),
			MessageLevel::Important,
		);

		let mut manager = UpdateManager::new(UpdateDepth::Shallow);
		let client = Client::new();
		manager.set_version(&self.config.version);
		manager.add_requirements(self.get_requirements());
		manager.set_client_id(settings.ms_client_id);
		if settings.offline_auth {
			manager.offline_auth();
		}
		manager
			.fulfill_requirements(users, plugins, paths, &client, o)
			.await
			.context("Update failed")?;

		let mut lock = Lockfile::open(paths).context("Failed to open lockfile")?;
		let result = self
			.setup(&mut manager, plugins, paths, users, &mut lock, o)
			.await
			.context("Failed to update instance")?;
		manager.add_result(result);

		let hook_arg = InstanceLaunchArg {
			id: self.id.to_string(),
			side: Some(self.get_side()),
			dir: self.dirs.get().inst_dir.to_string_lossy().into(),
			game_dir: self.dirs.get().game_dir.to_string_lossy().into(),
			version_info: manager.version_info.get_clone(),
			config: self.config.original_config_with_profiles.clone(),
			pid: None,
		};

		let mut installed_version = manager
			.get_core_version(o)
			.await
			.context("Failed to get core version")?;

		let mut instance = self
			.create_core_instance(&mut installed_version, paths, o)
			.await
			.context("Failed to create core instance")?;

		// Make sure that any fluff from the update gets ended
		o.end_process();

		o.display(
			MessageContents::StartProcess(translate!(o, PreparingLaunch)),
			MessageLevel::Important,
		);

		// Run pre-launch hooks
		let results = plugins
			.call_hook(OnInstanceLaunch, &hook_arg, paths, o)
			.context("Failed to call on launch hook")?;
		for result in results {
			result.result(o)?;
		}

		// Launch the instance using core
		let handle = instance
			.launch_with_handle(o)
			.await
			.context("Failed to launch core instance")?;

		// Run while_instance_launch hooks alongside
		let hook_handles = plugins
			.call_hook(WhileInstanceLaunch, &hook_arg, paths, o)
			.context("Failed to call while launch hook")?;
		let handle = InstanceHandle {
			inner: handle,
			instance_id: self.id.clone(),
			hook_handles,
			hook_arg,
		};

		// Update the running instance registry
		let mut running_instance_registry = RunningInstanceRegistry::open(paths)
			.context("Failed to open registry of running instances")?;
		running_instance_registry.add_instance(handle.get_pid(), &self.id);
		let _ = running_instance_registry.write();

		Ok(handle)
	}
}

/// Settings for launch provided to the instance launch function
pub struct LaunchSettings {
	/// The Microsoft client ID to use
	pub ms_client_id: ClientId,
	/// Whether to do offline auth
	pub offline_auth: bool,
}

/// Options for launching after conversion from the deserialized version
#[derive(Debug)]
pub struct LaunchOptions {
	/// Java kind
	pub java: JavaInstallationKind,
	/// JVM arguments
	pub jvm_args: Vec<String>,
	/// Game arguments
	pub game_args: Vec<String>,
	/// Minimum JVM memory
	pub min_mem: Option<MemoryNum>,
	/// Maximum JVM memory
	pub max_mem: Option<MemoryNum>,
	/// Environment variables
	pub env: HashMap<String, String>,
	/// Wrapper command
	pub wrapper: Option<WrapperCommand>,
	/// Quick Play options
	pub quick_play: QuickPlay,
	/// Whether or not to use the Log4J configuration
	pub use_log4j_config: bool,
}

/// A handle for an instance
pub struct InstanceHandle {
	/// Core InstanceHandle with the process
	inner: mcvm_core::InstanceHandle,
	/// The ID of the instance
	instance_id: InstanceID,
	/// Handles for hooks running while the instance is running
	hook_handles: Vec<HookHandle<WhileInstanceLaunch>>,
	/// Arg to pass to the stop hook when the instance is stopped
	hook_arg: InstanceLaunchArg,
}

impl InstanceHandle {
	/// Waits for the process to complete
	pub fn wait(
		mut self,
		plugins: &PluginManager,
		paths: &Paths,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<std::process::ExitStatus> {
		let pid = self.get_pid();

		// Wait for the process to complete while polling plugins
		let status = loop {
			for handle in &mut self.hook_handles {
				let _ = handle.poll(o);
			}

			let result = self.inner.try_wait();
			if let Ok(Some(status)) = result {
				break status;
			}
		};

		// Terminate any sibling processes now that the main one is complete
		for handle in self.hook_handles {
			handle.terminate();
		}

		Self::on_stop(&self.instance_id, pid, &self.hook_arg, plugins, paths, o)?;

		Ok(status)
	}

	/// Kills the process early
	pub fn kill(
		mut self,
		plugins: &PluginManager,
		paths: &Paths,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		let pid = self.get_pid();

		for handle in self.hook_handles {
			let _ = handle.kill(o);
		}
		let _ = self.inner.kill();

		Self::on_stop(&self.instance_id, pid, &self.hook_arg, plugins, paths, o)?;

		Ok(())
	}

	/// Gets the internal child process for the game, consuming the
	/// InstanceHandle
	pub fn get_process(self) -> std::process::Child {
		self.inner.get_process()
	}

	/// Gets the PID of the instance process
	pub fn get_pid(&self) -> u32 {
		self.inner.get_pid()
	}

	/// Function that should be run whenever the instance stops
	fn on_stop(
		instance_id: &str,
		pid: u32,
		arg: &InstanceLaunchArg,
		plugins: &PluginManager,
		paths: &Paths,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		// Remove the instance from the registry
		let running_instance_registry = RunningInstanceRegistry::open(paths);
		if let Ok(mut running_instance_registry) = running_instance_registry {
			running_instance_registry.remove_instance(pid, instance_id);
			let _ = running_instance_registry.write();
		}

		// Call on stop hooks
		let results = plugins
			.call_hook(OnInstanceStop, arg, paths, o)
			.context("Failed to call on stop hook")?;
		for result in results {
			result.result(o)?;
		}

		Ok(())
	}
}
