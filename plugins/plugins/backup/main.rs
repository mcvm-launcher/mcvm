mod backup;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Context;
use backup::{get_backup_directory, BackupAutoHook, Config, Index, DEFAULT_GROUP};
use clap::Parser;
use color_print::cprintln;
use mcvm_plugin::api::{CustomPlugin, HookContext};
use mcvm_plugin::api::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_plugin::hooks::{self, Hook};

use crate::backup::BackupSource;

fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::new("backup")?;
	plugin.subcommand(|ctx, args| {
		let Some(subcommand) = args.first() else {
			return Ok(());
		};
		if subcommand != "backup" && subcommand != "back" {
			return Ok(());
		}
		// Trick the parser to give it the right bin name
		let it = std::iter::once(format!("mcvm {subcommand}")).chain(args.into_iter().skip(1));
		let cli = Cli::parse_from(it);
		let result = match cli.command {
			Subcommand::List {
				raw,
				instance,
				group,
			} => list(&ctx, raw, &instance, group.as_deref()),
			Subcommand::Create { instance, group } => create(&ctx, &instance, group.as_deref()),
			Subcommand::Remove {
				instance,
				group,
				backup,
			} => remove(&ctx, &instance, group.as_deref(), &backup),
			Subcommand::Restore {
				instance,
				group,
				backup,
			} => restore(&ctx, &instance, group.as_deref(), &backup),
			Subcommand::Info {
				instance,
				group,
				backup,
			} => info(&ctx, &instance, group.as_deref(), &backup),
		};
		result?;

		Ok(())
	})?;

	plugin.on_instance_launch(|ctx, arg| {
		let inst_dir = PathBuf::from(&arg.dir);
		check_auto_hook(ctx, BackupAutoHook::OnLaunch, &arg.id, &inst_dir)?;

		Ok(())
	})?;

	plugin.on_instance_stop(|ctx, arg| {
		let inst_dir = PathBuf::from(&arg.dir);
		check_auto_hook(ctx, BackupAutoHook::OnStop, &arg.id, &inst_dir)?;

		Ok(())
	})?;

	Ok(())
}

#[derive(clap::Parser)]
struct Cli {
	#[command(subcommand)]
	command: Subcommand,
}

#[derive(clap::Subcommand)]
#[command(name = "mcvm backup")]
enum Subcommand {
	#[command(about = "List backups for an instance")]
	#[clap(alias = "ls")]
	List {
		/// Whether to remove formatting and warnings from the output
		#[arg(short, long)]
		raw: bool,
		/// The instance to list backups for
		instance: String,
		/// The group to list backups for
		group: Option<String>,
	},
	#[command(about = "Create a backup")]
	Create {
		/// The instance to create a backup for
		instance: String,
		/// The group to create the backup for
		group: Option<String>,
	},
	#[command(about = "Remove an existing backup")]
	Remove {
		/// The instance the backup is in
		instance: String,
		/// The group the backup is in
		group: Option<String>,
		/// The backup to remove
		backup: String,
	},
	#[command(about = "Restore an existing backup")]
	Restore {
		/// The instance the backup is in
		instance: String,
		/// The group the backup is in
		group: Option<String>,
		/// The backup to restore
		backup: String,
	},
	#[command(about = "Print information about a specific backup")]
	Info {
		/// The instance the backup is in
		instance: String,
		/// The group the backup is in
		group: Option<String>,
		/// The backup to get info about
		backup: String,
	},
}

fn list(
	ctx: &HookContext<'_, hooks::Subcommand>,
	raw: bool,
	instance: &str,
	group: Option<&str>,
) -> anyhow::Result<()> {
	let group = group.unwrap_or(DEFAULT_GROUP);

	let index = get_index(ctx, instance)?;
	let group = index
		.contents
		.groups
		.get(group)
		.context("Group does not exist")?;

	for backup in &group.backups {
		if raw {
			println!("{}", backup.id);
		} else {
			cprintln!("<k!> - </>{}", backup.id);
		}
	}

	index.finish()?;
	Ok(())
}

