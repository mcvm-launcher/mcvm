use anyhow::Context;
use clap::Subcommand;
use color_print::cprintln;
use mcvm::{
	config::plugin::PluginManager, core::io::json_from_file, plugin::plugin::PluginManifest,
};

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
}

pub async fn run(command: PluginSubcommand, data: &mut CmdData<'_>) -> anyhow::Result<()> {
	match command {
		PluginSubcommand::List { raw, loaded } => list(data, raw, loaded).await,
		PluginSubcommand::Info { plugin } => info(data, plugin).await,
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
