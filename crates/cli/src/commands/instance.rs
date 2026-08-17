use std::ops::DerefMut;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, bail};
use clap::Subcommand;
use color_print::{cprint, cprintln};
use inquire::Confirm;
use itertools::Itertools;
use nitrolaunch::config::modifications::{ConfigModification, apply_modifications_and_write};
use nitrolaunch::config_crate::instance::InstanceConfig;
use nitrolaunch::core::QuickPlayType;
use nitrolaunch::instance::Instance;
use nitrolaunch::instance::tracking::RunningInstanceRegistry;
use nitrolaunch::instance::transfer::load_formats;
use nitrolaunch::instance::update::manager::UpdateSettings;
use nitrolaunch::instance::update::{InstanceUpdateContext, UpdateFacets};
use nitrolaunch::io::lock::Lockfile;
use nitrolaunch::plugin_crate::try_read::LineDecoder;
use nitrolaunch::shared::id::InstanceID;
use nitrolaunch::shared::io::config::IO_CONFIG;
use nitrolaunch::shared::java_args::MemoryNum;
use nitrolaunch::shared::output::{MessageContents, NoOp};
use nitrolaunch::shared::util::to_string_json;

use nitrolaunch::instance::launch::LaunchSettings;
use nitrolaunch::shared::lang::translate::TranslationKey;
use nitrolaunch::shared::{Side, UpdateDepth, output::NitroOutput};
use reqwest::Client;

use super::CmdData;
use crate::commands::call_plugin_subcommand;
use crate::commands::config::edit_temp_file;
use crate::output::{
	HYPHEN_POINT, INSTANCE, LOADER, PACKAGE, VERSION, format_instance_stdout_line, icons_enabled,
	instance_stdout_line,
};
use crate::prompt::{
	pick_instance, pick_instance_id, pick_instances, pick_loader, pick_minecraft_version, pick_side,
};
use crate::secrets::get_ms_client_id;

#[derive(Debug, Subcommand)]
pub enum InstanceSubcommand {
	#[command(about = "List all instances")]
	#[clap(alias = "ls")]
	List {
		/// Whether to remove formatting and warnings from the output
		#[arg(short, long)]
		raw: bool,
		/// Filter by instance side
		#[arg(short, long)]
		side: Option<Side>,
	},
	#[command(about = "Print useful information about an instance")]
	Info { instance: Option<String> },
	#[command(about = "Launch instances to play the game")]
	Launch {
		/// An optional account to choose when launching
		#[arg(short, long)]
		account: Option<String>,
		/// Whether to launch in offline mode, skipping authentication. This only works
		/// if you have authenticated at least once
		#[arg(short, long)]
		offline: bool,
		/// Launch into a world or server. Can be either world:<world>, server:<ip> or realm:<realm>
		#[arg(short, long)]
		quick_play: Option<QuickPlayType>,
		/// The instance to launch
		instance: Option<String>,
	},
	#[command(about = "Update versions, files, and packages of an instance")]
	Update {
		/// Whether to force update files that have already been downloaded
		#[arg(short, long)]
		force: bool,
		/// Whether to update all instances
		#[arg(short, long)]
		all: bool,
		/// Whether to only update packages
		#[arg(short, long)]
		packages: bool,
		/// Whether to only update the modpack
		#[arg(short, long)]
		modpack: bool,
		/// Additional instance groups to update
		#[arg(short, long)]
		groups: Vec<String>,
		/// The instances to update
		instances: Vec<String>,
	},
	#[command(about = "Easily create a new instance")]
	Add {
		/// A plugin to create this instance with. Not all plugins support instances.
		#[arg(short, long)]
		plugin: Option<String>,
	},
	#[command(about = "Delete an instance and its files forever")]
	Delete {
		/// The instance to delete
		instance: Option<String>,
	},
	#[command(about = "Edit configuration for an instance")]
	Edit {
		/// The instance to edit
		instance: Option<String>,
	},
	#[command(about = "Import an instance from another launcher")]
	Import {
		/// The path to the instance
		path: String,
		/// The ID of the new instance
		instance: Option<String>,
		/// Which format to use
		#[arg(short, long)]
		format: Option<String>,
		/// The side of the instance. If not specified, auto-detects
		#[arg(short, long)]
		side: Option<Side>,
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
	#[command(about = "See which instances are running")]
	Status {},
	#[command(about = "Kill a running instance")]
	Kill {
		/// The ID of the instance to kill
		instance: Option<String>,
		/// The ID of the account that launched this instance
		#[arg(short, long)]
		account: Option<String>,
	},
	#[command(about = "View logs for an instance")]
	Logs {
		/// The instance to view the logs of
		instance: Option<String>,
	},
	#[command(about = "Duplicates an instance into a new one")]
	Duplicate {
		/// The instance to duplicate
		instance: Option<String>,
		/// The ID of the new instance
		new_id: Option<String>,
	},
	#[command(
		about = "Unlink an instance from its parent templates and combine into a single config"
	)]
	Consolidate {
		/// The instance to consolidate
		instance: Option<String>,
	},
	#[command(
		about = "Extract a template from an instance to let you share it with other instances"
	)]
	Extract {
		/// The instance to extract the template from
		instance: Option<String>,
		/// The ID of the new template
		new_id: Option<String>,
	},
	#[command(about = "Print the directory of an instance")]
	Dir {
		/// The instance to print the directory of
		instance: Option<String>,
	},
	#[clap(external_subcommand)]
	External(Vec<String>),
}

