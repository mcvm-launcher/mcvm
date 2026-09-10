mod account;
mod config;
mod files;
mod instance;
mod log;
mod modpack;
mod package;
mod plugin;
mod template;
mod r#try;
mod version;

use std::collections::HashMap;

use anyhow::{Context, bail};
use clap::{Parser, Subcommand};
use color_print::cformat;

use nitrolaunch::config::modifications::{ConfigModification, apply_modifications_and_write};
use nitrolaunch::config::{Config, is_first_run};
use nitrolaunch::config_crate::ConfigDeser;
use nitrolaunch::core::QuickPlayType;
use nitrolaunch::instance::transfer::{load_formats, migrate_instances};
use nitrolaunch::io::paths::Paths;
use nitrolaunch::plugin::PluginManager;
use nitrolaunch::plugin_crate::hook::hooks::{self, AddTranslations, SubcommandArg};
use nitrolaunch::shared::id::InstanceID;
use nitrolaunch::shared::lang::translate::TranslationKey;
use nitrolaunch::shared::later::Later;
use nitrolaunch::shared::nitro_executable::{NitroClientId, NitroExecutableRegistry};
use nitrolaunch::shared::output::{MessageContents, MessageLevel, NitroOutput};

use self::account::AccountSubcommand;
use self::config::ConfigSubcommand;
use self::files::FilesSubcommand;
use self::instance::InstanceSubcommand;
use self::log::LogSubcommand;
use self::modpack::ModpackSubcommand;
use self::package::PackageSubcommand;
use self::plugin::PluginSubcommand;
use self::template::TemplateSubcommand;
use self::r#try::TrySubcommand;
use self::version::VersionSubcommand;

use super::output::TerminalOutput;

#[derive(Debug, Subcommand)]
pub enum Command {
	#[command(about = "Launch instances to play the game")]
	Launch {
		/// An optional account to choose when launching
		#[arg(short, long)]
		account: Option<String>,
		/// Whether to launch in offline mode, skipping authentication. This only works
		/// if you have authenticated at least once
		#[arg(short, long)]
		offline: bool,
		/// Launch into a world or server. Can be either world:<world>, server:<ip> or realm:<realm>
		#[arg(short, long)]
		quick_play: Option<QuickPlayType>,
		/// The instance to launch
		instance: Option<String>,
	},
	#[command(about = "Manage instances")]
	#[clap(alias = "inst")]
	Instance {
		#[command(subcommand)]
		command: InstanceSubcommand,
	},
	#[command(about = "Do operations with instance templates")]
	#[clap(alias = "temp")]
	Template {
		#[command(subcommand)]
		command: TemplateSubcommand,
	},
	#[command(about = "Manage accounts and authentication")]
	Account {
		#[command(subcommand)]
		command: AccountSubcommand,
	},
	#[command(about = "Manage packages")]
	#[clap(alias = "pkg")]
	Package {
		#[command(subcommand)]
		command: PackageSubcommand,
	},
	#[command(about = "Manage plugins")]
	#[clap(alias = "plug")]
	Plugin {
		#[command(subcommand)]
		command: PluginSubcommand,
	},
	#[command(about = "Import modpacks")]
	Modpack {
		#[command(subcommand)]
		command: ModpackSubcommand,
	},
	#[command(about = "Import instances from another launcher")]
	Migrate {
		/// Which format to use
		format: Option<String>,
		/// Specific instances to migrate. Will migrate all if none are specified
		#[arg(short = 'i', long)]
		instances: Vec<String>,
		/// Whether to copy the instance files. By default, will link to the existing ones instead.
		#[arg(short = 'c', long)]
		copy: bool,
	},
	#[command(about = "Try out a new version or modpack using a temporary instance")]
	Try {
		#[command(subcommand)]
		command: TrySubcommand,
	},
	#[command(about = "Manage configuration")]
	#[clap(alias = "cfg", alias = "conf")]
	Config {
		#[command(subcommand)]
		command: ConfigSubcommand,
	},
	#[command(about = "Manage global log files for the launcher")]
	Log {
		#[command(subcommand)]
		command: LogSubcommand,
	},
	#[command(about = "Deal with files created by Nitrolaunch")]
	Files {
		#[command(subcommand)]
		command: FilesSubcommand,
	},
	#[command(about = "Get information about Minecraft versions")]
	#[command(long_about = "Use --version to get the Nitrolaunch version")]
	Version {
		#[command(subcommand)]
		command: VersionSubcommand,
	},
	#[clap(external_subcommand)]
	External(Vec<String>),
}

