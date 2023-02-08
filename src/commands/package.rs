use super::lib::{CmdData, CmdError};
use crate::util::print::HYPHEN_POINT;
use crate::package::PkgKind;

use color_print::{cprintln, cprint};

static LIST_HELP: &str = "List all installed packages";

pub fn help() {
	cprintln!("<i>package:</i> Manage mcvm packages");
	cprintln!("<s>Usage:</s> mcvm package <k!><<subcommand>> [options]</k!>");
	cprintln!();
	cprintln!("<s>Subcommands:");
	cprintln!("{}<i,c>list:</i,c> {}", HYPHEN_POINT, LIST_HELP);
}

fn list(data: &mut CmdData) -> Result<(), CmdError> {
	data.ensure_config()?;

	if let Some(config) = &data.config {
		cprintln!("<s>Packages:");
		for (id, package) in config.packages.iter() {
			cprint!("{}", HYPHEN_POINT);
			match package.kind {
				PkgKind::Local(..) => cprint!("<m!>{}", id),
				PkgKind::Remote(..) => cprint!("<g!>{}", id)
			}
			for (prof_id, profile) in config.profiles.iter() {
				if profile.packages.contains(id) {
					cprint!(" <k!>({})", prof_id);
				}
			}
			cprintln!();
		}
	}
	Ok(())
}

pub fn run(argc: usize, argv: &[String], data: &mut CmdData)
-> Result<(), CmdError> {
	if argc == 0 {
		help();
		return Ok(());
	}

	match argv[0].as_str() {
		"list" => list(data)?,
		"help" => help(),
		cmd => cprintln!("<r>Unknown subcommand {}", cmd)
	}

	Ok(())
}
