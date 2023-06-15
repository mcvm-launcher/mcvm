mod files;
mod instance;
mod package;
mod profile;
mod user;
mod snapshot;

use anyhow::Context;
use clap::{Parser, Subcommand};
use color_print::cprintln;

use mcvm::data::config::Config;
use mcvm::io::files::paths::Paths;
use mcvm::io::Later;

use self::files::FilesSubcommand;
use self::instance::InstanceSubcommand;
use self::package::PackageSubcommand;
use self::profile::ProfileSubcommand;
use self::snapshot::SnapshotSubcommand;
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

	pub async fn ensure_config(&mut self, show_warnings: bool) -> anyhow::Result<()> {
		if self.config.is_empty() {
			self.ensure_paths()
				.await
				.context("Failed to set up directories")?;
			self.config.fill(
				Config::load(&Config::get_path(self.paths.get()), show_warnings)
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
	}
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
		Command::Launch { instance } => instance::launch(&instance, false, None, None, data).await,
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
