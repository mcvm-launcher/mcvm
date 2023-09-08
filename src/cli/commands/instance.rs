use anyhow::{anyhow, bail, Context};
use clap::Subcommand;
use color_print::cprintln;
use inquire::Select;
use itertools::Itertools;
use mcvm::data::config::Config;
use mcvm::data::id::{ProfileID, InstanceID};
use mcvm::data::user::AuthState;

use mcvm::io::lock::Lockfile;
use mcvm::{data::instance::InstKind, util::print::HYPHEN_POINT};
use mcvm_shared::instance::Side;
use mcvm_shared::output::MessageLevel;

use super::CmdData;
use crate::cli::get_ms_client_id;

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
		/// Whether to print the command that was generated when launching
		#[arg(short, long)]
		debug: bool,
		/// An optional user to choose when launching
		#[arg(short, long)]
		user: Option<String>,
		/// The instance to launch
		instance: Option<String>,
	},
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
			if !profile.instances.contains(id) {
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
	debug: bool,
	user: Option<String>,
	data: &mut CmdData,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	if debug {
		data.output.set_log_level(MessageLevel::Debug);
	}

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
		if !config.users.users.contains_key(&user) {
			bail!("User '{user}' does not exist");
		}
		config.users.state = AuthState::UserChosen(user);
	}

	let mut lock = Lockfile::open(&data.paths)?;

	instance
		.launch(
			&data.paths,
			&mut lock,
			&mut config.users,
			&profile.version,
			get_ms_client_id(),
			&mut data.output,
		)
		.await
		.context("Instance failed to launch")?;

	Ok(())
}

/// Pick which instance to launch
fn pick_instance(instance: Option<String>, config: &Config) -> anyhow::Result<InstanceID> {
	if let Some(instance) = instance {
		Ok(InstanceID::from(instance))
	} else {
		let options: Vec<InstanceID> = config.instances.keys().cloned().collect();
		let selection = Select::new("Choose an instance to launch", options)
			.prompt()
			.context("Prompt failed")?;

		Ok(selection)
	}
}

pub async fn run(command: InstanceSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match command {
		InstanceSubcommand::List { raw, side, profile } => list(data, raw, side, profile).await,
		InstanceSubcommand::Launch {
			debug,
			user,
			instance,
		} => launch(instance, debug, user, data).await,
	}
}
