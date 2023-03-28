mod files;
mod launch;
pub mod package;
mod profile;
mod user;

use clap::{Subcommand, Parser};
use color_print::cprintln;

use crate::data::config::Config;
use crate::io::files::paths::Paths;

use self::package::PackageSubcommand;
use self::files::FilesSubcommand;
use self::user::UserSubcommand;
use self::profile::ProfileSubcommand;

// Data passed to commands
pub struct CmdData {
	pub paths: Option<Paths>,
	pub config: Option<Config>,
}

impl CmdData {
	pub fn new() -> Self {
		Self {
			paths: None,
			config: None,
		}
	}

	pub fn ensure_paths(&mut self) -> anyhow::Result<()> {
		if self.paths.is_none() {
			self.paths = Some(Paths::new()?);
		}
		Ok(())
	}

	pub fn ensure_config(&mut self) -> anyhow::Result<()> {
		if self.config.is_none() {
			self.ensure_paths()?;
			if let Some(paths) = &self.paths {
				self.config = Some(Config::load(&paths.project.config_dir().join("mcvm.json"))?);
			}
		}
		Ok(())
	}
}

#[derive(Debug, Subcommand)]
pub enum Command {
	Profile {
		#[command(subcommand)]
		command: ProfileSubcommand,
	},
	User {
		#[command(subcommand)]
		command: UserSubcommand,
	},
	Launch {
		#[arg(short, long)]
		debug: bool,
		instance: String,
	},
	Version,
	Files {
		#[command(subcommand)]
		command: FilesSubcommand,
	},
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
		Command::User { command } => user::run(command, data),
		Command::Launch { debug, instance } => launch::run(&instance, debug, data).await,
		Command::Version => Ok(cprintln!("mcvm version <g>{}</g>", env!("CARGO_PKG_VERSION"))),
		Command::Files { command } => files::run(command, data),
		Command::Package { command } => package::run(command, data).await,
	}
}
