mod files;
mod instance;
mod package;
mod profile;
mod snapshot;
mod user;

use anyhow::Context;
use clap::{Parser, Subcommand};
use color_print::cprintln;

use mcvm::data::config::Config;
use mcvm::io::files::paths::Paths;
use mcvm_shared::later::Later;

use self::files::FilesSubcommand;
use self::instance::InstanceSubcommand;
use self::package::PackageSubcommand;
use self::profile::ProfileSubcommand;
use self::snapshot::SnapshotSubcommand;
use self::user::UserSubcommand;

use super::output::TerminalOutput;

/// Data passed to commands
pub struct CmdData {
	pub paths: Paths,
	pub config: Later<Config>,
	pub output: TerminalOutput,
}

impl CmdData {
	pub async fn new() -> anyhow::Result<Self> {
		Ok(Self {
			paths: Paths::new().await?,
			config: Later::new(),
			output: TerminalOutput::new(),
		})
	}

	/// Ensure that the config is loaded
	pub async fn ensure_config(&mut self, show_warnings: bool) -> anyhow::Result<()> {
		if self.config.is_empty() {
			self.config.fill(
				Config::load(
					&Config::get_path(&self.paths),
					show_warnings,
					&mut self.output,
				)
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
	#[command(about = "Manage instances")]
	Instance {
		#[command(subcommand)]
		command: InstanceSubcommand,
	},
	#[command(about = "Manage snapshots for instances")]
	Snapshot {
		#[command(subcommand)]
		command: SnapshotSubcommand,
	},
}

#[derive(Debug, Parser)]
pub struct Cli {
	#[command(subcommand)]
	command: Command,
}

/// Print the mcvm version
fn print_version() {
	let version = env!("CARGO_PKG_VERSION");
	cprintln!("mcvm version <g>{}</g>", version);
}

/// Run the command line interface
pub async fn run_cli(data: &mut CmdData) -> anyhow::Result<()> {
	let cli = Cli::try_parse();
	if let Err(e) = &cli {
		if let clap::error::ErrorKind::DisplayHelp = e.kind() {
			println!("{e}");
			return Ok(());
		}
	}
	let cli = cli?;
	match cli.command {
		Command::Profile { command } => profile::run(command, data).await,
		Command::User { command } => user::run(command, data).await,
		Command::Launch { instance } => instance::launch(&instance, false, None, data).await,
		Command::Version => {
			print_version();
			Ok(())
		}
		Command::Files { command } => files::run(command, data).await,
		Command::Package { command } => package::run(command, data).await,
		Command::Instance { command } => instance::run(command, data).await,
		Command::Snapshot { command } => snapshot::run(command, data).await,
	}
}