pub async fn run(command: InstanceSubcommand, mut data: CmdData<'_>) -> anyhow::Result<()> {
	match command {
		InstanceSubcommand::List { raw, side } => list(&mut data, raw, side).await,
		InstanceSubcommand::Launch {
			account,
			offline,
			quick_play,
			instance,
		} => launch(instance, account, offline, quick_play, data).await,
		InstanceSubcommand::Info { instance } => info(&mut data, instance).await,
		InstanceSubcommand::Update {
			force,
			all,
			packages,
			modpack,
			groups,
			instances,
		} => update(&mut data, instances, groups, all, force, packages, modpack).await,
		InstanceSubcommand::Dir { instance } => dir(&mut data, instance).await,
		InstanceSubcommand::Add { plugin } => add(&mut data, plugin).await,
		InstanceSubcommand::Import {
			instance,
			path,
			format,
			side,
		} => import(&mut data, instance, path, format, side).await,
		InstanceSubcommand::Export {
			instance,
			format,
			output,
		} => export(&mut data, instance, format, output).await,
		InstanceSubcommand::Delete { instance } => delete(&mut data, instance).await,
		InstanceSubcommand::Edit { instance } => edit(&mut data, instance).await,
		InstanceSubcommand::Duplicate { instance, new_id } => {
			duplicate(&mut data, instance, new_id).await
		}
		InstanceSubcommand::Consolidate { instance } => consolidate(&mut data, instance).await,
		InstanceSubcommand::Extract { instance, new_id } => {
			extract(&mut data, instance, new_id).await
		}
		InstanceSubcommand::Logs { instance } => logs(&mut data, instance).await,
		InstanceSubcommand::Status {} => status(&mut data).await,
		InstanceSubcommand::Kill { instance, account } => kill(&mut data, instance, account).await,
		InstanceSubcommand::External(args) => {
			call_plugin_subcommand(args, Some("instance"), &mut data).await
		}
	}
}

async fn list(data: &mut CmdData<'_>, raw: bool, side: Option<Side>) -> anyhow::Result<()> {
	data.ensure_config(!raw).await?;
	let config = data.config.get_mut();

	for (id, instance) in config.instances.iter().sorted_by_key(|x| x.0) {
		if let Some(side) = side
			&& instance.side() != side
		{
			continue;
		}

		if raw {
			println!("{id}");
		} else {
			match instance.side() {
				Side::Client => cprintln!("{}<g>{}", HYPHEN_POINT, id),
				Side::Server => cprintln!("{}<b>{}", HYPHEN_POINT, id),
			}
		}
	}

	Ok(())
}

