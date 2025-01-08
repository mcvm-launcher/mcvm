use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{bail, Context};
use clap::Subcommand;
use color_print::{cprint, cprintln};
use inquire::Select;
use itertools::Itertools;
use mcvm::config::builder::InstanceBuilder;
use mcvm::config::modifications::{apply_modifications_and_write, ConfigModification};
use mcvm::config::Config;
use mcvm::core::util::versions::{MinecraftLatestVersion, MinecraftVersionDeser};
use mcvm::instance::transfer::load_formats;
use mcvm::instance::update::InstanceUpdateContext;
use mcvm::instance::Instance;
use mcvm::io::lock::Lockfile;
use mcvm::shared::id::InstanceID;

use mcvm::instance::launch::LaunchSettings;
use mcvm::shared::modifications::{ClientType, ServerType};
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
		/// Additional instance groups to update
		#[arg(short, long)]
		groups: Vec<String>,
		/// The instances to update
		instances: Vec<String>,
	},
	#[command(about = "Print the directory of an instance")]
	Dir {
		/// The instance to print the directory of
		instance: Option<String>,
	},
	#[command(about = "Easily create a new instance")]
	Add,
	#[command(about = "Import an instance from another launcher")]
	Import {
		/// The ID of the new instance
		instance: String,
		/// Which format to use
		#[arg(short, long)]
		format: Option<String>,
	},
	#[command(about = "Export an instance for use in another launcher")]
	Export {
		/// The ID of the instance to export
		instance: Option<String>,
		/// Which format to use
		#[arg(short, long)]
		format: Option<String>,
		/// Where to export the instance to. Defaults to ./<instance-id>.zip
		#[arg(short, long)]
		output: Option<String>,
	},
}

pub async fn run(command: InstanceSubcommand, mut data: CmdData<'_>) -> anyhow::Result<()> {
	match command {
		InstanceSubcommand::List { raw, side } => list(&mut data, raw, side).await,
		InstanceSubcommand::Launch {
			user,
			offline,
			instance,
		} => launch(instance, user, offline, data).await,
		InstanceSubcommand::Info { instance } => info(&mut data, &instance).await,
		InstanceSubcommand::Update {
			force,
			all,
			skip_packages,
			groups,
			instances,
		} => update(&mut data, instances, groups, all, force, skip_packages).await,
		InstanceSubcommand::Dir { instance } => dir(&mut data, instance).await,
		InstanceSubcommand::Add => add(&mut data).await,
		InstanceSubcommand::Import { instance, format } => {
			import(&mut data, instance, format).await
		}
		InstanceSubcommand::Export {
			instance,
			format,
			output,
		} => export(&mut data, instance, format, output).await,
	}
}

async fn list(data: &mut CmdData<'_>, raw: bool, side: Option<Side>) -> anyhow::Result<()> {
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

async fn info(data: &mut CmdData<'_>, id: &str) -> anyhow::Result<()> {
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
	mut data: CmdData<'_>,
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
			output: data.output,
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
			data.output,
		)
		.await
		.context("Instance failed to launch")?;

	// Drop the config early so that it isn't wasting memory while the instance is running
	let plugins = config.plugins.clone();
	std::mem::drop(data.config);
	// Unload plugins that we don't need anymore

	instance_handle
		.wait(&plugins, &data.paths, data.output)
		.context("Failed to wait for instance child process")?;

	Ok(())
}

async fn dir(data: &mut CmdData<'_>, instance: Option<String>) -> anyhow::Result<()> {
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
	data: &mut CmdData<'_>,
	instances: Vec<String>,
	groups: Vec<String>,
	all: bool,
	force: bool,
	skip_packages: bool,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let mut ids: Vec<InstanceID> = if all {
		config.instances.keys().cloned().collect()
	} else {
		instances.into_iter().map(InstanceID::from).collect()
	};

	for group in groups {
		let group = Arc::from(group);
		let group = config
			.instance_groups
			.get(&group)
			.with_context(|| format!("Instance group '{group}' does not exist"))?;
		ids.extend(group.clone());
	}

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
			output: data.output,
		};

		instance
			.update(!skip_packages, force, &mut ctx)
			.await
			.context("Failed to update instance")?;

		// Clear the package registry to prevent dependency chains in requests being carried over
		config.packages.clear();

		// Mark the instance as having completed its first update
		lock.update_instance_has_done_first_update(instance.get_id());
		lock.finish(&data.paths)
			.context("Failed to finish using lockfile")?;
	}

	Ok(())
}

