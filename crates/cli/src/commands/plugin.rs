use anyhow::Context;
use clap::Subcommand;
use color_print::cprintln;
use itertools::Itertools;

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
	},
	#[command(about = "Print useful information about a plugin")]
	Info { plugin: String },
}

pub async fn run(command: PluginSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match command {
		PluginSubcommand::List { raw } => list(data, raw).await,
		PluginSubcommand::Info { plugin } => info(data, plugin).await,
	}
}

async fn list(data: &mut CmdData, raw: bool) -> anyhow::Result<()> {
	data.ensure_config(!raw).await?;
	let config = data.config.get_mut();

	let lock = config.plugins.get_lock()?;
	for plugin in lock.manager.iter_plugins().sorted_by_key(|x| x.get_id()) {
		if raw {
			println!("{}", plugin.get_id());
		} else {
			cprintln!("{}<s>{}", HYPHEN_POINT, plugin.get_id());
		}
	}

	Ok(())
}

async fn info(data: &mut CmdData, plugin: String) -> anyhow::Result<()> {
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
