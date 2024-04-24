use std::collections::HashMap;

use anyhow::Context;
use mcvm_core::auth_crate::mc::ClientId;
use mcvm_core::io::java::args::{ArgsPreset, MemoryNum};
use mcvm_core::io::java::install::JavaInstallationKind;
use mcvm_core::user::UserManager;
use mcvm_core::util::versions::MinecraftVersion;
use mcvm_core::InstanceHandle;
use mcvm_shared::lang::translate::TranslationKey;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::translate;
use reqwest::Client;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::data::config::instance::QuickPlay;
use crate::data::config::plugin::PluginManager;
use crate::data::profile::update::manager::UpdateManager;
use crate::io::files::paths::Paths;

use super::Instance;

impl Instance {
	/// Launch the instance process
	pub async fn launch(
		&mut self,
		paths: &Paths,
		users: &mut UserManager,
		plugins: &PluginManager,
		version: &MinecraftVersion,
		settings: LaunchSettings,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<InstanceHandle> {
		o.display(
			MessageContents::StartProcess(translate!(o, StartUpdatingInstance)),
			MessageLevel::Important,
		);

		let mut manager = UpdateManager::new(false, true);
		let client = Client::new();
		manager.set_version(version);
		manager.add_requirements(self.get_requirements());
		manager.set_client_id(settings.ms_client_id);
		if settings.offline_auth {
			manager.offline_auth();
		}
		manager
			.fulfill_requirements(users, plugins, paths, &client, o)
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

		// Make sure that any fluff from the update gets ended
		o.end_process();

		o.display(
			MessageContents::Success(translate!(o, Launch)),
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
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct WrapperCommand {
	/// The command to run
	pub cmd: String,
	/// The command's arguments
	pub args: Vec<String>,
}
