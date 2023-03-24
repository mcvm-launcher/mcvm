use super::lib::CmdData;
use crate::util::print::HYPHEN_POINT;

use color_print::cprintln;

use std::fs;

static REMOVE_HELP: &str = "Remove files downloaded by mcvm, not including any user data";

pub fn help() {
	cprintln!("<i>files:</i> Manage mcvm's internals");
	cprintln!("<s>Usage:</s> mcvm files <k!><<subcommand>> [options]</k!>");
	cprintln!();
	cprintln!("<s>Subcommands:");
	cprintln!("{}<i,c>remove:</i,c> {}", HYPHEN_POINT, REMOVE_HELP);
}

pub fn remove(data: &mut CmdData) -> anyhow::Result<()> {
	data.ensure_paths()?;
	if let Some(paths) = &data.paths {
		cprintln!("<g>Removing internal files...");
		fs::remove_dir_all(&paths.internal)?;
	}
	Ok(())
}

pub fn run(argc: usize, argv: &[String], data: &mut CmdData) -> anyhow::Result<()> {
	if argc == 0 {
		help();
		return Ok(());
	}

	match argv[0].as_str() {
		"remove" => remove(data)?,
		cmd => cprintln!("<r>Unknown subcommand {}", cmd),
	}

	Ok(())
}
