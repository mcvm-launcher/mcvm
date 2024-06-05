use super::CmdData;

use anyhow::Context;
use clap::Subcommand;
use color_print::cprintln;

use std::fs;

#[derive(Debug, Subcommand)]
pub enum FilesSubcommand {
	#[command(
		about = "Remove cached files",
		long_about = "Remove cached files downloaded by mcvm. This does not include
files in your instances or any other user data."
	)]
	Remove {
		/// Whether to remove the internal data directory as well.
		/// Don't do this unless something isn't working.
		#[arg(short, long)]
		data: bool,
	},
}

pub async fn run(subcommand: FilesSubcommand, data: &mut CmdData<'_>) -> anyhow::Result<()> {
	match subcommand {
		FilesSubcommand::Remove { data: remove_data } => remove(data, remove_data).await,
	}
}

pub async fn remove(data: &mut CmdData<'_>, remove_data: bool) -> anyhow::Result<()> {
	cprintln!("<g>Removing cached files...");
	fs::remove_dir_all(data.paths.project.cache_dir())
		.context("Failed to remove cache directory")?;
	if remove_data {
		cprintln!("<g>Removing internal files...");
		fs::remove_dir_all(&data.paths.internal)
			.context("Failed to remove internal data directory")?;
	}

	Ok(())
}
