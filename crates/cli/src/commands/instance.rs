use std::time::Duration;

use anyhow::{anyhow, Context};
use clap::Subcommand;
use color_print::cprintln;
use inquire::Select;
use itertools::Itertools;
use mcvm::data::config::Config;
use mcvm::shared::id::{InstanceRef, ProfileID};

use mcvm::data::instance::launch::LaunchSettings;
use mcvm::shared::modifications::Proxy;
use mcvm::shared::Side;
use reqwest::Client;

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
}

pub async fn run(command: InstanceSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match command {
		InstanceSubcommand::List { raw, side, profile } => list(data, raw, side, profile).await,
		InstanceSubcommand::Launch {
			user,
			offline,
			instance,
		} => launch(instance, user, offline, data).await,
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

	let profile = config
		.profiles
		.get_mut(&instance_ref.profile)
		.context("Profile in instance reference does not exist")?;
	let instance = profile
		.instances
		.get(&instance_ref.instance)
		.context("Instance in profile does not exist")?;
	let side = instance.get_side();

	if let Some(user) = user {
		config
			.users
			.choose_user(&user)
			.context("Failed to choose user")?;
	}

	// Launch the proxy first
	let proxy_handle = if side == Side::Server && profile.modifications.proxy != Proxy::None {
		let client = Client::new();
		profile
			.launch_proxy(&client, &data.paths, &config.plugins, &mut data.output)
			.await
			.context("Failed to launch profile proxy")?
	} else {
		None
	};

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

	// Await both asynchronously if the proxy is present
	if let Some(mut proxy_handle) = proxy_handle {
		let proxy = async move {
			proxy_handle
				.wait()
				.context("Failed to wait for proxy child process")?;

			Ok::<(), anyhow::Error>(())
		};

		let instance = async move {
			// Wait for the proxy to start up
			tokio::time::sleep(Duration::from_secs(5)).await;
			instance_handle
				.wait(&mut data.output)
				.context("Failed to wait for instance child process")?;

			Ok::<(), anyhow::Error>(())
		};

		tokio::try_join!(proxy, instance).context("Failed to launch proxy and instance")?;
	} else {
		// Otherwise, just wait for the instance
		instance_handle
			.wait(&mut data.output)
			.context("Failed to wait for instance child process")?;
	}

	Ok(())
}

/// Pick which instance to launch
fn pick_instance(instance: Option<String>, config: &Config) -> anyhow::Result<InstanceRef> {
	if let Some(instance) = instance {
		InstanceRef::parse(instance).context("Failed to parse instance reference")
	} else {
		let options = config.get_all_instances().sorted().collect();
		let selection = Select::new("Choose an instance to launch", options)
			.prompt()
			.context("Prompt failed")?;

		Ok(selection)
	}
}
