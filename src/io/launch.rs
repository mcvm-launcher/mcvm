use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context;
use color_print::cprintln;

use crate::data::config::instance::QuickPlay;
use crate::data::instance::launch::client::create_quick_play_args;
use crate::io::java::args::ArgsPreset;
use crate::io::java::{
	args::{MemoryArg, MemoryNum},
	JavaKind,
};
use crate::util::utc_timestamp;
use mcvm_shared::instance::Side;
use mcvm_shared::versions::VersionInfo;

use super::files::paths::Paths;

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
	pub fn generate_game_args(&self, version_info: &VersionInfo, side: Side) -> Vec<String> {
		let mut out = self.game_args.clone();

		if let Side::Client = side {
			out.extend(create_quick_play_args(&self.quick_play, version_info));
		}

		out
	}
}

/// Get the name of the launch log file
fn log_file_name(instance_name: &str) -> anyhow::Result<String> {
	Ok(format!("{instance_name}-{}.txt", utc_timestamp()?))
}

/// Get the path to the launch log file
fn log_file_path(instance_name: &str, paths: &Paths) -> anyhow::Result<PathBuf> {
	Ok(paths.launch_logs.join(log_file_name(instance_name)?))
}

/// Argument for the launch function
pub struct LaunchArgument<'a> {
	pub instance_name: &'a str,
	pub side: Side,
	pub options: &'a LaunchOptions,
	pub debug: bool,
	pub version_info: &'a VersionInfo,
	pub cwd: &'a Path,
	pub command: &'a str,
	pub jvm_args: &'a [String],
	pub main_class: Option<&'a str>,
	pub game_args: &'a [String],
	pub additional_env_vars: &'a HashMap<String, String>,
}

/// Launch the game
pub fn launch(paths: &Paths, arg: &LaunchArgument) -> anyhow::Result<()> {
	let mut log = File::create(log_file_path(arg.instance_name, paths)?)
		.context("Failed to open launch log file")?;
	let mut cmd = match &arg.options.wrapper {
		Some(wrapper) => {
			let mut cmd = Command::new(wrapper);
			cmd.arg(arg.command);
			cmd
		}
		None => Command::new(arg.command),
	};
	cmd.current_dir(arg.cwd);
	cmd.envs(arg.options.env.clone());
	cmd.envs(arg.additional_env_vars);

	cmd.args(arg.options.generate_jvm_args());
	cmd.args(arg.jvm_args);
	if let Some(main_class) = arg.main_class {
		cmd.arg(main_class);
	}
	cmd.args(arg.game_args);
	cmd.args(arg.options.generate_game_args(arg.version_info, arg.side));

	writeln!(log, "Launch command: {cmd:#?}").context("Failed to write to launch log file")?;
	if arg.debug {
		cprintln!("<s>Launch command:");
		cprintln!("<k!>{:#?}", cmd);
	}

	let mut child = cmd.spawn().context("Failed to spawn child process")?;
	child.wait().context("Failed to wait for child process")?;

	Ok(())
}
