use anyhow::Context;
use clap::Subcommand;
use color_print::{cprint, cprintln};
use inquire::Select;
use itertools::Itertools;
use mcvm::data::config::Config;
use mcvm::data::instance::update::InstanceUpdateContext;
use mcvm::io::lock::Lockfile;
use mcvm::shared::id::InstanceID;

use mcvm::data::instance::launch::LaunchSettings;
use mcvm::shared::Side;
use reqwest::Client;

use super::CmdData;
use crate::output::{icons_enabled, HYPHEN_POINT, INSTANCE, LOADER, PACKAGE, VERSION};
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
	#[command(about = "Print useful information about an instance")]
	Info { instance: String },
	Update {
		/// Whether to force update files that have already been downloaded
		#[arg(short, long)]
		force: bool,
		/// Whether to update all instances
		#[arg(short, long)]
		all: bool,
		/// Whether to skip updating packages
		#[arg(short = 'P', long)]
		skip_packages: bool,
		/// The instance to update
		instance: String,
	},
	#[command(about = "Print the directory of an instance")]
	Dir {
		/// The instance to print the directory of
		instance: Option<String>,
	},
}

pub async fn run(command: InstanceSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match command {
		InstanceSubcommand::List { raw, side } => list(data, raw, side).await,
		InstanceSubcommand::Launch {
			user,
			offline,
			instance,
		} => launch(instance, user, offline, data).await,
		InstanceSubcommand::Info { instance } => info(data, &instance).await,
		InstanceSubcommand::Update {
			force,
			all,
			skip_packages,
			instance,
		} => update(data, instance, all, force, skip_packages).await,
		InstanceSubcommand::Dir { instance } => dir(data, instance).await,
	}
}

async fn list(data: &mut CmdData, raw: bool, side: Option<Side>) -> anyhow::Result<()> {
	data.ensure_config(!raw).await?;
	let config = data.config.get_mut();

	for (id, instance) in config.instances.iter().sorted_by_key(|x| x.0) {
		if let Some(side) = side {
			if instance.get_side() != side {
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

async fn info(data: &mut CmdData, id: &str) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	fn print_indent() {
		print!("   ");
	}

	let instance = config
		.instances
		.get(id)
		.with_context(|| format!("Unknown instance '{id}'"))?;

	if icons_enabled() {
		print!("{} ", INSTANCE);
	}
	cprintln!("<s><g>Instance <b>{}", id);
	print_indent();
	if icons_enabled() {
		print!("{} ", VERSION);
	}
	cprintln!("<s>Version:</s> <g>{}", instance.get_config().version);

	print_indent();
	cprint!("{}Type: ", HYPHEN_POINT);
	match instance.get_side() {
		Side::Client => cprint!("<y!>Client"),
		Side::Server => cprint!("<c!>Server"),
	}
	cprintln!();

	if instance.get_config().modifications.common_modloader() {
		print_indent();
		if icons_enabled() {
			print!("{} ", LOADER);
		}
		cprintln!(
			"<s>Modloader:</s> <g>{}",
			instance
				.get_config()
				.modifications
				.get_modloader(Side::Client)
		);
	} else {
		print_indent();
		if icons_enabled() {
			print!("{} ", LOADER);
		}
		cprintln!(
			"<s>Client:</s> <g>{}",
			instance.get_config().modifications.client_type
		);
		print_indent();
		if icons_enabled() {
			print!("{} ", LOADER);
		}
		cprintln!(
			"<s>Server:</s> <g>{}",
			instance.get_config().modifications.server_type
		);
	}

	print_indent();
	if icons_enabled() {
		print!("{} ", PACKAGE);
	}
	cprintln!("<s>Packages:");
	for pkg in instance.get_configured_packages() {
		print_indent();
		cprint!("{}", HYPHEN_POINT);
		cprint!("<b!>{}<g!>", pkg.id);
		cprintln!();
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

	let instance_id = pick_instance(instance, config).context("Failed to pick instance")?;

	let instance = config
		.instances
		.get_mut(&instance_id)
		.context("Instance does not exist")?;

	// Perform first update if needed
	let mut lock = Lockfile::open(&data.paths).context("Failed to open lockfile")?;
	if !lock.has_instance_done_first_update(&instance_id) {
		cprintln!("<s>Performing first update of instance profile...");

		let client = Client::new();
		let mut ctx = InstanceUpdateContext {
			packages: &mut config.packages,
			users: &config.users,
			plugins: &config.plugins,
			prefs: &config.prefs,
			paths: &data.paths,
			lock: &mut lock,
			client: &client,
			output: &mut data.output,
		};

		instance
			.update(true, false, &mut ctx)
			.await
			.context("Failed to perform first update for instance")?;

		// Since the update was successful, we can mark the instance as ready
		lock.update_instance_has_done_first_update(&instance_id);
		lock.finish(&data.paths)
			.context("Failed to finish using lockfile")?;
	}

	if let Some(user) = user {
		config
			.users
			.choose_user(&user)
			.context("Failed to choose user")?;
	}

	let launch_settings = LaunchSettings {
		ms_client_id: get_ms_client_id(),
		offline_auth: offline,
	};
	let instance_handle = instance
		.launch(
			&data.paths,
			&mut config.users,
			&config.plugins,
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
		.instances
		.get_mut(&instance)
		.context("Instance does not exist")?;
	instance.ensure_dirs(&data.paths)?;

	println!("{}", &instance.get_dirs().get().game_dir.to_string_lossy());

	Ok(())
}

async fn update(
	data: &mut CmdData,
	instance: String,
	all: bool,
	force: bool,
	skip_packages: bool,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let ids: Vec<InstanceID> = if all {
		config.instances.keys().cloned().collect()
	} else {
		vec![InstanceID::from(instance)]
	};

	let client = Client::new();
	let mut lock = Lockfile::open(&data.paths).context("Failed to open lockfile")?;
	for id in ids {
		let instance = config
			.instances
			.get_mut(&id)
			.with_context(|| format!("Unknown instance '{id}'"))?;

		let mut ctx = InstanceUpdateContext {
			packages: &mut config.packages,
			users: &config.users,
			plugins: &config.plugins,
			prefs: &config.prefs,
			paths: &data.paths,
			lock: &mut lock,
			client: &client,
			output: &mut data.output,
		};

		instance
			.update(!skip_packages, force, &mut ctx)
			.await
			.context("Failed to update instance")?;
	}

	Ok(())
}

/// Pick which instance to use
pub fn pick_instance(instance: Option<String>, config: &Config) -> anyhow::Result<InstanceID> {
	if let Some(instance) = instance {
		Ok(instance.into())
	} else {
		let options = config.instances.keys().sorted().collect();
		let selection = Select::new("Choose an instance", options)
			.prompt()
			.context("Prompt failed")?;

		Ok(selection.to_owned())
	}
}
