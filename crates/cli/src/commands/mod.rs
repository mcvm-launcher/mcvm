mod config;
mod files;
mod instance;
mod package;
mod plugin;
mod profile;
mod user;

use anyhow::{bail, Context};
use clap::{Parser, Subcommand};
use color_print::{cformat, cprintln};

use mcvm::data::config::plugin::PluginManager;
use mcvm::data::config::{Config, ConfigDeser};
use mcvm::io::files::paths::Paths;
use mcvm::plugin::hooks;
use mcvm::shared::later::Later;
use mcvm::shared::output::{MCVMOutput, MessageContents, MessageLevel};

use self::config::ConfigSubcommand;
use self::files::FilesSubcommand;
use self::instance::InstanceSubcommand;
use self::package::PackageSubcommand;
use self::plugin::PluginSubcommand;
use self::profile::ProfileSubcommand;
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
	#[command(about = "Manage plugins")]
	#[clap(alias = "plug")]
	Plugin {
		#[command(subcommand)]
		command: PluginSubcommand,
	},
	#[command(about = "Manage config")]
	#[clap(alias = "cfg")]
	Config {
		#[command(subcommand)]
		command: ConfigSubcommand,
	},
	#[clap(external_subcommand)]
	External(Vec<String>),
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
		Command::Launch { instance } => instance::launch(instance, None, false, &mut data).await,
		Command::Version => {
			print_version();
			Ok(())
		}
		Command::Files { command } => files::run(command, &mut data).await,
		Command::Package { command } => package::run(command, &mut data).await,
		Command::Instance { command } => instance::run(command, &mut data).await,
		Command::Plugin { command } => plugin::run(command, &mut data).await,
		Command::Config { command } => config::run(command, &mut data).await,
		Command::External(args) => call_plugin_subcommand(args, &mut data).await,
	};

	if let Err(e) = &res {
		// Don't use the existing process or section
		data.output.end_process();
		data.output.end_section();
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
			let plugins = PluginManager::load(&self.paths, &mut self.output)
				.context("Failed to load plugins configuration")?;

			self.config.fill(
				Config::load(
					&Config::get_path(&self.paths),
					plugins,
					show_warnings,
					&self.paths,
					&mut self.output,
				)
				.context("Failed to load config")?,
			);
		}

		// Update the translation map
		if let Some(map) = self.config.get().get_translation_map()? {
			self.output.set_translation_map(map);
		}

		Ok(())
	}

	/// Get the raw deserialized config
	pub fn get_raw_config(&self) -> anyhow::Result<ConfigDeser> {
		let config =
			Config::open(&Config::get_path(&self.paths)).context("Failed to open config")?;

		Ok(config)
	}
}

/// Print the mcvm version
fn print_version() {
	let version = env!("CARGO_PKG_VERSION");
	let mcvm_version = mcvm::VERSION;
	cprintln!("CLI version: <g>{}</g>", version);
	cprintln!("MCVM version: <g>{}</g>", mcvm_version);
}

/// Call a plugin subcommand
async fn call_plugin_subcommand(args: Vec<String>, data: &mut CmdData) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get();

	// Make sure the subcommand is handled by one of the plugins
	let subcommand = args
		.first()
		.context("Subcommand does not have first argument")?;

	{
		let lock = config.plugins.get_lock()?;
		let exists = lock
			.manager
			.iter_plugins()
			.any(|x| x.get_manifest().subcommands.contains_key(subcommand));
		if !exists {
			bail!("Subcommand '{subcommand}' does not exist");
		}
	}

	let results = config
		.plugins
		.call_hook(hooks::Subcommand, &args, &data.paths, &mut data.output)
		.context("Plugin subcommand failed")?;
	for result in results {
		result.result(&mut data.output)?;
	}

	Ok(())
}