#[derive(Debug, Parser)]
#[command(version)]
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

	let log_level = get_log_level(&cli);

	// Prepare the command data
	let paths = Paths::new()
		.await
		.context("Failed to set up system paths")?;
	let mut output = TerminalOutput::new(&paths).context("Failed to set up output")?;

	if let Ok(mut exec_registry) = NitroExecutableRegistry::open(&paths.internal) {
		let _ = exec_registry.add_this(NitroClientId::Cli);
	}

	let _ = write_extra_commands_file(&paths);

	// First launch message
	if is_first_run(&paths) {
		output.display(MessageContents::Header("Welcome to Nitrolaunch!".into()));

		let install_default = output
			.prompt_yes_no(
				true,
				MessageContents::Simple(
					"You probably want to install the default set of plugins, \
				which includes features like Modrinth, Fabric, and stats tracking. \
				Would you like to do that now?"
						.into(),
				),
			)
			.await?;

		if install_default {
			let mut data = CmdData::new(paths.clone(), &mut output)?;
			data.output.set_log_level(log_level);

			if let Err(e) = plugin::install(
				&mut data,
				vec![
					"fabric_quilt".into(),
					"modrinth".into(),
					"smithed".into(),
					"curseforge".into(),
					"stats".into(),
					"docs".into(),
					"multimc_transfer".into(),
					"xmcl_transfer".into(),
				],
				None,
				Vec::new(),
			)
			.await
			{
				output.display(MessageContents::Error(format!(
					"Failed to install default plugins: {e}"
				)));
				bail!("");
			}

			output.display(
				MessageContents::Header("Use the nitro migrate command to use instances from an existing launcher, and make sure to join the Discord!".into()),
			);
			output.display(MessageContents::Hyperlink(
				"https://discord.gg/cgVapnVfZJ".into(),
			));
		}
	}

	let res = {
		let mut data = CmdData::new(paths, &mut output)?;
		data.output.set_log_level(log_level);

		match cli.command {
			Command::Account { command } => account::run(command, &mut data).await,
			Command::Launch {
				account,
				offline,
				quick_play,
				instance,
			} => instance::launch(instance, account, offline, quick_play, data).await,
			Command::Files { command } => files::run(command, &mut data).await,
			Command::Package { command } => package::run(command, data).await,
			Command::Instance { command } => instance::run(command, data).await,
			Command::Plugin { command } => plugin::run(command, &mut data).await,
			Command::Config { command } => config::run(command, &mut data).await,
			Command::Template { command } => template::run(command, &mut data).await,
			Command::Modpack { command } => modpack::run(command, &mut data).await,
			Command::Migrate {
				format,
				instances,
				copy,
			} => migrate(format, instances, copy, &mut data).await,
			Command::Log { command } => log::run(command, &mut data).await,
			Command::Try { command } => r#try::run(command, &mut data).await,
			Command::Version { command } => version::run(command, &mut data).await,
			Command::External(args) => call_plugin_subcommand(args, None, &mut data).await,
		}
	};

	if let Err(e) = &res {
		// Don't use the existing process or section
		output.end_process();
		output.end_section();
		output.display(MessageContents::Error(format!("{e:?}")));
	}

	output.shutdown().await;

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
pub struct CmdData<'a> {
	pub paths: Paths,
	pub config: Later<Config>,
	pub output: &'a mut TerminalOutput,
}

impl<'a> CmdData<'a> {
	pub fn new(paths: Paths, output: &'a mut TerminalOutput) -> anyhow::Result<Self> {
		Ok(Self {
			paths,
			config: Later::new(),
			output,
		})
	}

