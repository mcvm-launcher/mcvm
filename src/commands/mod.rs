mod files;
mod launch;
pub mod package;
mod profile;
mod user;

use anyhow::Context;
use clap::{Parser, Subcommand};
use color_print::cprintln;

use crate::data::config::Config;
use crate::io::files::paths::Paths;
use crate::io::Later;

use self::files::FilesSubcommand;
use self::package::PackageSubcommand;
use self::profile::ProfileSubcommand;
use self::user::UserSubcommand;

// Data passed to commands
pub struct CmdData {
	pub paths: Later<Paths>,
	pub config: Later<Config>,
}

impl CmdData {
	pub fn new() -> Self {
		Self {
			paths: Later::new(),
			config: Later::new(),
		}
	}

	pub async fn ensure_paths(&mut self) -> anyhow::Result<()> {
		if self.paths.is_empty() {
			self.paths.fill(Paths::new().await?);
		}
		Ok(())
	}

	pub async fn ensure_config(&mut self) -> anyhow::Result<()> {
		if self.config.is_empty() {
			self.ensure_paths()
				.await
				.context("Failed to set up directories")?;
			self.config.fill(
				Config::load(&self.paths.get().project.config_dir().join("mcvm.json"))
					.context("Failed to load config")?,
			);
		}

		Ok(())
	}
}

#[derive(Debug, Subcommand)]
pub enum Command {
	#[command(about = "Manage profiles")]
	Profile {
		#[command(subcommand)]
		command: ProfileSubcommand,
	},
	#[command(about = "Manage users and authentication")]
	User {
		#[command(subcommand)]
		command: UserSubcommand,
	},
	#[command(about = "Launch instances to play the game")]
	Launch {
		/// Whether to print the command that was generated when launching
		#[arg(short, long)]
		debug: bool,
		/// The instance to launch
		instance: String,
	},
	#[command(about = "Print the mcvm version")]
	Version,
	#[command(about = "Deal with files created by mcvm")]
	Files {
		#[command(subcommand)]
		command: FilesSubcommand,
	},
	#[command(about = "Manage packages")]
	Package {
		#[command(subcommand)]
		command: PackageSubcommand,
	},
}

#[derive(Debug, Parser)]
pub struct Cli {
	#[command(subcommand)]
	command: Command,
}

pub async fn run_cli(data: &mut CmdData) -> anyhow::Result<()> {
	let cli = Cli::try_parse()?;
	match cli.command {
		Command::Profile { command } => profile::run(command, data).await,
		Command::User { command } => user::run(command, data).await,
		Command::Launch { debug, instance } => launch::run(&instance, debug, data).await,
		Command::Version => Ok(cprintln!(
			"mcvm version <g>{}</g>",
			env!("CARGO_PKG_VERSION")
		)),
		Command::Files { command } => files::run(command, data).await,
		Command::Package { command } => package::run(command, data).await,
	}
}