async fn info(data: &mut CmdData<'_>, id: Option<String>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let id = pick_instance(id, config)?;

	fn print_indent() {
		print!("   ");
	}

	let instance = config
		.instances
		.get(&id)
		.with_context(|| format!("Unknown instance '{id}'"))?;

	if icons_enabled() {
		print!("{} ", INSTANCE);
	}
	cprintln!("<s><g>Instance <b>{}", id);

	cprintln!("<s>Basic Info:");
	if !instance.original_config().from.is_empty() {
		print_indent();
		cprint!("<s>Derives from:</> ");
		cprint!("<b>{}", instance.original_config().from.iter().join(", "));
		cprintln!();
	}

	print_indent();
	if icons_enabled() {
		print!("{} ", VERSION);
	}
	cprintln!("<s>Version:</s> <g>{}", instance.version());

	print_indent();
	cprint!("<s>Type: ");
	match instance.side() {
		Side::Client => cprint!("<y!>Client"),
		Side::Server => cprint!("<c!>Server"),
	}
	cprintln!();

	print_indent();
	if icons_enabled() {
		print!("{} ", LOADER);
	}
	cprintln!("<s>Loader:</s> <g>{}", instance.loader());

	print_indent();
	if icons_enabled() {
		print!("{} ", PACKAGE);
	}
	cprintln!("<s>Packages:");
	for pkg in instance.packages().iter().sorted_by_key(|x| x.req.clone()) {
		print_indent();
		cprint!("{}", HYPHEN_POINT);
		cprint!("<b!>{}<g!>", pkg.req);
		cprintln!();
	}

	cprintln!("<s>Misc Info:");

	print_indent();
	let size = instance.get_size().await;
	match size {
		Ok(size) => {
			cprintln!("<s>Size on Disk: <g>{}", MemoryNum::from_bytes(size))
		}
		Err(e) => {
			cprintln!("<s,r>Failed to get disk size: {e}");
		}
	}

	Ok(())
}

pub async fn launch(
	instance: Option<String>,
	account: Option<String>,
	offline: bool,
	quick_play: Option<QuickPlayType>,
	mut data: CmdData<'_>,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let instance_id = pick_instance(instance, config).context("Failed to pick instance")?;

	let client = Client::new();
	let core = config
		.get_core(
			Some(&get_ms_client_id()),
			&UpdateSettings {
				depth: UpdateDepth::Shallow,
				offline_auth: offline,
			},
			&client,
			&config.plugins,
			&data.paths,
			data.output,
		)
		.await?;

	let instance = config
		.instances
		.get_mut(&instance_id)
		.context("Instance does not exist")?;

	if let Some(account) = account {
		config
			.accounts
			.choose_account(&account)
			.context("Failed to choose account")?;
	}

	let launch_settings = LaunchSettings {
		offline_auth: offline,
		pipe_stdin: true,
		quick_play,
	};

	let mut lock = Lockfile::open(&data.paths)?;

	let mut ctx = InstanceUpdateContext {
		packages: &mut config.packages,
		accounts: &mut config.accounts,
		plugins: &config.plugins,
		prefs: &config.prefs,
		paths: &data.paths,
		lock: &mut lock,
		client: &client,
		output: data.output,
		core: &core,
	};

	let mut instance_handle = instance
		.launch(launch_settings, &mut ctx)
		.await
		.context("Instance failed to launch")?;

	// Drop items early so that they aren't wasting memory while the instance is running
	let plugins = config.plugins.clone();
	std::mem::drop(data.config);
	lock.finish(&data.paths)?;
	std::mem::drop(lock);
	std::mem::drop(client);

	let format_output = IO_CONFIG.get_bool("format_instance_output").unwrap_or(true);

	if format_output {
		let mut decoder = LineDecoder::new();
		let output_fn = move |chunk: &[u8]| {
			if chunk.is_empty() {
				return;
			}

			let lines = decoder.decode(chunk, chunk.len());
			let Ok(Some(lines)) = lines else {
				return;
			};

			for line in lines {
				instance_stdout_line(&line);
			}
		};

		instance_handle.set_output_fn(Box::new(output_fn));
	}

	instance_handle
		.wait(&plugins, &data.paths, data.output)
		.await
		.context("Failed to wait for instance child process")?;

	Ok(())
}

