use std::collections::HashMap;

use super::CmdData;
use mcvm::package::reg::{PkgRequest, PkgRequestSource};
use mcvm::util::print::{ReplPrinter, HYPHEN_POINT};

use anyhow::{bail, Context};
use clap::Subcommand;
use color_print::{cformat, cprint, cprintln};

#[derive(Debug, Subcommand)]
pub enum PackageSubcommand {
	#[command(about = "List all installed packages across all profiles")]
	#[clap(alias = "ls")]
	List {
		/// Whether to remove formatting and warnings from the output
		#[arg(short, long)]
		raw: bool,
		/// A profile to filter packages from
		#[arg(short, long)]
		profile: Option<String>,
	},
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
	#[command(about = "Print information about a specific package")]
	Info {
		/// The package to get info about
		package: String,
	},
}

async fn list(data: &mut CmdData, raw: bool, profile: Option<String>) -> anyhow::Result<()> {
	data.ensure_paths().await?;
	data.ensure_config(!raw).await?;
	let paths = data.paths.get();
	let config = data.config.get_mut();

	if let Some(profile_id) = profile {
		if let Some(profile) = config.profiles.get(&profile_id) {
			if raw {
				for pkg in &profile.packages {
					println!("{}", pkg.req);
				}
			} else if profile.packages.is_empty() {
				cprintln!("<s>Profile <b>{}</b> has no packages installed", profile_id);
			} else {
				cprintln!("<s>Packages in profile <b>{}</b>:", profile_id);
				for pkg in &profile.packages {
					let version = config
						.packages
						.get_version(&pkg.req, paths)
						.await
						.context("Failed to get version of package")?;
					cprintln!("{}<b!>{}</>:<b!>{}</>", HYPHEN_POINT, pkg.req, version);
				}
			}
		} else {
			bail!("Unknown profile '{profile_id}'");
		}
	} else {
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
		if raw {
			for (pkg, ..) in found_pkgs {
				println!("{pkg}");
			}
		} else {
			cprintln!("<s>Packages:");
			for (pkg, (version, profiles)) in found_pkgs {
				cprintln!("<b!>{}</>:<b!>{}</>", pkg, version);
				for profile in profiles {
					cprintln!("{}<k!>{}", HYPHEN_POINT, profile);
				}
			}
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
	printer.print(&cformat!("<s>Updating packages..."));
	config
		.packages
		.update_cached_packages(paths)
		.await
		.context("Failed to update cached packages")?;

	Ok(())
}

async fn cat(data: &mut CmdData, name: &str, raw: bool) -> anyhow::Result<()> {
	data.ensure_config(!raw).await?;
	data.ensure_paths().await?;
	let paths = data.paths.get();
	let config = data.config.get_mut();

	let req = PkgRequest::new(name, PkgRequestSource::UserRequire);
	let contents = config.packages.load(&req, paths).await?;
	if !raw {
		cprintln!("<s,b>Contents of package <g>{}</g>:</s,b>", req);
	}
	cprint!("{}", contents);

	Ok(())
}

async fn info(data: &mut CmdData, id: &str) -> anyhow::Result<()> {
	data.ensure_paths().await?;
	data.ensure_config(true).await?;
	let paths = data.paths.get();
	let config = data.config.get_mut();

	let req = PkgRequest::new(id, PkgRequestSource::UserRequire);
	let package_version = config
		.packages
		.get_version(&req, paths)
		.await
		.context("Failed to get package version from registry")?;
	let metadata = config
		.packages
		.get_metadata(&req, paths)
		.await
		.context("Failed to get metadata from the registry")?;
	if let Some(name) = &metadata.name {
		cprintln!(
			"<s><g>Package</g> <b>{}</b> <y>v{}</y>",
			name,
			package_version
		);
	} else {
		cprintln!("<s><g>Package</g> <b>{}</b>:<y>{}</y>", id, package_version);
	}
	if let Some(description) = &metadata.description {
		if !description.is_empty() {
			cprintln!("   <k!>{}", description);
		}
	}
	cprintln!("   <s>ID:</s> <g>{}", id);
	if let Some(version) = &metadata.version {
		if !version.is_empty() {
			cprintln!("   <s>Version:</s> <g>{}", version);
		}
	}
	if let Some(authors) = &metadata.authors {
		if !authors.is_empty() {
			cprintln!("   <s>Authors:</s> <g>{}", authors.join(", "));
		}
	}
	if let Some(website) = &metadata.website {
		if !website.is_empty() {
			cprintln!("   <s>Website:</s> <b!>{}", website);
		}
	}
	if let Some(support) = &metadata.support {
		if !support.is_empty() {
			cprintln!("   <s>Support Link:</s> <b!>{}", support);
		}
	}

	Ok(())
}

pub async fn run(subcommand: PackageSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match subcommand {
		PackageSubcommand::List { raw, profile } => list(data, raw, profile).await,
		PackageSubcommand::Sync => sync(data).await,
		PackageSubcommand::Cat { raw, package } => cat(data, &package, raw).await,
		PackageSubcommand::Info { package } => info(data, &package).await,
	}
}
