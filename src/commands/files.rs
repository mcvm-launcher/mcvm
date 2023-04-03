use super::CmdData;

use anyhow::Context;
use clap::Subcommand;
use color_print::cprintln;

use std::fs;

#[derive(Debug, Subcommand)]
pub enum FilesSubcommand {
	#[command(
		about = "Remove all cached and downloaded files",
		long_about = "Remove game files and cached files downloaded by mcvm. This does not include
files in your instances or any other user data. Don't do this unless something isn't working."
	)]
	Remove,
}

pub fn remove(data: &mut CmdData) -> anyhow::Result<()> {
	data.ensure_paths()?;
	if let Some(paths) = &data.paths {
		cprintln!("<g>Removing internal files...");
		fs::remove_dir_all(&paths.internal).context("Failed to remove internal data directory")?;
	}
	Ok(())
}

pub fn run(subcommand: FilesSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match subcommand {
		FilesSubcommand::Remove => remove(data),
	}
}
