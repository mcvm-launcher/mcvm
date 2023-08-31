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
use mcvm_shared::instance::Side;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::versions::VersionInfo;

use crate::data::config::instance::QuickPlay;
use crate::data::instance::InstKind;
use crate::data::profile::update::UpdateManager;
use crate::data::user::UserManager;
use crate::io::files::paths::Paths;
use crate::io::java::args::{ArgsPreset, MemoryArg, MemoryNum};
use crate::io::java::JavaKind;
use crate::io::lock::Lockfile;
use crate::util::print::PrintOptions;
use crate::util::utc_timestamp;
use crate::util::versions::MinecraftVersion;

use self::client::create_quick_play_args;

use super::Instance;

impl Instance {
	// Launch the instance
	pub async fn launch(
		&mut self,
		paths: &Paths,
		lock: &mut Lockfile,
		users: &UserManager,
		version: &MinecraftVersion,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		o.display(
			MessageContents::StartProcess("Checking for updates".to_string()),
			MessageLevel::Important,
		);
		let options = PrintOptions::new(false, 0);
		let mut manager = UpdateManager::new(options, false, true);
		manager
			.fulfill_version_manifest(version, paths, o)
			.await
			.context("Failed to get version data")?;
		manager.add_requirements(self.get_requirements());
		manager
			.fulfill_requirements(paths, lock, o)
			.await
			.context("Update failed")?;

		self.create(&manager, paths, users, o)
			.await
			.context("Failed to update instance")?;
		let version_info = manager.version_info.get();
		o.display(
			MessageContents::Success("Launching!".to_string()),
			MessageLevel::Important,
		);
		match &self.kind {
			InstKind::Client { .. } => {
				self.launch_client(paths, users, version_info, o)
					.context("Failed to launch client")?;
			}
			InstKind::Server { .. } => {
				self.launch_server(paths, version_info, o)
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
		let mut cmd = match &self.launch.wrapper {
			Some(wrapper) => {
				let mut cmd = Command::new(wrapper);
				cmd.arg(properties.command);
				cmd
			}
			None => Command::new(properties.command),
		};
		cmd.current_dir(properties.cwd);
		cmd.envs(self.launch.env.clone());
		cmd.envs(properties.additional_env_vars);

		cmd.args(self.launch.generate_jvm_args());
		cmd.args(properties.jvm_args);
		if let Some(main_class) = properties.main_class {
			cmd.arg(main_class);
		}
		cmd.args(properties.game_args);
		cmd.args(
			self.launch
				.generate_game_args(version_info, self.kind.to_side(), o),
		);

		writeln!(log, "Launch command: {cmd:#?}").context("Failed to write to launch log file")?;
		o.display(
			MessageContents::Property(
				"Launch command".to_string(),
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
	pub cwd: &'a Path,
	pub command: &'a str,
	pub jvm_args: &'a [String],
	pub main_class: Option<&'a str>,
	pub game_args: &'a [String],
	pub additional_env_vars: &'a HashMap<String, String>,
}

/// Options for launching after conversion from the deserialized version
#[derive(Debug)]
pub struct LaunchOptions {
	pub java: JavaKind,
	pub jvm_args: Vec<String>,
	pub game_args: Vec<String>,
	pub min_mem: Option<MemoryNum>,
	pub max_mem: Option<MemoryNum>,
	pub preset: ArgsPreset,
	pub env: HashMap<String, String>,
	pub wrapper: Option<String>,
	pub quick_play: QuickPlay,
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

/// Get the name of the launch log file
fn log_file_name(instance_id: &str) -> anyhow::Result<String> {
	Ok(format!("{instance_id}-{}.txt", utc_timestamp()?))
}

/// Get the path to the launch log file
fn log_file_path(instance_id: &str, paths: &Paths) -> anyhow::Result<PathBuf> {
	Ok(paths.launch_logs.join(log_file_name(instance_id)?))
}