async fn dir(data: &mut CmdData<'_>, instance: Option<String>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get();

	let instance = pick_instance(instance, config).context("Failed to pick instance")?;
	let instance = config
		.instances
		.get(&instance)
		.context("Instance does not exist")?;
	instance.ensure_dir()?;

	if let Some(dir) = instance.dir() {
		println!("{}", dir.to_string_lossy());
	} else {
		bail!("Instance has no directory");
	}

	Ok(())
}

async fn update(
	data: &mut CmdData<'_>,
	instances: Vec<String>,
	groups: Vec<String>,
	all: bool,
	force: bool,
	packages: bool,
	modpack: bool,
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

	if ids.is_empty() {
		ids = pick_instances(config)?;
	}

	let client = Client::new();
	let mut lock = Lockfile::open(&data.paths).context("Failed to open lockfile")?;
	let core = config
		.get_core(
			Some(&get_ms_client_id()),
			&UpdateSettings {
				depth: UpdateDepth::Full,
				offline_auth: false,
			},
			&client,
			&config.plugins,
			&data.paths,
			&mut NoOp,
		)
		.await?;

	for id in ids {
		let instance = config
			.instances
			.get_mut(&id)
			.with_context(|| format!("Unknown instance '{id}'"))?;

		let mut ctx = InstanceUpdateContext {
			packages: &config.packages,
			accounts: &mut config.accounts,
			plugins: &config.plugins,
			prefs: &config.prefs,
			paths: &data.paths,
			lock: &mut lock,
			client: &client,
			output: data.output,
			core: &core,
		};

		let depth = if force {
			UpdateDepth::Force
		} else {
			UpdateDepth::Full
		};

		let facets = UpdateFacets::from_flags(packages, modpack);

		instance
			.update(depth, facets, &mut ctx)
			.await
			.context("Failed to update instance")?;

		// Clear the package registry to prevent dependency chains in requests being carried over
		config.packages.clear();

		// Mark the instance as having completed its first update
		lock.update_instance_has_done_first_update(instance.id());
		lock.finish(&data.paths)
			.context("Failed to finish using lockfile")?;
	}

	Ok(())
}

async fn add(data: &mut CmdData<'_>, plugin: Option<String>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get();
	let mut raw_config = data.get_raw_config()?;

	// Build the instance
	let id = pick_instance_id()?;

	let side = pick_side(None)?;

	let client = Client::new();
	let core = config
		.get_core(
			Some(&get_ms_client_id()),
			&UpdateSettings {
				depth: UpdateDepth::Full,
				offline_auth: false,
			},
			&client,
			&config.plugins,
			&data.paths,
			&mut NoOp,
		)
		.await?;
	let manifest = core
		.get_version_manifest(None, UpdateDepth::Shallow, &mut NoOp)
		.await?;

	let version = pick_minecraft_version(&manifest.list).await?;

	let loader = pick_loader(None, Some(side), &config.plugins, &data.paths).await?;

	let instance_config = InstanceConfig {
		side: Some(side),
		version: Some(version.to_serialized()),
		loader: Some(to_string_json(&loader)),
		source_plugin: plugin,
		..Default::default()
	};

	apply_modifications_and_write(
		&mut raw_config,
		vec![ConfigModification::AddInstance(id, instance_config)],
		&data.paths,
		&data.config.get().plugins,
		data.output,
	)
	.await
	.context("Failed to write modified config")?;

	data.output
		.display(MessageContents::Success("Instance added".into()));

	Ok(())
}

