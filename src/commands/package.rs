use std::collections::HashMap;

use super::lib::{CmdData, CmdError};
use crate::package::reg::PkgRequest;
use crate::util::print::{HYPHEN_POINT, ReplPrinter};

use color_print::{cprintln, cformat};

static LIST_HELP: &str = "List all installed packages";
static SYNC_HELP: &str = "Update all package indexes";
static CAT_HELP: &str = "Print the contents of a package";

pub fn help() {
	cprintln!("<i>package:</i> Manage mcvm packages");
	cprintln!("<s>Usage:</s> mcvm package <k!><<subcommand>> [options]</k!>");
	cprintln!();
	cprintln!("<s>Subcommands:");
	cprintln!("{}<i,c>list, ls:</i,c> {}", HYPHEN_POINT, LIST_HELP);
	cprintln!("{}<i,c>sync:</i,c> {}", HYPHEN_POINT, SYNC_HELP);
	cprintln!("{}<i,c>cat:</i,c> {}", HYPHEN_POINT, CAT_HELP);
}

fn list(data: &mut CmdData) -> Result<(), CmdError> {
	data.ensure_paths()?;
	data.ensure_config()?;

	if let Some(config) = &mut data.config {
		if let Some(paths) = &data.paths {
			let mut found_pkgs: HashMap<String, (String, Vec<String>)> = HashMap::new();
			for (id, profile) in config.profiles.iter() {
				if !profile.packages.is_empty() {
					for pkg in profile.packages.iter() {
						let version = config.packages.get_version(&pkg.req, paths)?;
						found_pkgs.entry(pkg.req.name.clone())
							.or_insert((version, vec![])).1.push(id.clone());
					}
				}
			}
			cprintln!("<s>Packages:");
			for (pkg, (version, profiles)) in found_pkgs {
				cprintln!("<b!>{}:{}", pkg, version);
				for profile in profiles {
					cprintln!("{}<k!>{}", HYPHEN_POINT, profile);
				}
			}
		}
	}
	Ok(())
}

fn sync(data: &mut CmdData) -> Result<(), CmdError> {
	data.ensure_config()?;
	data.ensure_paths()?;

	if let Some(config) = &mut data.config {
		if let Some(paths) = &data.paths {
			let mut printer = ReplPrinter::new(true);
			for repo in config.packages.repos.iter_mut() {
				printer.print(&cformat!("Syncing repository <b>{}</b>...", repo.id));
				match repo.sync(paths) {
					Ok(..) => {}
					Err(e) => {
						printer.print(&cformat!("<r>{}", e));
						continue;
					}
				};
				printer.print(&cformat!("<g>Synced repository <b!>{}</b!>", repo.id));
				cprintln!();
			}
			printer.finish();
			cprintln!("<s>Removing cached packages...");
			for (_, profile) in config.profiles.iter() {
				for pkg in profile.packages.iter() {
					config.packages.remove_cached(&pkg.req, paths)?;
				}
			}
		}
	}
	
	Ok(())
}

async fn cat(data: &mut CmdData, name: &str) -> Result<(), CmdError> {
	data.ensure_config()?;
	data.ensure_paths()?;

	if let Some(config) = &mut data.config {
		if let Some(paths) = &data.paths {
			let req = PkgRequest::new(name);
			let contents = config.packages.load(&req, false, paths)?;
			cprintln!("<s,b>Contents of package <g>{}</g>:</s,b>", req);
			cprintln!("{}", contents);
		}
	}

	Ok(())
}

pub async fn run(argc: usize, argv: &[String], data: &mut CmdData)
-> Result<(), CmdError> {
	if argc == 0 {
		help();
		return Ok(());
	}

	match argv[0].as_str() {
		"list" | "ls" => list(data)?,
		"sync" => sync(data)?,
		"cat" => match argc {
			2 => cat(data, &argv[1]).await?,
			_ => cprintln!("{}", CAT_HELP)
		}
		cmd => cprintln!("<r>Unknown subcommand {}", cmd)
	}

	Ok(())
}
