use std::collections::HashMap;

use super::lib::{CmdData, CmdError};
use crate::data::{asset::Modloader, instance::InstKind};
use crate::package::reg::PkgRequest;
use crate::package::eval::eval::{EvalConstants, Routine};
use crate::util::print::{HYPHEN_POINT, ReplPrinter};
use crate::util::versions::{VersionPattern, MinecraftVersion};

use color_print::{cprintln, cformat};

static LIST_HELP: &str = "List all installed packages";
static SYNC_HELP: &str = "Update all package indexes";
static CAT_HELP: &str = "Print the contents of a package";

pub fn help() {
	cprintln!("<i>package:</i> Manage mcvm packages");
	cprintln!("<s>Usage:</s> mcvm package <k!><<subcommand>> [options]</k!>");
	cprintln!();
	cprintln!("<s>Subcommands:");
	cprintln!("{}<i,c>list:</i,c> {}", HYPHEN_POINT, LIST_HELP);
	cprintln!("{}<i,c>sync:</i,c> {}", HYPHEN_POINT, SYNC_HELP);
	cprintln!("{}<i,c>cat:</i,c> {}", HYPHEN_POINT, CAT_HELP);
}

fn list(data: &mut CmdData) -> Result<(), CmdError> {
	data.ensure_config()?;

	if let Some(config) = &data.config {
		let mut found_pkgs: HashMap<String, Vec<(String, String)>> = HashMap::new();
		for (id, profile) in config.profiles.iter() {
			if !profile.packages.is_empty() {
				for req in profile.packages.iter() {
					found_pkgs.entry(req.name.clone())
						.or_insert(vec![]).push((req.version.as_string().to_owned(), id.clone()));
				}
			}
		}
		cprintln!("<s>Packages:");
		for (pkg, versions) in found_pkgs {
			cprintln!("<g!>{}", pkg);
			for (version, profile) in versions {
				cprintln!("{}<b>{} <k!>{}", HYPHEN_POINT, version, profile);
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
				repo.sync(paths)?;
				printer.print(&cformat!("<g>Synced repository <b!>{}</b!>", repo.id));
				cprintln!();
			}
		}
	}
	
	Ok(())
}

async fn cat(data: &mut CmdData, name: &str, version: &str) -> Result<(), CmdError> {
	data.ensure_config()?;
	data.ensure_paths()?;

	if let Some(config) = &mut data.config {
		if let Some(paths) = &data.paths {
			let req = PkgRequest::new(name, &VersionPattern::from(version));
			let contents = config.packages.load(&req, paths)?;
			cprintln!("<s,b>Contents of package <g>{}</g>:</s,b>", req);
			cprintln!("{}", contents);
			config.packages.parse(&req, paths)?;
			let constants = EvalConstants {
				version: MinecraftVersion::Unknown(String::from("1.19.3")),
				modloader: Modloader::Fabric,
				side: InstKind::Client
			};
			config.packages.eval(&req, paths, Routine::Install, &constants).await?;
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
		"list" => list(data)?,
		"sync" => sync(data)?,
		"cat" => match argc {
			2 => cat(data, &argv[1], "latest").await?,
			3 => cat(data, &argv[1], &argv[2]).await?,
			_ => cprintln!("{}", CAT_HELP)
		}
		cmd => cprintln!("<r>Unknown subcommand {}", cmd)
	}

	Ok(())
}
