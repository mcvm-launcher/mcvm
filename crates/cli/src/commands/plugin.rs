use anyhow::{bail, Context};
use clap::Subcommand;
use color_print::cprintln;
use mcvm::core::io::json_from_file;
use mcvm::plugin::install::get_verified_plugins;
use mcvm::plugin::PluginManager;
use mcvm::plugin_crate::plugin::PluginManifest;
use mcvm::shared::lang::translate::TranslationKey;
use mcvm::shared::output::{MCVMOutput, MessageContents, MessageLevel};
use reqwest::Client;

use super::CmdData;
use crate::output::HYPHEN_POINT;

#[derive(Debug, Subcommand)]
pub enum PluginSubcommand {
	#[command(about = "List all enabled plugins")]
	#[clap(alias = "ls")]
	List {
		/// Whether to remove formatting and warnings from the output
		#[arg(short, long)]
		raw: bool,
		/// Whether to filter only the loaded plugins
		#[arg(short, long)]
		loaded: bool,
	},
	#[command(about = "Print useful information about a plugin")]
	Info { plugin: String },
	#[command(about = "Install a plugin from the verified list")]
	Install {
		plugin: String,
		/// The version of the plugin to install
		#[arg(short, long)]
		version: Option<String>,
	},
}

pub async fn run(command: PluginSubcommand, data: &mut CmdData<'_>) -> anyhow::Result<()> {
	match command {
		PluginSubcommand::List { raw, loaded } => list(data, raw, loaded).await,
		PluginSubcommand::Info { plugin } => info(data, plugin).await,
		PluginSubcommand::Install { plugin, version } => install(data, plugin, version).await,
	}
}

async fn list(data: &mut CmdData<'_>, raw: bool, loaded: bool) -> anyhow::Result<()> {
	data.ensure_config(!raw).await?;
	let config = data.config.get_mut();

	let mut available_plugins = PluginManager::get_available_plugins(&data.paths)
		.context("Failed to get list of available plugins")?;
	available_plugins.sort_by_key(|x| x.0.clone());

	let lock = config.plugins.get_lock()?;
	let loaded_plugins: Vec<_> = lock.manager.iter_plugins().map(|x| x.get_id()).collect();

	for (plugin_id, plugin_path) in available_plugins {
		let is_loaded = loaded_plugins.contains(&&plugin_id);
		if loaded && !is_loaded {
			continue;
		}

		if raw {
			println!("{}", plugin_id);
		} else {
			if is_loaded {
				cprintln!("{}<s>{}</> [Loaded]", HYPHEN_POINT, plugin_id);
			} else {
				let is_valid = json_from_file::<PluginManifest>(plugin_path).is_ok();
				if is_valid {
					cprintln!("{}{} [Unloaded]", HYPHEN_POINT, plugin_id);
				} else {
					cprintln!("{}<r>{} [Invalid]", HYPHEN_POINT, plugin_id);
				}
			}
		}
	}

	Ok(())
}

async fn info(data: &mut CmdData<'_>, plugin: String) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let lock = config.plugins.get_lock()?;
	let plugin = lock
		.manager
		.iter_plugins()
		.find(|x| x.get_id() == &plugin)
		.context("Plugin does not exist")?;

	cprintln!(
		"<s>Plugin <b>{}</>:",
		plugin
			.get_manifest()
			.name
			.as_ref()
			.unwrap_or(plugin.get_id())
	);
	if let Some(description) = &plugin.get_manifest().description {
		cprintln!("{}", description);
	}
	cprintln!("{}<s>ID:</> {}", HYPHEN_POINT, plugin.get_id());

	Ok(())
}

async fn install(
	data: &mut CmdData<'_>,
	plugin: String,
	version: Option<String>,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let client = Client::new();

	let verified_list = get_verified_plugins(&client)
		.await
		.context("Failed to get verified plugin list")?;
	let Some(plugin) = verified_list.get(&plugin) else {
		bail!("Unknown plugin '{plugin}'");
	};

	data.output.display(
		MessageContents::StartProcess(
			data.output
				.translate(TranslationKey::StartInstallingPlugin)
				.to_string(),
		),
		MessageLevel::Important,
	);
	plugin
		.install(version.as_deref(), &data.paths, &client)
		.await
		.context("Failed to install plugin")?;
	data.output.display(
		MessageContents::Success(
			data.output
				.translate(TranslationKey::FinishInstallingPlugin)
				.to_string(),
		),
		MessageLevel::Important,
	);

	Ok(())
}
