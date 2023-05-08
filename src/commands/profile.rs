use super::CmdData;
use crate::data::instance::InstKind;
use crate::data::profile::update::update_profiles;
use crate::util::print::HYPHEN_POINT;

use anyhow::bail;
use anyhow::Context;
use clap::Subcommand;
use color_print::{cprint, cprintln};

#[derive(Debug, Subcommand)]
pub enum ProfileSubcommand {
	#[command(about = "Print useful information about a profile")]
	Info { profile: String },
	#[command(about = "List all profiles")]
	#[clap(alias = "ls")]
	List,
	#[command(
		about = "Update a profile",
		long_about = "Update the game files, extensions, packages, and addons of a profile."
	)]
	Update {
		/// Whether to force update files that have already been downloaded
		#[arg(short, long)]
		force: bool,
		/// Whether to update all profiles
		#[arg(short, long)]
		all: bool,
		/// The profiles to update
		profiles: Vec<String>,
	},
}

async fn info(data: &mut CmdData, id: &str) -> anyhow::Result<()> {
	data.ensure_paths().await?;
	data.ensure_config().await?;
	let paths = data.paths.get();
	let config = data.config.get_mut();

	if let Some(profile) = config.profiles.get(id) {
		cprintln!("<s><g>Profile <b>{}", id);
		cprintln!("   <s>Version:</s> <g>{}", profile.version);
		cprintln!("   <s>Modloader:</s> <g>{}", profile.modloader);
		cprintln!("   <s>Plugin Loader:</s> <g>{}", profile.plugin_loader);
		cprintln!("   <s>Instances:");
		for inst_id in profile.instances.iter() {
			if let Some(instance) = config.instances.get(inst_id) {
				cprint!("   {}", HYPHEN_POINT);
				match instance.kind {
					InstKind::Client { .. } => cprint!("<y!>Client {}", inst_id),
					InstKind::Server { .. } => cprint!("<c!>Server {}", inst_id),
				}
				cprintln!();
			}
		}
		cprintln!("   <s>Packages:");
		for pkg in profile.packages.iter() {
			let pkg_version = config
				.packages
				.get_version(&pkg.req, paths)
				.await
				.context("Failed to get package version")?;
			cprint!("   {}", HYPHEN_POINT);
			cprint!("<b!>{}:<g!>{}", pkg.req.name, pkg_version);
			cprintln!();
		}
	} else {
		bail!("Unknown profile '{id}'");
	}

	Ok(())
}

async fn list(data: &mut CmdData) -> anyhow::Result<()> {
	data.ensure_config().await?;
	let config = data.config.get();

	cprintln!("<s>Profiles:");
	for (id, profile) in config.profiles.iter() {
		cprintln!("<s><g>   {}", id);
		for inst_id in profile.instances.iter() {
			if let Some(instance) = config.instances.get(inst_id) {
				match instance.kind {
					InstKind::Client { .. } => cprintln!("   {}<y!>{}", HYPHEN_POINT, inst_id),
					InstKind::Server { .. } => cprintln!("   {}<c!>{}", HYPHEN_POINT, inst_id),
				}
			}
		}
	}

	Ok(())
}

async fn profile_update(
	data: &mut CmdData,
	ids: &[String],
	force: bool,
	all: bool,
) -> anyhow::Result<()> {
	data.ensure_paths().await?;
	data.ensure_config().await?;
	let paths = data.paths.get();
	let config = data.config.get_mut();

	let ids = if all {
		config.profiles.keys().cloned().collect::<Vec<String>>()
	} else {
		ids.to_vec()
	};

	update_profiles(paths, config, &ids, force).await?;

	Ok(())
}

pub async fn run(subcommand: ProfileSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match subcommand {
		ProfileSubcommand::Info { profile } => info(data, &profile).await,
		ProfileSubcommand::List => list(data).await,
		ProfileSubcommand::Update {
			force,
			all,
			profiles,
		} => profile_update(data, &profiles, force, all).await,
	}
}