async fn add(data: &mut CmdData<'_>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let mut config = data.get_raw_config()?;

	// Build the profile
	let id = inquire::Text::new("What is the ID for the instance?").prompt()?;
	let id = InstanceID::from(id);
	let version = inquire::Text::new("What Minecraft version should the instance be?").prompt()?;
	let version = match version.as_str() {
		"latest" => MinecraftVersionDeser::Latest(MinecraftLatestVersion::Release),
		"latest_snapshot" => MinecraftVersionDeser::Latest(MinecraftLatestVersion::Snapshot),
		other => MinecraftVersionDeser::Version(other.into()),
	};
	let side_options = vec![Side::Client, Side::Server];
	let side =
		inquire::Select::new("What side should the instance be on?", side_options).prompt()?;

	let mut instance = InstanceBuilder::new(id.clone(), side);
	instance.version(version);

	match side {
		Side::Client => {
			let options = vec![
				ClientType::None,
				ClientType::Vanilla,
				ClientType::Fabric,
				ClientType::Quilt,
			];
			let client_type =
				inquire::Select::new("What client type should the instance use?", options)
					.prompt()?;
			instance.client_type(client_type);
		}
		Side::Server => {
			let options = vec![
				ServerType::None,
				ServerType::Vanilla,
				ServerType::Fabric,
				ServerType::Quilt,
				ServerType::Paper,
				ServerType::Sponge,
				ServerType::Folia,
			];
			let server_type =
				inquire::Select::new("What server type should the instance use?", options)
					.prompt()?;
			instance.server_type(server_type);
		}
	}

	let instance_config = instance.build_config();

	apply_modifications_and_write(
		&mut config,
		vec![ConfigModification::AddInstance(id, instance_config)],
		&data.paths,
	)
	.context("Failed to write modified config")?;

	cprintln!("<g>Instance added.");

	Ok(())
}

async fn import(
	data: &mut CmdData<'_>,
	instance: String,
	format: Option<String>,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get();

	let instance = InstanceID::from(instance);

	if config.instances.contains_key(&instance) {
		bail!("An instance with that ID already exists");
	}

	// Figure out the format
	let formats = load_formats(&config.plugins, &data.paths, data.output)
		.context("Failed to get available transfer formats")?;
	let format = if let Some(format) = &format {
		format
	} else {
		let options = formats.iter_format_names().collect();
		inquire::Select::new("What format is the imported instance in?", options).prompt()?
	};

	let new_instance_config = Instance::import(
		&instance,
		format,
		&formats,
		&config.plugins,
		&data.paths,
		data.output,
	)
	.context("Failed to import the new instance")?;

	let mut config = data.get_raw_config()?;
	apply_modifications_and_write(
		&mut config,
		vec![ConfigModification::AddInstance(
			instance,
			new_instance_config,
		)],
		&data.paths,
	)
	.context("Failed to write modified config")?;

	Ok(())
}

async fn export(
	data: &mut CmdData<'_>,
	instance: Option<String>,
	format: Option<String>,
	output: Option<String>,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let instance = pick_instance(instance, config)?;

	// Figure out the format
	let formats = load_formats(&config.plugins, &data.paths, data.output)
		.context("Failed to get available transfer formats")?;
	let format = if let Some(format) = &format {
		format
	} else {
		let options = formats.iter_format_names().collect();
		inquire::Select::new("What format is the imported instance in?", options).prompt()?
	};

	let result_path = if let Some(output) = output {
		PathBuf::from(output)
	} else {
		PathBuf::from(format!("./{instance}.zip"))
	};

	let instance = config
		.instances
		.get_mut(&instance)
		.context("The provided instance does not exist")?;

	let lock = Lockfile::open(&data.paths).context("Failed to open Lockfile")?;

	instance
		.export(
			format,
			&result_path,
			&formats,
			&config.plugins,
			&lock,
			&data.paths,
			data.output,
		)
		.context("Failed to export instance")?;

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
