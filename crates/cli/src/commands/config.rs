use super::CmdData;

use anyhow::Context;
use clap::Subcommand;
use mcvm::config::{plugin::PluginManager, Config};

use std::{path::PathBuf, process::Command};

#[derive(Debug, Subcommand)]
pub enum ConfigSubcommand {
	#[command(about = "Edit config using your default text editor")]
	Edit,
	#[command(about = "Edit plugin config using your default text editor")]
	EditPlugins,
	#[command(about = "Backup configuration files to identical copies")]
	Backup,
}

pub async fn run(subcommand: ConfigSubcommand, data: &mut CmdData<'_>) -> anyhow::Result<()> {
	match subcommand {
		ConfigSubcommand::Edit => edit(data).await,
		ConfigSubcommand::EditPlugins => edit_plugins(data).await,
		ConfigSubcommand::Backup => backup(data).await,
	}
}

async fn edit(data: &mut CmdData<'_>) -> anyhow::Result<()> {
	let path = Config::get_path(&data.paths);

	Config::create_default(&path).context("Failed to create default config")?;

	edit_text(path).context("Failed to edit config")?;

	Ok(())
}

async fn edit_plugins(data: &mut CmdData<'_>) -> anyhow::Result<()> {
	let path = PluginManager::get_config_path(&data.paths);

	PluginManager::create_default(&data.paths).context("Failed to create default config")?;

	edit_text(path).context("Failed to edit config")?;

	Ok(())
}

async fn backup(data: &mut CmdData<'_>) -> anyhow::Result<()> {
	let config_path = Config::get_path(&data.paths);
	let plugins_path = PluginManager::get_config_path(&data.paths);

	Config::create_default(&config_path).context("Failed to create default config")?;
	PluginManager::create_default(&data.paths).context("Failed to create default plugin config")?;

	let mut backup_config_path = config_path.clone();
	let mut backup_plugins_path = plugins_path.clone();
	backup_config_path.set_extension("json.bak");
	backup_plugins_path.set_extension("json.bak");
	std::fs::copy(config_path, backup_config_path).context("Failed to backup config file")?;
	std::fs::copy(plugins_path, backup_plugins_path)
		.context("Failed to backup plugin config file")?;

	Ok(())
}

/// Run the text editor on the user's system
fn edit_text(path: PathBuf) -> anyhow::Result<()> {
	#[cfg(target_os = "linux")]
	let mut command = {
		let editor = std::env::var("EDITOR").unwrap_or_else(|_| {
			// Pick the best Vim-style editor based on which one is available
			if which::which("nvim").is_ok_and(|path| path.exists()) {
				"nvim".into()
			} else {
				"vim".into()
			}
		});
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
