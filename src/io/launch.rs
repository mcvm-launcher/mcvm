use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use color_print::cprintln;

use crate::io::java::args::ArgsPreset;
use crate::io::java::{
	args::{MemoryArg, MemoryNum},
	JavaKind,
};

use super::files::paths::Paths;

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
			Some(min) => match &self.max_mem {
				Some(max) => Some(MemoryNum::avg(min.clone(), max.clone())),
				None => None
			}
			None => None	
		};
		out.extend(self.preset.generate_args(avg));

		out
	}
}

fn log_file_name(instance_name: &str) -> anyhow::Result<String> {
	let now = SystemTime::now();
	Ok(format!("{instance_name}-{}.txt", now.duration_since(UNIX_EPOCH)?.as_secs()))
}

fn log_file_path(instance_name: &str, paths: &Paths) -> anyhow::Result<PathBuf> {
	Ok(paths.launch_logs.join(log_file_name(instance_name)?))
}

/// Launch the game
pub fn launch(
	paths: &Paths,
	instance_name: &str,
	options: &LaunchOptions,
	debug: bool,
	cwd: &Path,
	command: &str,
	jvm_args: &[String],
	main_class: Option<&str>,
	game_args: &[String],
) -> anyhow::Result<()> {
	let mut log = File::create(log_file_path(instance_name, paths)?)
		.context("Failed to open launch log file")?;
	let mut cmd = match &options.wrapper {
		Some(wrapper) => {
			let mut cmd = Command::new(wrapper);
			cmd.arg(command);
			cmd
		}
		None => Command::new(command),
	};
	cmd.current_dir(cwd);
	cmd.envs(options.env.clone());
	
	cmd.args(options.generate_jvm_args());
	cmd.args(jvm_args);
	if let Some(main_class) = main_class {
		cmd.arg(main_class);
	}
	cmd.args(game_args);
	cmd.args(&options.game_args);

	writeln!(log, "Launch command: {cmd:#?}").context("Failed to write to launch log file")?;
	if debug {
		cprintln!("<s>Launch command:");
		cprintln!("<k!>{:#?}", cmd);
	}

	let mut child = cmd.spawn().context("Failed to spawn child process")?;
	child.wait().context("Failed to wait for child process")?;
	
	Ok(())
}