async fn import(
	data: &mut CmdData<'_>,
	instance: Option<String>,
	path: String,
	format: Option<String>,
	side: Option<Side>,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get();

	let instance = if let Some(instance) = instance {
		InstanceID::from(instance)
	} else {
		pick_instance_id()?
	};

	if config.instances.contains_key(&instance) {
		bail!("An instance with that ID already exists");
	}

	// Figure out the format
	let formats = load_formats(&config.plugins, &data.paths, data.output)
		.await
		.context("Failed to get available transfer formats")?;
	let format = if let Some(format) = &format {
		format
	} else {
		let options: Vec<_> = formats.iter_format_names().collect();
		if options.is_empty() {
			bail!(
				"{}",
				data.output.translate(TranslationKey::NoTransferFormats)
			);
		}
		inquire::Select::new("What format is the imported instance in?", options).prompt()?
	};

	let client = Client::new();
	let core = config
		.get_core(
			None,
			&UpdateSettings {
				depth: UpdateDepth::Shallow,
				offline_auth: false,
			},
			&client,
			&config.plugins,
			&data.paths,
			data.output,
		)
		.await?;
	let version_manifest = core
		.get_version_manifest(None, UpdateDepth::Shallow, data.output)
		.await
		.context("Failed to get Minecraft version manifest")?;

	let new_instance_config = Instance::import(
		&instance,
		format,
		&PathBuf::from(path),
		side,
		&formats,
		&version_manifest.list,
		&config.plugins,
		&data.paths,
		data.output,
	)
	.await
	.context("Failed to import the new instance")?;

	let mut config2 = data.get_raw_config()?;
	apply_modifications_and_write(
		&mut config2,
		vec![ConfigModification::AddInstance(
			instance,
			new_instance_config,
		)],
		&data.paths,
		&config.plugins,
		data.output,
	)
	.await
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
		.await
		.context("Failed to get available transfer formats")?;
	let format = if let Some(format) = &format {
		format
	} else {
		let options: Vec<_> = formats.iter_format_names().collect();
		if options.is_empty() {
			bail!(
				"{}",
				data.output.translate(TranslationKey::NoTransferFormats)
			);
		}
		inquire::Select::new("What format is the exported instance in?", options).prompt()?
	};

	let result_path = if let Some(output) = output {
		PathBuf::from(output)
	} else {
		let current_dir = std::env::current_dir()?;
		current_dir.join(format!("{instance}.zip"))
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
		.await
		.context("Failed to export instance")?;

	Ok(())
}

async fn delete(data: &mut CmdData<'_>, id: Option<String>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let id = pick_instance(id, config)?;

	let instance = config
		.instances
		.get_mut(&id)
		.with_context(|| format!("Unknown instance '{id}'"))?;

	let prompt = Confirm::new(
		"Are you SURE you want to delete this instance? This will remove world saves as well! (y/n)",
	);
	if !prompt.prompt()? {
		cprintln!("<r>Cancelled.");
		return Ok(());
	}

	let mut process = data.output.get_process();
	process.display(MessageContents::StartProcess("Deleting instance".into()));

	instance
		.delete(&data.paths, &config.plugins, process.deref_mut())
		.await
		.context("Failed to delete instance")?;

	process.display(MessageContents::Success("Instance deleted".into()));

	Ok(())
}

async fn edit(data: &mut CmdData<'_>, id: Option<String>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let mut raw_config = data.get_raw_config()?;
	let config = data.config.get_mut();

	let id = pick_instance(id, config)?;

	let instance = config
		.instances
		.get_mut(&id)
		.with_context(|| format!("Unknown instance '{id}'"))?;

	let mut inst_config = instance.original_config().clone();
	if inst_config.source_plugin.is_some() && !inst_config.is_editable {
		bail!("This plugin instance does not support editing");
	}
	inst_config.remove_plugin_only_fields();

	let text = serde_json::to_string_pretty(&inst_config).context("Failed to serialize config")?;
	let edited = edit_temp_file(&text, &format!("Editing instance {id}"), &data.paths)?;
	let mut new_config: InstanceConfig = serde_json::from_str(&edited)
		.context("Failed to serialize. Make sure your config is valid JSON")?;
	new_config.restore_plugin_only_fields(&inst_config);

	let modifications = vec![ConfigModification::UpdateInstance(id, new_config)];
	apply_modifications_and_write(
		&mut raw_config,
		modifications,
		&data.paths,
		&config.plugins,
		data.output,
	)
	.await
	.context("Failed to modify and write config")?;

	data.output
		.display(MessageContents::Success("Changes saved".into()));

	Ok(())
}

async fn logs(data: &mut CmdData<'_>, id: Option<String>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let id = pick_instance(id, config)?;

	let instance = config
		.instances
		.get_mut(&id)
		.with_context(|| format!("Unknown instance '{id}'"))?;

	let logs = instance
		.get_logs(&config.plugins, &data.paths, data.output)
		.await
		.context("Failed to get instance logs")?;

	if logs.is_empty() {
		cprintln!("No logs available");
		return Ok(());
	}

	loop {
		let select = inquire::Select::new("Browsing logs. Press Escape to exit.", logs.clone());
		let log = select.prompt_skippable()?;
		if let Some(log) = log {
			if let Ok(log_text) = instance
				.get_log(&log, &config.plugins, &data.paths, data.output)
				.await
			{
				let log_text = log_text.lines().map(format_instance_stdout_line).join("\n");
				cprintln!("<s>Log <g>{log}");
				println!("{log_text}");
			} else {
				cprintln!("<s,r>Failed to read log {log}");
			}
			inquire::Confirm::new("Press Escape to return to browse page").prompt_skippable()?;
		} else {
			break;
		}
	}

	Ok(())
}

async fn duplicate(
	data: &mut CmdData<'_>,
	instance: Option<String>,
	new_id: Option<String>,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get();

	let instance = pick_instance(instance, config)?;
	let instance = config
		.instances
		.get(&instance)
		.context("Instance does not exist")?;

	let new_id = if let Some(new_id) = new_id {
		new_id.into()
	} else {
		pick_instance_id()?
	};

	instance
		.duplicate(&new_id, &data.paths, &config.plugins, data.output)
		.await?;

	data.output
		.display(MessageContents::Success("Changes saved".into()));

	Ok(())
}

async fn consolidate(data: &mut CmdData<'_>, instance: Option<String>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get();

	let instance = pick_instance(instance, config)?;
	let instance = config
		.instances
		.get(&instance)
		.context("Instance does not exist")?;

	instance
		.consolidate(&data.paths, &config.plugins, data.output)
		.await?;

	data.output
		.display(MessageContents::Success("Changes saved".into()));

	Ok(())
}

async fn extract(
	data: &mut CmdData<'_>,
	instance: Option<String>,
	new_id: Option<String>,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get();

	let instance = pick_instance(instance, config)?;
	let instance = config
		.instances
		.get(&instance)
		.context("Instance does not exist")?;

	let new_id = if let Some(new_id) = new_id {
		new_id.into()
	} else {
		pick_instance_id()?
	};

	instance
		.extract(&new_id, &data.paths, &config.plugins, data.output)
		.await?;

	data.output
		.display(MessageContents::Success("Changes saved".into()));

	Ok(())
}

async fn status(data: &mut CmdData<'_>) -> anyhow::Result<()> {
	let registry = RunningInstanceRegistry::open(&data.paths).context("Failed to open registry")?;
	let count = registry.iter_entries().count();
	if count == 0 {
		println!("No instances running");
	} else {
		cprintln!("<s><b>{count}</> instances running:");

		for entry in registry.iter_entries() {
			cprint!("{HYPHEN_POINT}<s,g>{}", entry.instance_id);
			if let Some(account) = &entry.account {
				cprint!(" as <b>{account}");
			}
			cprintln!(" [<m>{}</>]", entry.pid);
		}
	}

	Ok(())
}

async fn kill(
	data: &mut CmdData<'_>,
	instance: Option<String>,
	account: Option<String>,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get();

	let instance = pick_instance(instance, config)?;

	let mut registry =
		RunningInstanceRegistry::open(&data.paths).context("Failed to open registry")?;

	registry.kill_instance(&instance, account.as_deref());

	registry.write()?;

	data.output
		.display(MessageContents::Success("Instance killed".into()));

	Ok(())
}
