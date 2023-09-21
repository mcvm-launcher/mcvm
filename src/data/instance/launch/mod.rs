/// Launching the client
pub mod client;
/// Launching the server
pub mod server;

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::versions::VersionInfo;
use mcvm_shared::Side;
use oauth2::ClientId;
use reqwest::Client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::data::config::instance::QuickPlay;
use crate::data::instance::InstKind;
use crate::data::profile::update::manager::UpdateManager;
use crate::data::user::UserManager;
use crate::io::files::paths::Paths;
use crate::io::java::args::{ArgsPreset, MemoryArg, MemoryNum};
use crate::io::java::install::JavaInstallationKind;
use crate::io::lock::Lockfile;
use crate::util::print::PrintOptions;
use crate::util::utc_timestamp;
use crate::util::versions::MinecraftVersion;

use self::client::create_quick_play_args;

use super::Instance;

impl Instance {
	/// Launch the instance process
	pub async fn launch(
		&mut self,
		paths: &Paths,
		lock: &mut Lockfile,
		users: &mut UserManager,
		version: &MinecraftVersion,
		ms_client_id: ClientId,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		o.display(
			MessageContents::StartProcess("Checking for updates".into()),
			MessageLevel::Important,
		);
		let options = PrintOptions::new(false, 0);
		let mut manager = UpdateManager::new(options, false, true);
		let client = Client::new();
		manager
			.fulfill_version_manifest(version, paths, &client, o)
			.await
			.context("Failed to get version data")?;
		manager.add_requirements(self.get_requirements());
		manager
			.fulfill_requirements(paths, lock, &client, o)
			.await
			.context("Update failed")?;

		self.create(&manager, paths, users, &client, o)
			.await
			.context("Failed to update instance")?;
		let version_info = manager.version_info.get();
		o.display(
			MessageContents::Success("Launching!".into()),
			MessageLevel::Important,
		);
		match &self.kind {
			InstKind::Client { .. } => {
				self.launch_client(
					paths,
					users,
					version_info,
					&client,
					ms_client_id,
					&manager,
					o,
				)
				.await
				.context("Failed to launch client")?;
			}
			InstKind::Server { .. } => {
				self.launch_server(paths, version_info, &manager, o)
					.context("Failed to launch server")?;
			}
		}
		Ok(())
	}

	/// Actually launch the game
	fn launch_game_process(
		&self,
		properties: LaunchProcessProperties,
		version_info: &VersionInfo,
		paths: &Paths,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		let mut log = File::create(log_file_path(&self.id, paths)?)
			.context("Failed to open launch log file")?;
		let mut cmd = match &self.config.launch.wrapper {
			Some(wrapper) => {
				let mut cmd = Command::new(&wrapper.cmd);
				cmd.args(&wrapper.args);
				cmd.arg(properties.command);
				cmd
			}
			None => Command::new(properties.command),
		};
		cmd.current_dir(properties.cwd);
		cmd.envs(self.config.launch.env.clone());
		cmd.envs(properties.additional_env_vars);

		cmd.args(self.config.launch.generate_jvm_args());
		cmd.args(properties.jvm_args);
		if let Some(main_class) = properties.main_class {
			cmd.arg(main_class);
		}
		cmd.args(properties.game_args);
		cmd.args(
			self.config
				.launch
				.generate_game_args(version_info, self.kind.to_side(), o),
		);

		writeln!(log, "Launch command: {cmd:#?}").context("Failed to write to launch log file")?;
		o.display(
			MessageContents::Property(
				"Launch command".into(),
				Box::new(MessageContents::Simple(format!("{cmd:#?}"))),
			),
			MessageLevel::Debug,
		);

		let mut child = cmd.spawn().context("Failed to spawn child process")?;
		child.wait().context("Failed to wait for child process")?;

		Ok(())
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

impl LaunchOptions {
	/// Create the args for the JVM when launching the game
	pub fn generate_jvm_args(&self) -> Vec<String> {
		let mut out = self.jvm_args.clone();

		if let Some(n) = &self.min_mem {
			out.push(MemoryArg::Min.to_string(n.clone()));
		}
		if let Some(n) = &self.max_mem {
			out.push(MemoryArg::Max.to_string(n.clone()));
		}

		let avg = match &self.min_mem {
			Some(min) => self
				.max_mem
				.as_ref()
				.map(|max| MemoryNum::avg(min.clone(), max.clone())),
			None => None,
		};
		out.extend(self.preset.generate_args(avg));

		out
	}

	/// Create the args for the game when launching
	pub fn generate_game_args(
		&self,
		version_info: &VersionInfo,
		side: Side,
		o: &mut impl MCVMOutput,
	) -> Vec<String> {
		let mut out = self.game_args.clone();

		if let Side::Client = side {
			out.extend(create_quick_play_args(&self.quick_play, version_info, o));
		}

		out
	}
}

/// A wrapper command
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct WrapperCommand {
	/// The command to run
	pub cmd: String,
	/// The command's arguments
	pub args: Vec<String>,
}

/// Get the name of the launch log file
fn log_file_name(instance_id: &str) -> anyhow::Result<String> {
	Ok(format!("{instance_id}-{}.txt", utc_timestamp()?))
}

/// Get the path to the launch log file
fn log_file_path(instance_id: &str, paths: &Paths) -> anyhow::Result<PathBuf> {
	Ok(paths.launch_logs.join(log_file_name(instance_id)?))
}
