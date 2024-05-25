use super::CmdData;

use anyhow::Context;
use clap::Subcommand;
use mcvm::data::config::{plugin::PluginManager, Config};

use std::{path::PathBuf, process::Command};

#[derive(Debug, Subcommand)]
pub enum ConfigSubcommand {
	#[command(about = "Edit config using your default text editor")]
	Edit {},
	#[command(about = "Edit plugin config using your default text editor")]
	EditPlugins {},
}

pub async fn run(subcommand: ConfigSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match subcommand {
		ConfigSubcommand::Edit {} => edit(data).await,
		ConfigSubcommand::EditPlugins {} => edit_plugins(data).await,
	}
}

async fn edit(data: &mut CmdData) -> anyhow::Result<()> {
	let path = Config::get_path(&data.paths);

	Config::create_default(&path).context("Failed to create default config")?;

	edit_text(path).context("Failed to edit config")?;

	Ok(())
}

async fn edit_plugins(data: &mut CmdData) -> anyhow::Result<()> {
	let path = PluginManager::get_path(&data.paths);

	PluginManager::create_default(&data.paths).context("Failed to create default config")?;

	edit_text(path).context("Failed to edit config")?;

	Ok(())
}

/// Run the text editor on the user's system
fn edit_text(path: PathBuf) -> anyhow::Result<()> {
	#[cfg(target_os = "linux")]
	let mut command = {
		let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".into());
		Command::new(editor)
	};
	#[cfg(target_os = "windows")]
	let mut command = Command::new("notepad");
	#[cfg(target_os = "macos")]
	let mut command = Command::new("open");
	#[cfg(target_os = "macos")]
	command.arg("-t");

	command.arg(path);

	command.spawn()?.wait()?;

	Ok(())
}
