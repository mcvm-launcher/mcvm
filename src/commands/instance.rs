use anyhow::{bail, Context};
use clap::Subcommand;
use color_print::cprintln;

use crate::{data::instance::InstKind, util::print::HYPHEN_POINT};

use super::CmdData;

#[derive(Debug, Subcommand)]
pub enum InstanceSubcommand {
	#[command(about = "List all instances in all profiles")]
	#[clap(alias = "ls")]
	List {
		/// Whether to remove formatting from the output
		#[arg(short, long)]
		raw: bool,
	},
	#[command(about = "Launch instances to play the game")]
	Launch {
		/// Whether to print the command that was generated when launching
		#[arg(short, long)]
		debug: bool,
		/// An optional Minecraft session token to override with
		#[arg(long)]
		token: Option<String>,
		/// The instance to launch
		instance: String,
	},
}

async fn list(data: &mut CmdData, raw: bool) -> anyhow::Result<()> {
	data.ensure_config().await?;
	let config = data.config.get_mut();
	for (id, instance) in config.instances.iter() {
		if raw {
			println!("{id}");
		} else {
			match instance.kind {
				InstKind::Client { .. } => cprintln!("{}<y!>{}", HYPHEN_POINT, id),
				InstKind::Server { .. } => cprintln!("{}<c!>{}", HYPHEN_POINT, id),
			}
		}
	}

	Ok(())
}

pub async fn launch(
	instance: &str,
	debug: bool,
	token: Option<String>,
	data: &mut CmdData,
) -> anyhow::Result<()> {
	data.ensure_paths().await?;
	data.ensure_config().await?;
	let paths = data.paths.get();
	let config = data.config.get_mut();

	if let Some(instance) = config.instances.get_mut(instance) {
		let (.., profile) = config
			.profiles
			.iter()
			.find(|(.., profile)| profile.instances.contains(&instance.id))
			.expect("Instance does not belong to any profiles");
		instance
			.launch(paths, &config.auth, debug, token, &profile.version)
			.await
			.context("Instance failed to launch")?;
	} else {
		bail!("Unknown instance '{}'", instance);
	}

	Ok(())
}

pub async fn run(command: InstanceSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match command {
		InstanceSubcommand::List { raw } => list(data, raw).await,
		InstanceSubcommand::Launch {
			debug,
			token,
			instance,
		} => launch(&instance, debug, token, data).await,
	}
}
