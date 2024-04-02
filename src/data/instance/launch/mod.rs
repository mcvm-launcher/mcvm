use std::collections::HashMap;
use std::path::Path;

use anyhow::Context;
use mcvm_core::io::java::args::{ArgsPreset, MemoryNum};
use mcvm_core::io::java::install::JavaInstallationKind;
use mcvm_core::user::UserManager;
use mcvm_core::util::versions::MinecraftVersion;
use mcvm_core::InstanceHandle;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use oauth2::ClientId;
use reqwest::Client;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::data::config::instance::QuickPlay;
use crate::data::profile::update::manager::UpdateManager;
use crate::io::files::paths::Paths;
use crate::util::print::PrintOptions;

use super::Instance;

impl Instance {
	/// Launch the instance process
	pub async fn launch(
		&mut self,
		paths: &Paths,
		users: &mut UserManager,
		version: &MinecraftVersion,
		ms_client_id: ClientId,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<InstanceHandle> {
		o.display(
			MessageContents::StartProcess("Checking for updates".into()),
			MessageLevel::Important,
		);

		let options = PrintOptions::new(false, 0);
		let mut manager = UpdateManager::new(options, false, true);
		let client = Client::new();
		manager.set_version(version);
		manager.add_requirements(self.get_requirements());
		manager.set_client_id(ms_client_id);
		manager
			.fulfill_requirements(users, paths, &client, o)
			.await
			.context("Update failed")?;

		let result = self
			.create(&mut manager, paths, users, &client, o)
			.await
			.context("Failed to update instance")?;
		manager.add_result(result);

		let mut installed_version = manager
			.get_core_version(o)
			.await
			.context("Failed to get core version")?;

		let mut instance = self
			.create_core_instance(&mut installed_version, paths, o)
			.await
			.context("Failed to create core instance")?;

		o.display(
			MessageContents::Success("Launching!".into()),
			MessageLevel::Important,
		);
		// Launch the instance using core
		let handle = instance
			.launch_with_handle(o)
			.await
			.context("Failed to launch core instance")?;

		Ok(handle)
	}
}

/// Argument for the launch_game_process command that includes properties about the launch command
pub struct LaunchProcessProperties<'a> {
	/// The current working directory, usually the instance subdir
	pub cwd: &'a Path,
	/// The base command to run, usually the path to the JVM
	pub command: &'a str,
	/// Arguments for the JVM
	pub jvm_args: &'a [String],
	/// The Java main class to run
	pub main_class: Option<&'a str>,
	/// Arguments for the game
	pub game_args: &'a [String],
	/// Additional environment variables to add to the launch command
	pub additional_env_vars: &'a HashMap<String, String>,
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
	/// Java arguments preset
	pub preset: ArgsPreset,
	/// Environment variables
	pub env: HashMap<String, String>,
	/// Wrapper command
	pub wrapper: Option<WrapperCommand>,
	/// Quick Play options
	pub quick_play: QuickPlay,
	/// Whether or not to use the Log4J configuration
	pub use_log4j_config: bool,
}

/// A wrapper command
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct WrapperCommand {
	/// The command to run
	pub cmd: String,
	/// The command's arguments
	pub args: Vec<String>,
}
