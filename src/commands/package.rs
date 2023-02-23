use super::lib::{CmdData, CmdError};
use crate::util::print::{HYPHEN_POINT, ReplPrinter};

use color_print::{cprintln, cprint, cformat};

static LIST_HELP: &str = "List all installed packages";
static SYNC_HELP: &str = "Update all package indexes";

pub fn help() {
	cprintln!("<i>package:</i> Manage mcvm packages");
	cprintln!("<s>Usage:</s> mcvm package <k!><<subcommand>> [options]</k!>");
	cprintln!();
	cprintln!("<s>Subcommands:");
	cprintln!("{}<i,c>list:</i,c> {}", HYPHEN_POINT, LIST_HELP);
	cprintln!("{}<i,c>sync:</i,c> {}", HYPHEN_POINT, SYNC_HELP);
}

fn list(data: &mut CmdData) -> Result<(), CmdError> {
	data.ensure_config()?;

	if let Some(config) = &data.config {
		cprintln!("<s>Packages:");
		// for (id, package) in config.packages.iter() {
		// 	cprint!("{}", HYPHEN_POINT);
		// 	match package.kind {
		// 		PkgKind::Local(..) => cprint!("<m!>{}", package.full_name()),
		// 		PkgKind::Remote(..) => cprint!("<g!>{}", package.full_name())
		// 	}
		// 	for (prof_id, profile) in config.profiles.iter() {
		// 		if profile.packages.contains(id) {
		// 			cprint!(" <k!>({})", prof_id);
		// 		}
		// 	}
		// 	cprintln!();
		// }
	}
	Ok(())
}

fn sync(data: &mut CmdData) -> Result<(), CmdError> {
	data.ensure_config()?;
	data.ensure_paths()?;

	if let Some(config) = &mut data.config {
		if let Some(paths) = &data.paths {
			let mut printer = ReplPrinter::new(true);
			for repo in config.package_repos.iter_mut() {
				printer.print(&cformat!("Syncing repository <b>{}</b>...", repo.id));
				repo.sync(paths)?;
				printer.print(&cformat!("<g>Synced repository <b!>{}</b!>", repo.id));
				cprintln!();
			}
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
		"sync" => sync(data)?,
		cmd => cprintln!("<r>Unknown subcommand {}", cmd)
	}

	Ok(())
}
