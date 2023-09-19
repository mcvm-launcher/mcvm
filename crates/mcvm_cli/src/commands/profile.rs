use super::CmdData;
use itertools::Itertools;
use mcvm::data::id::ProfileID;
use mcvm::data::instance::InstKind;
use mcvm::data::profile::update::update_profiles;

use anyhow::bail;
use clap::Subcommand;
use color_print::{cprint, cprintln};
use mcvm::shared::Side;

use crate::output::HYPHEN_POINT;

#[derive(Debug, Subcommand)]
pub enum ProfileSubcommand {
	#[command(about = "Print useful information about a profile")]
	Info { profile: String },
	#[command(about = "List all profiles")]
	#[clap(alias = "ls")]
	List {
		/// Whether to remove formatting and warnings from the output
		#[arg(short, long)]
		raw: bool,
	},
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
		/// Whether to skip updating packages
		#[arg(short = 'P', long)]
		skip_packages: bool,
		/// The profiles to update
		profiles: Vec<String>,
	},
}

pub async fn run(subcommand: ProfileSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match subcommand {
		ProfileSubcommand::Info { profile } => info(data, &profile).await,
		ProfileSubcommand::List { raw } => list(data, raw).await,
		ProfileSubcommand::Update {
			force,
			all,
			profiles,
			skip_packages,
		} => update(data, &profiles, force, all, skip_packages).await,
	}
}

async fn info(data: &mut CmdData, id: &str) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	if let Some(profile) = config.profiles.get(id) {
		cprintln!("<s><g>Profile <b>{}", id);
		cprintln!("   <s>Version:</s> <g>{}", profile.version);

		if profile.modifications.common_modloader() {
			cprintln!(
				"   <s>Modloader:</s> <g>{}",
				profile.modifications.get_modloader(Side::Client)
			);
		} else {
			cprintln!("   <s>Client:</s> <g>{}", profile.modifications.client_type);
			cprintln!("   <s>Server:</s> <g>{}", profile.modifications.server_type);
		}

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
		for pkg in profile.packages.iter_global() {
			cprint!("   {}", HYPHEN_POINT);
			cprint!("<b!>{}<g!>", pkg);
			cprintln!();
		}
	} else {
		bail!("Unknown profile '{id}'");
	}

	Ok(())
}

async fn list(data: &mut CmdData, raw: bool) -> anyhow::Result<()> {
	data.ensure_config(!raw).await?;
	let config = data.config.get();

	if !raw {
		cprintln!("<s>Profiles:");
	}
	for (id, profile) in config.profiles.iter().sorted_by_key(|x| x.0) {
		if raw {
			println!("{id}");
		} else {
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
	}

	Ok(())
}

async fn update(
	data: &mut CmdData,
	ids: &[String],
	force: bool,
	all: bool,
	skip_packages: bool,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let ids: Vec<ProfileID> = if all {
		config.profiles.keys().cloned().collect()
	} else {
		ids.iter().cloned().map(ProfileID::from).collect()
	};

	update_profiles(
		&data.paths,
		config,
		&ids,
		force,
		!skip_packages,
		&mut data.output,
	)
	.await?;

	Ok(())
}
