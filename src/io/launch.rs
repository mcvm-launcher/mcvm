use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use color_print::cprintln;

use crate::data::config::instance::QuickPlay;
use crate::data::instance::Side;
use crate::io::java::args::ArgsPreset;
use crate::io::java::{
	args::{MemoryArg, MemoryNum},
	JavaKind,
};
use crate::util::versions::VersionPattern;

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
			Some(min) => match &self.max_mem {
				Some(max) => Some(MemoryNum::avg(min.clone(), max.clone())),
				None => None,
			},
			None => None,
		};
		out.extend(self.preset.generate_args(avg));

		out
	}

	/// Create the args for the game when launching
	pub fn generate_game_args(
		&self,
		version: &str,
		version_list: &[String],
		side: Side,
	) -> Vec<String> {
		let mut out = self.game_args.clone();

		if let Side::Client = side {
			match &self.quick_play {
				QuickPlay::World { .. } | QuickPlay::Realm { .. } | QuickPlay::Server { .. } => {
					let after_23w14a = VersionPattern::After(String::from("23w14a"))
						.matches_single(version, version_list);
					out.push(String::from("--quickPlayPath"));
					out.push(String::from("quickPlay/log.json"));
					match &self.quick_play {
						QuickPlay::None => {},
						QuickPlay::World { world } => {
							if after_23w14a {
								out.push(String::from("--quickPlaySingleplayer"));
								out.push(world.clone());
							} else {
								cprintln!("<y>Warning: World Quick Play has no effect before 23w14a (1.20)");
							}
						}
						QuickPlay::Realm { realm } => {
							if after_23w14a {
								out.push(String::from("--quickPlayRealms"));
								out.push(realm.clone());
							} else {
								cprintln!("<y>Warning: Realm Quick Play has no effect before 23w14a (1.20)");
							}
						}
						QuickPlay::Server { server, port } => {
							if after_23w14a {
								out.push(String::from("--quickPlayMultiplayer"));
								if let Some(port) = port {
									out.push(format!("{server}:{port}"));
								} else {
									out.push(server.clone());
								}
							} else {
								out.push(String::from("--server"));
								out.push(server.clone());
								if let Some(port) = port {
									out.push(String::from("--port"));
									out.push(port.to_string());
								}
							}
						}
					}
				}
				_ => {}
			}
		}

		out
	}
}

fn log_file_name(instance_name: &str) -> anyhow::Result<String> {
	let now = SystemTime::now();
	Ok(format!(
		"{instance_name}-{}.txt",
		now.duration_since(UNIX_EPOCH)?.as_secs()
	))
}

fn log_file_path(instance_name: &str, paths: &Paths) -> anyhow::Result<PathBuf> {
	Ok(paths.launch_logs.join(log_file_name(instance_name)?))
}

/// Launch the game
pub fn launch(
	paths: &Paths,
	instance_name: &str,
	side: Side,
	options: &LaunchOptions,
	debug: bool,
	version: &str,
	version_list: &[String],
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
	cmd.args(options.generate_game_args(version, version_list, side));

	writeln!(log, "Launch command: {cmd:#?}").context("Failed to write to launch log file")?;
	if debug {
		cprintln!("<s>Launch command:");
		cprintln!("<k!>{:#?}", cmd);
	}

	let mut child = cmd.spawn().context("Failed to spawn child process")?;
	child.wait().context("Failed to wait for child process")?;

	Ok(())
}
