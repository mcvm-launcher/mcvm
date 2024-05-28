use anyhow::{anyhow, Context};
use clap::Subcommand;
use color_print::cprintln;
use inquire::Select;
use itertools::Itertools;
use mcvm::data::config::Config;
use mcvm::data::profile::update::update_profiles;
use mcvm::io::lock::Lockfile;
use mcvm::shared::id::{InstanceRef, ProfileID};

use mcvm::data::instance::launch::LaunchSettings;
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
		/// Whether to launch in offline mode, skipping authentication. This only works
		/// if you have authenticated at least once
		#[arg(short, long)]
		offline: bool,
		/// The instance to launch, as an instance reference (profile:instance)
		instance: Option<String>,
	},
	#[command(about = "Print the directory of an instance")]
	Dir {
		/// The instance to print the directory of
		instance: Option<String>,
	},
}

pub async fn run(command: InstanceSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match command {
		InstanceSubcommand::List { raw, side, profile } => list(data, raw, side, profile).await,
		InstanceSubcommand::Launch {
			user,
			offline,
			instance,
		} => launch(instance, user, offline, data).await,
		InstanceSubcommand::Dir { instance } => dir(data, instance).await,
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

	for id in config.get_all_instances().sorted() {
		let instance = config
			.get_instance(&id)
			.expect("Instance in instance list does not exist in config");
		if let Some(side) = side {
			if instance.get_side() != side {
				continue;
			}
		}

		if let Some(profile) = profile {
			if !profile.instances.contains_key(&id.instance) {
				continue;
			}
		}

		if raw {
			println!("{id}");
		} else {
			match instance.get_side() {
				Side::Client => cprintln!("{}<y!>{}", HYPHEN_POINT, id),
				Side::Server => cprintln!("{}<c!>{}", HYPHEN_POINT, id),
			}
		}
	}

	Ok(())
}

pub async fn launch(
	instance: Option<String>,
	user: Option<String>,
	offline: bool,
	data: &mut CmdData,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let instance_ref = pick_instance(instance, config).context("Failed to pick instance")?;

	// Perform first update if needed
	let mut lock = Lockfile::open(&data.paths).context("Failed to open lockfile")?;
	if !lock.has_instance_done_first_update(&instance_ref.to_string()) {
		cprintln!("<s>Performing first update of instance profile...");

		update_profiles(
			&data.paths,
			config,
			&[instance_ref.get_profile_id()],
			false,
			true,
			&mut data.output,
		)
		.await
		.context("Failed to update instance profile")?;

		// Since the update was successful, we can mark the instance as ready
		lock.update_instance_has_done_first_update(&instance_ref.to_string());
		lock.finish(&data.paths)
			.context("Failed to finish using lockfile")?;
	}

	let profile = config
		.profiles
		.get_mut(&instance_ref.get_profile_id())
		.context("Profile in instance reference does not exist")?;

	if let Some(user) = user {
		config
			.users
			.choose_user(&user)
			.context("Failed to choose user")?;
	}

	let instance = profile
		.instances
		.get_mut(&instance_ref.instance)
		.context("Instance in profile does not exist")?;

	let launch_settings = LaunchSettings {
		ms_client_id: get_ms_client_id(),
		offline_auth: offline,
	};
	let instance_handle = instance
		.launch(
			&data.paths,
			&mut config.users,
			&config.plugins,
			&profile.version,
			launch_settings,
			&mut data.output,
		)
		.await
		.context("Instance failed to launch")?;

	instance_handle
		.wait(&config.plugins, &data.paths, &mut data.output)
		.context("Failed to wait for instance child process")?;

	Ok(())
}

async fn dir(data: &mut CmdData, instance: Option<String>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;

	let instance = pick_instance(instance, data.config.get()).context("Failed to pick instance")?;
	let instance = data
		.config
		.get_mut()
		.get_instance_mut(&instance)
		.context("Instance does not exist")?;
	instance.ensure_dirs(&data.paths)?;

	println!("{}", &instance.get_dirs().get().game_dir.to_string_lossy());

	Ok(())
}

/// Pick which instance to use
pub fn pick_instance(instance: Option<String>, config: &Config) -> anyhow::Result<InstanceRef> {
	if let Some(instance) = instance {
		InstanceRef::parse(instance).context("Failed to parse instance reference")
	} else {
		let options = config.get_all_instances().sorted().collect();
		let selection = Select::new("Choose an instance", options)
			.prompt()
			.context("Prompt failed")?;

		Ok(selection)
	}
}
