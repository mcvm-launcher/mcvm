use anyhow::{anyhow, Context};
use clap::Subcommand;
use color_print::cprintln;
use inquire::Select;
use itertools::Itertools;
use mcvm::data::config::Config;
use mcvm::data::id::{InstanceRef, ProfileID};

use mcvm::data::instance::InstKind;
use mcvm::shared::Side;

use super::CmdData;
use crate::output::HYPHEN_POINT;
use crate::secrets::get_ms_client_id;

#[derive(Debug, Subcommand)]
pub enum InstanceSubcommand {
	#[command(about = "List all instances in all profiles")]
	#[clap(alias = "ls")]
	List {
		/// Whether to remove formatting and warnings from the output
		#[arg(short, long)]
		raw: bool,
		/// Filter by instance side
		#[arg(short, long)]
		side: Option<Side>,
		/// Filter by profile
		#[arg(short, long)]
		profile: Option<String>,
	},
	#[command(about = "Launch instances to play the game")]
	Launch {
		/// An optional user to choose when launching
		#[arg(short, long)]
		user: Option<String>,
		/// The instance to launch, as an instance reference (profile:instance)
		instance: Option<String>,
	},
}

pub async fn run(command: InstanceSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match command {
		InstanceSubcommand::List { raw, side, profile } => list(data, raw, side, profile).await,
		InstanceSubcommand::Launch { user, instance } => launch(instance, user, data).await,
	}
}

async fn list(
	data: &mut CmdData,
	raw: bool,
	side: Option<Side>,
	profile: Option<String>,
) -> anyhow::Result<()> {
	data.ensure_config(!raw).await?;
	let config = data.config.get_mut();

	let profile = if let Some(profile) = profile {
		let profile = ProfileID::from(profile);
		Some(
			config
				.profiles
				.get(&profile)
				.ok_or(anyhow!("Profile '{profile}' does not exist"))?,
		)
	} else {
		None
	};

	for (id, instance) in config.instances.iter().sorted_by_key(|x| x.0) {
		if let Some(side) = side {
			if instance.kind.to_side() != side {
				continue;
			}
		}

		if let Some(profile) = profile {
			if !profile.instances.contains(&id.instance) {
				continue;
			}
		}

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
	instance: Option<String>,
	user: Option<String>,
	data: &mut CmdData,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let instance = pick_instance(instance, config).context("Failed to pick instance")?;

	let instance = config
		.instances
		.get_mut(&instance)
		.ok_or(anyhow!("Unknown instance '{instance}'"))?;
	let (.., profile) = config
		.profiles
		.iter()
		.find(|(.., profile)| profile.instances.contains(&instance.id))
		.expect("Instance does not belong to any profiles");

	if let Some(user) = user {
		config
			.users
			.choose_user(&user)
			.context("Failed to choose user")?;
	}

	let mut handle = instance
		.launch(
			&data.paths,
			&mut config.users,
			&profile.version,
			get_ms_client_id(),
			&mut data.output,
		)
		.await
		.context("Instance failed to launch")?;

	handle.wait().context("Failed to wait for child process")?;

	Ok(())
}

/// Pick which instance to launch
fn pick_instance(instance: Option<String>, config: &Config) -> anyhow::Result<InstanceRef> {
	if let Some(instance) = instance {
		InstanceRef::parse(instance).context("Failed to parse instance reference")
	} else {
		let options: Vec<InstanceRef> = config.instances.keys().cloned().collect();
		let selection = Select::new("Choose an instance to launch", options)
			.prompt()
			.context("Prompt failed")?;

		Ok(selection)
	}
}