fn create(
	ctx: &HookContext<'_, hooks::Subcommand>,
	instance: &str,
	group: Option<&str>,
) -> anyhow::Result<()> {
	let group = group.unwrap_or(DEFAULT_GROUP);

	let mut index = get_index(ctx, instance)?;

	let inst_dir = ctx.get_data_dir()?.join("instances").join(instance);

	index.create_backup(BackupSource::User, Some(group), &inst_dir)?;

	index.finish()?;

	cprintln!("<g>Backup created.");

	Ok(())
}

fn remove(
	ctx: &HookContext<'_, hooks::Subcommand>,
	instance: &str,
	group: Option<&str>,
	backup: &str,
) -> anyhow::Result<()> {
	let group = group.unwrap_or(DEFAULT_GROUP);

	let mut index = get_index(ctx, instance)?;

	index.remove_backup(group, backup)?;
	index.finish()?;

	cprintln!("<g>Backup removed.");

	Ok(())
}

fn restore(
	ctx: &HookContext<'_, hooks::Subcommand>,
	instance: &str,
	group: Option<&str>,
	backup: &str,
) -> anyhow::Result<()> {
	let group = group.unwrap_or(DEFAULT_GROUP);

	let index = get_index(ctx, instance)?;

	let inst_dir = ctx.get_data_dir()?.join("instances").join(instance);

	index.restore_backup(group, backup, &inst_dir)?;
	index.finish()?;

	cprintln!("<g>Backup restored.");

	Ok(())
}

fn info(
	ctx: &HookContext<'_, hooks::Subcommand>,
	instance: &str,
	group: Option<&str>,
	backup_id: &str,
) -> anyhow::Result<()> {
	let group = group.unwrap_or(DEFAULT_GROUP);

	let index = get_index(ctx, instance)?;

	let backup = index.get_backup(group, backup_id)?;

	cprintln!(
		"<s>Backup <b>{}</b> in instance <g>{}</g>:",
		backup_id,
		instance
	);
	cprintln!("<k!> - </>Date created: <c>{}", backup.date);

	Ok(())
}

fn get_index<H: Hook>(ctx: &HookContext<'_, H>, inst_id: &str) -> anyhow::Result<Index> {
	let dir = get_backup_directory(&get_backups_dir(ctx)?, inst_id);
	Index::open(&dir, inst_id, &get_backup_config(inst_id, ctx)?)
}

fn get_backups_dir<H: Hook>(ctx: &HookContext<'_, H>) -> anyhow::Result<PathBuf> {
	let dir = ctx.get_data_dir()?.join("backups");
	std::fs::create_dir_all(&dir)?;
	Ok(dir)
}

fn get_backup_config<H: Hook>(instance: &str, ctx: &HookContext<'_, H>) -> anyhow::Result<Config> {
	let config = ctx.get_custom_config().unwrap_or("{}");
	let mut config: HashMap<String, Config> =
		serde_json::from_str(config).context("Failed to deserialize custom config")?;
	let config = config.remove(&instance.to_string()).unwrap_or_default();
	Ok(config)
}

fn check_auto_hook<H: Hook>(
	mut ctx: HookContext<'_, H>,
	hook: BackupAutoHook,
	instance: &str,
	inst_dir: &Path,
) -> anyhow::Result<()> {
	let mut index = get_index(&ctx, instance)?;
	let groups = index.config.groups.clone();

	let creating_backups = groups
		.values()
		.any(|x| matches!(x.on, Some(x) if x == hook));

	if creating_backups {
		ctx.get_output().start_process();
		ctx.get_output().display(
			MessageContents::StartProcess("Creating backups".into()),
			MessageLevel::Important,
		);
	}

	for (group_id, group) in groups {
		if let Some(on) = &group.on {
			#[allow(irrefutable_let_patterns)]
			if on == &hook {
				index.create_backup(BackupSource::Auto, Some(&group_id), inst_dir)?;
			}
		}
	}

	if creating_backups {
		ctx.get_output().display(
			MessageContents::Success("Backups created".into()),
			MessageLevel::Important,
		);
		ctx.get_output().end_process();
	}

	index.finish()?;

	Ok(())
}
