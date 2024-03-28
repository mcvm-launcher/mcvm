mod files;
mod instance;
mod package;
mod profile;
mod snapshot;
mod tool;
mod user;

use anyhow::{bail, Context};
use clap::{Parser, Subcommand};
use color_print::{cformat, cprintln};

use mcvm::data::config::Config;
use mcvm::io::files::paths::Paths;
use mcvm::shared::later::Later;
use mcvm::shared::output::{MCVMOutput, MessageContents, MessageLevel};

use self::files::FilesSubcommand;
use self::instance::InstanceSubcommand;
use self::package::PackageSubcommand;
use self::profile::ProfileSubcommand;
use self::snapshot::SnapshotSubcommand;
use self::tool::ToolSubcommand;
use self::user::UserSubcommand;

use super::output::TerminalOutput;

#[derive(Debug, Subcommand)]
pub enum Command {
	#[command(about = "Manage profiles")]
	#[clap(alias = "prof")]
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
		instance: Option<String>,
	},
	#[command(about = "Print the mcvm version")]
	Version,
	#[command(about = "Deal with files created by mcvm")]
	Files {
		#[command(subcommand)]
		command: FilesSubcommand,
	},
	#[command(about = "Manage packages")]
	#[clap(alias = "pkg")]
	Package {
		#[command(subcommand)]
		command: PackageSubcommand,
	},
	#[command(about = "Manage instances")]
	#[clap(alias = "inst")]
	Instance {
		#[command(subcommand)]
		command: InstanceSubcommand,
	},
	#[command(about = "Manage snapshots for instances")]
	Snapshot {
		#[command(subcommand)]
		command: SnapshotSubcommand,
	},
	#[command(about = "Access different tools and tests included with mcvm")]
	Tool {
		#[command(subcommand)]
		command: ToolSubcommand,
	},
}

#[derive(Debug, Parser)]
pub struct Cli {
	#[command(subcommand)]
	command: Command,
	#[arg(short, long)]
	debug: bool,
	#[arg(short = 'D', long)]
	trace: bool,
}

/// Run the command line interface
pub async fn run_cli() -> anyhow::Result<()> {
	// Parse the CLI
	let cli = Cli::try_parse();
	if let Err(e) = &cli {
		if let clap::error::ErrorKind::DisplayHelp
		| clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
		| clap::error::ErrorKind::DisplayVersion = e.kind()
		{
			println!("{e}");
			return Ok(());
		} else {
			eprintln!("{}", cformat!("<r>{e}"));
			bail!("");
		}
	}
	let cli = cli?;

	// Prepare the command data
	let mut data = CmdData::new().await?;
	let log_level = get_log_level(&cli);
	data.output.set_log_level(log_level);

	let res = match cli.command {
		Command::Profile { command } => profile::run(command, &mut data).await,
		Command::User { command } => user::run(command, &mut data).await,
		Command::Launch { instance } => instance::launch(instance, None, &mut data).await,
		Command::Version => {
			print_version();
			Ok(())
		}
		Command::Files { command } => files::run(command, &mut data).await,
		Command::Package { command } => package::run(command, &mut data).await,
		Command::Instance { command } => instance::run(command, &mut data).await,
		Command::Snapshot { command } => snapshot::run(command, &mut data).await,
		Command::Tool { command } => tool::run(command, &mut data).await,
	};

	if let Err(e) = &res {
		data.output.display(
			MessageContents::Error(format!("{e:?}")),
			MessageLevel::Important,
		);
	}

	res
}

/// Get the log level based on the debug options
fn get_log_level(cli: &Cli) -> MessageLevel {
	if cli.trace {
		MessageLevel::Trace
	} else if cli.debug {
		MessageLevel::Debug
	} else {
		MessageLevel::Important
	}
}

/// Data passed to commands
pub struct CmdData {
	pub paths: Paths,
	pub config: Later<Config>,
	pub output: TerminalOutput,
}

impl CmdData {
	pub async fn new() -> anyhow::Result<Self> {
		let paths = Paths::new()
			.await
			.context("Failed to set up system paths")?;
		let output = TerminalOutput::new(&paths).context("Failed to set up output")?;
		Ok(Self {
			paths,
			config: Later::new(),
			output,
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

/// Print the mcvm version
fn print_version() {
	let version = env!("CARGO_PKG_VERSION");
	let mcvm_version = mcvm::VERSION;
	cprintln!("CLI version: <g>{}</g>", version);
	cprintln!("MCVM version: <g>{}</g>", mcvm_version);
}
