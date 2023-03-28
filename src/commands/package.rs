use std::collections::HashMap;

use super::CmdData;
use crate::package::reg::PkgRequest;
use crate::util::print::{ReplPrinter, HYPHEN_POINT};

use clap::Subcommand;
use color_print::{cformat, cprintln};

#[derive(Debug, Subcommand)]
pub enum PackageSubcommand {
	List,
	Sync,
	Cat { pkg: String },
}

async fn list(data: &mut CmdData) -> anyhow::Result<()> {
	data.ensure_paths()?;
	data.ensure_config()?;

	if let Some(config) = &mut data.config {
		if let Some(paths) = &data.paths {
			let mut found_pkgs: HashMap<String, (String, Vec<String>)> = HashMap::new();
			for (id, profile) in config.profiles.iter() {
				if !profile.packages.is_empty() {
					for pkg in profile.packages.iter() {
						let version = config.packages.get_version(&pkg.req, paths).await?;
						found_pkgs
							.entry(pkg.req.name.clone())
							.or_insert((version, vec![]))
							.1
							.push(id.clone());
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

async fn sync(data: &mut CmdData) -> anyhow::Result<()> {
	data.ensure_config()?;
	data.ensure_paths()?;

	if let Some(config) = &mut data.config {
		if let Some(paths) = &data.paths {
			let mut printer = ReplPrinter::new(true);
			for repo in config.packages.repos.iter_mut() {
				printer.print(&cformat!("Syncing repository <b>{}</b>...", repo.id));
				match repo.sync(paths).await {
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
					config.packages.remove_cached(&pkg.req, paths).await?;
				}
			}
		}
	}

	Ok(())
}

async fn cat(data: &mut CmdData, name: &str) -> anyhow::Result<()> {
	data.ensure_config()?;
	data.ensure_paths()?;

	if let Some(config) = &mut data.config {
		if let Some(paths) = &data.paths {
			let req = PkgRequest::new(name);
			let contents = config.packages.load(&req, false, paths).await?;
			cprintln!("<s,b>Contents of package <g>{}</g>:</s,b>", req);
			cprintln!("{}", contents);
		}
	}

	Ok(())
}

pub async fn run(subcommand: PackageSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match subcommand {
		PackageSubcommand::List => list(data).await,
		PackageSubcommand::Sync => sync(data).await,
		PackageSubcommand::Cat { pkg } => cat(data, &pkg).await,
	}
}
