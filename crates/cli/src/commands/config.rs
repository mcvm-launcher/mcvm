use super::CmdData;

use anyhow::Context;
use clap::Subcommand;
use mcvm::data::config::Config;

use std::{path::PathBuf, process::Command};

#[derive(Debug, Subcommand)]
pub enum ConfigSubcommand {
	#[command(about = "Edit config using your default text editor")]
	Edit {},
}

pub async fn run(subcommand: ConfigSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match subcommand {
		ConfigSubcommand::Edit {} => edit(data).await,
	}
}

pub async fn edit(data: &mut CmdData) -> anyhow::Result<()> {
	let path = Config::get_path(&data.paths);

	edit_text(path).context("Failed to edit config")?;

	Ok(())
}

/// Run the text editor on the user's system
fn edit_text(path: PathBuf) -> anyhow::Result<()> {
	#[cfg(target_family = "unix")]
	let mut command =
		Command::new(std::env::var("EDITOR").context("Failed to get editor environment variable")?);
	#[cfg(target_os = "windows")]
	let mut command = Command::new("notepad");
	#[cfg(target_os = "macos")]
	let mut command = Command::new("open").arg("-t");

	command.arg(path);

	command.spawn()?.wait()?;

	Ok(())
}
