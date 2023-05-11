use std::collections::HashMap;

use super::CmdData;
use crate::package::reg::{PkgRequest, PkgRequestSource};
use crate::util::print::{ReplPrinter, HYPHEN_POINT};

use anyhow::Context;
use clap::Subcommand;
use color_print::{cformat, cprint, cprintln};

#[derive(Debug, Subcommand)]
pub enum PackageSubcommand {
	#[command(about = "List all installed packages across all profiles")]
	#[clap(alias = "ls")]
	List,
	#[command(
		about = "Sync package indexes with ones from package repositories",
		long_about = "Sync all package indexes from remote repositories. They will be
cached locally, but all currently cached package scripts will be removed"
	)]
	Sync,
	#[command(
		about = "Print the contents of a package to standard out",
		long_about = "Print the contents of any package to standard out.
This package does not need to be installed, it just has to be in the index."
	)]
	Cat {
		/// Whether to remove formatting and warnings from the output
		#[arg(short, long)]
		raw: bool,
		/// The package to print
		package: String,
	},
}

async fn list(data: &mut CmdData) -> anyhow::Result<()> {
	data.ensure_paths().await?;
	data.ensure_config(true).await?;
	let paths = data.paths.get();
	let config = data.config.get_mut();

	let mut found_pkgs: HashMap<String, (u32, Vec<String>)> = HashMap::new();
	for (id, profile) in config.profiles.iter() {
		if !profile.packages.is_empty() {
			for pkg in profile.packages.iter() {
				let version = config
					.packages
					.get_version(&pkg.req, paths)
					.await
					.context("Failed to get version of package")?;
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

	Ok(())
}

async fn sync(data: &mut CmdData) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	data.ensure_paths().await?;
	let paths = data.paths.get();
	let config = data.config.get_mut();

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

	Ok(())
}

async fn cat(data: &mut CmdData, name: &str, raw: bool) -> anyhow::Result<()> {
	data.ensure_config(!raw).await?;
	data.ensure_paths().await?;
	let paths = data.paths.get();
	let config = data.config.get_mut();

	let req = PkgRequest::new(name, PkgRequestSource::UserRequire);
	let contents = config.packages.load(&req, false, paths).await?;
	if !raw {
		cprintln!("<s,b>Contents of package <g>{}</g>:</s,b>", req);
	}
	cprint!("{}", contents);

	Ok(())
}

pub async fn run(subcommand: PackageSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match subcommand {
		PackageSubcommand::List => list(data).await,
		PackageSubcommand::Sync => sync(data).await,
		PackageSubcommand::Cat { raw, package } => cat(data, &package, raw).await,
	}
}
