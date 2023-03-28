use super::CmdData;

use clap::Subcommand;
use color_print::cprintln;

use std::fs;

#[derive(Debug, Subcommand)]
pub enum FilesSubcommand {
	Remove,
}

pub fn remove(data: &mut CmdData) -> anyhow::Result<()> {
	data.ensure_paths()?;
	if let Some(paths) = &data.paths {
		cprintln!("<g>Removing internal files...");
		fs::remove_dir_all(&paths.internal)?;
	}
	Ok(())
}

pub fn run(subcommand: FilesSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match subcommand {
		FilesSubcommand::Remove => remove(data),
	}
}

// pub fn run(argc: usize, argv: &[String], data: &mut CmdData) -> anyhow::Result<()> {
// 	if argc == 0 {
// 		help();
// 		return Ok(());
// 	}

// 	match argv[0].as_str() {
// 		"remove" => remove(data)?,
// 		cmd => cprintln!("<r>Unknown subcommand {}", cmd),
// 	}

// 	Ok(())
// }