	/// Ensure that the config is loaded
	pub async fn ensure_config(&mut self, show_warnings: bool) -> anyhow::Result<()> {
		if self.config.is_empty() {
			let plugins = PluginManager::load(&self.paths, self.output)
				.await
				.context("Failed to load plugins configuration")?;

			self.config.fill(
				Config::load(
					&Config::get_path(&self.paths),
					plugins,
					show_warnings,
					&self.paths,
					crate::secrets::get_ms_client_id(),
					self.output,
				)
				.await
				.context("Failed to load config")?,
			);
		}

		// Update the translation map from plugins
		let mut results = self
			.config
			.get()
			.plugins
			.call_hook(AddTranslations, &(), &self.paths, self.output)
			.await
			.context("Failed to get extra translations from plugins")?;

		while let Some(mut result) = results.next_result(self.output).await? {
			let map = result.remove(&self.config.get().prefs.language);
			if let Some(map) = map {
				self.output.set_translation_map(map);
			}
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

/// Runs instance migration
async fn migrate(
	format: Option<String>,
	instances: Vec<String>,
	copy: bool,
	data: &mut CmdData<'_>,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get();

	// Figure out the format
	let formats = load_formats(&config.plugins, &data.paths, data.output)
		.await
		.context("Failed to get available transfer formats")?;

	let format = if let Some(format) = &format {
		format
	} else {
		let options: Vec<_> = formats
			.formats
			.iter()
			.filter(|x| x.1.supports_migration())
			.map(|x| x.0)
			.collect();
		if options.is_empty() {
			bail!(
				"{}",
				data.output.translate(TranslationKey::NoTransferFormats)
			);
		}
		inquire::Select::new("What launcher do you want to import from?", options).prompt()?
	};

	let new_configs = migrate_instances(
		format,
		Some(instances).filter(|x| !x.is_empty()),
		!copy,
		&formats,
		&config.plugins,
		&data.paths,
		data.output,
	)
	.await
	.context("Failed to migrate instances")?;

	let mut config2 = data.get_raw_config()?;

	for key in new_configs.keys() {
		if config2
			.instances
			.contains_key(&InstanceID::from(key.clone()))
		{
			bail!("Duplicate instance ID {key}");
		}
	}

	apply_modifications_and_write(
		&mut config2,
		new_configs
			.into_iter()
			.map(|(id, config)| ConfigModification::AddInstance(id.into(), config))
			.collect(),
		&data.paths,
		&config.plugins,
		data.output,
	)
	.await
	.context("Failed to write modified config")?;

	Ok(())
}

/// Call a plugin subcommand
async fn call_plugin_subcommand(
	args: Vec<String>,
	supercommand: Option<&str>,
	data: &mut CmdData<'_>,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get();

	// Make sure the subcommand is handled by one of the plugins
	let subcommand = args
		.first()
		.context("Subcommand does not have first argument")?;

	let plugin = {
		let lock = config.plugins.get_lock().await;
		let plugin = lock.manager.get_subcommand(subcommand, supercommand);
		let Some(plugin) = plugin else {
			bail!("Subcommand '{subcommand}' does not exist");
		};
		plugin
	};

	let mut instance_configs = HashMap::new();
	for (id, instance) in &config.instances {
		instance_configs.insert(id.clone(), instance.config().clone());
	}

	let arg = SubcommandArg {
		args,
		supercommand: supercommand.map(|x| x.to_string()),
		instances: instance_configs,
	};

	let result = config
		.plugins
		.call_hook_on_plugin(hooks::Subcommand, &plugin, &arg, &data.paths, data.output)
		.await
		.context("Plugin subcommand failed")?;
	if let Some(result) = result {
		result.result(data.output).await?;
	}

	Ok(())
}

/// Writes a file that can be sourced to provide extra shell commands
fn write_extra_commands_file(paths: &Paths) -> anyhow::Result<()> {
	let commands = r#"
nitro-instance-cd () {
	cd $(nitro inst dir $1)
}
nitro-inst-cd () {
	cd $(nitro inst dir $1)
}
	"#;

	let file = paths.internal.join("commands.sh");
	std::fs::write(file, commands).context("Failed to write")
}
