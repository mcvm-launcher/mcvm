mod snapshot;

use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};

use anyhow::Context;
use clap::Parser;
use color_print::cprintln;
use mcvm_plugin::{
	api::{CustomPlugin, HookContext},
	hooks,
};
use mcvm_shared::id::InstanceRef;
use snapshot::{Config, Index};

use crate::snapshot::SnapshotKind;

fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::new("snapshot")?;
	plugin.subcommand(|ctx, args| {
		let Some(subcommand) = args.first() else {
			return Ok(());
		};
		if subcommand != "snapshot" && subcommand != "snap" {
			return Ok(());
		}
		// Trick the parser to give it the right bin name
		let it = std::iter::once(format!("mcvm {subcommand}")).chain(args.into_iter().skip(1));
		let cli = Cli::parse_from(it);
		let result = match cli.command {
			Subcommand::List { raw, instance } => list(&ctx, raw, &instance),
			Subcommand::Create { instance, snapshot } => create(&ctx, &instance, &snapshot),
			Subcommand::Remove { instance, snapshot } => remove(&ctx, &instance, &snapshot),
			Subcommand::Restore { instance, snapshot } => restore(&ctx, &instance, &snapshot),
			Subcommand::Info { instance, snapshot } => info(&ctx, &instance, &snapshot),
		};
		result?;

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
#[command(name = "mcvm snapshot")]
enum Subcommand {
	#[command(about = "List snapshots for an instance")]
	#[clap(alias = "ls")]
	List {
		/// Whether to remove formatting and warnings from the output
		#[arg(short, long)]
		raw: bool,
		/// The instance to list snapshots for
		instance: String,
	},
	#[command(about = "Create a snapshot")]
	Create {
		/// The instance to create a snapshot for
		instance: String,
		/// The id of the snapshot
		snapshot: String,
	},
	#[command(about = "Remove an existing snapshot")]
	Remove {
		/// The instance the snapshot is in
		instance: String,
		/// The snapshot to remove
		snapshot: String,
	},
	#[command(about = "Restore an existing snapshot")]
	Restore {
		/// The instance the snapshot is in
		instance: String,
		/// The snapshot to restore
		snapshot: String,
	},
	#[command(about = "Print information about a specific snapshot")]
	Info {
		/// The instance the snapshot is in
		instance: String,
		/// The snapshot to get info about
		snapshot: String,
	},
}

fn list(ctx: &HookContext<'_, hooks::Subcommand>, raw: bool, instance: &str) -> anyhow::Result<()> {
	let snapshots_dir = get_snapshots_dir(ctx)?;
	let instance_snap_dir = get_instance_snapshot_dir(instance, &snapshots_dir)?;
	let index = Index::open(&instance_snap_dir)?;

	for snapshot in &index.snapshots {
		if raw {
			println!("{}", snapshot.id);
		} else {
			cprintln!("<k!> - </>{}", snapshot.id);
		}
	}

	index.finish(&instance_snap_dir)?;
	Ok(())
}

fn create(
	ctx: &HookContext<'_, hooks::Subcommand>,
	instance: &str,
	snapshot: &str,
) -> anyhow::Result<()> {
	let inst_ref =
		InstanceRef::parse(instance.into()).context("Failed to parse instance reference")?;

	let snapshots_dir = get_snapshots_dir(ctx)?;
	let instance_snap_dir = get_instance_snapshot_dir(&inst_ref.instance, &snapshots_dir)?;
	let mut index = Index::open(&instance_snap_dir)?;

	if index.snapshot_exists(snapshot) {
		cprintln!("<y>Warning: Overwriting existing snapshot with same ID");
	}

	let config = get_snapshot_config(&inst_ref, ctx)
		.context("Failed to get snapshot config for instance")?;
	let inst_dir = ctx
		.get_data_dir()?
		.join("instances")
		.join(inst_ref.profile.to_string())
		.join(&inst_ref.instance.to_string());

	index.create_snapshot(
		SnapshotKind::User,
		snapshot.into(),
		&config,
		&inst_ref.instance,
		&inst_dir,
		&instance_snap_dir,
	)?;

	index.finish(&instance_snap_dir)?;

	cprintln!("<g>Snapshot created.");

	Ok(())
}

fn remove(
	ctx: &HookContext<'_, hooks::Subcommand>,
	instance: &str,
	snapshot: &str,
) -> anyhow::Result<()> {
	let inst_ref =
		InstanceRef::parse(instance.into()).context("Failed to parse instance reference")?;

	let snapshots_dir = get_snapshots_dir(ctx)?;
	let instance_snap_dir = get_instance_snapshot_dir(&inst_ref.instance, &snapshots_dir)?;
	let mut index = Index::open(&instance_snap_dir)?;

	index.remove_snapshot(snapshot, &inst_ref.instance, &instance_snap_dir)?;
	index.finish(&instance_snap_dir)?;

	cprintln!("<g>Snapshot removed.");

	Ok(())
}

fn restore(
	ctx: &HookContext<'_, hooks::Subcommand>,
	instance: &str,
	snapshot: &str,
) -> anyhow::Result<()> {
	let inst_ref =
		InstanceRef::parse(instance.into()).context("Failed to parse instance reference")?;

	let snapshots_dir = get_snapshots_dir(ctx)?;
	let instance_snap_dir = get_instance_snapshot_dir(&inst_ref.instance, &snapshots_dir)?;
	let index = Index::open(&instance_snap_dir)?;

	let inst_dir = ctx
		.get_data_dir()?
		.join("instances")
		.join(inst_ref.profile.to_string())
		.join(&inst_ref.instance.to_string());

	index.restore_snapshot(snapshot, &inst_ref.instance, &inst_dir, &instance_snap_dir)?;
	index.finish(&instance_snap_dir)?;

	cprintln!("<g>Snapshot restored.");

	Ok(())
}

fn info(
	ctx: &HookContext<'_, hooks::Subcommand>,
	instance: &str,
	snapshot_id: &str,
) -> anyhow::Result<()> {
	let inst_ref =
		InstanceRef::parse(instance.into()).context("Failed to parse instance reference")?;

	let snapshots_dir = get_snapshots_dir(ctx)?;
	let instance_snap_dir = get_instance_snapshot_dir(&inst_ref.instance, &snapshots_dir)?;
	let index = Index::open(&instance_snap_dir)?;

	let snapshot = index
		.snapshots
		.iter()
		.find(|x| x.id == snapshot_id)
		.context("Snapshot does not exist")?;

	cprintln!(
		"<s>Snapshot <b>{}</b> in instance <g>{}</g>:",
		snapshot_id,
		inst_ref
	);
	cprintln!("<k!> - </>Date created: <c>{}", snapshot.date);

	Ok(())
}

fn get_snapshots_dir(ctx: &HookContext<'_, hooks::Subcommand>) -> anyhow::Result<PathBuf> {
	let dir = ctx.get_data_dir()?.join("snapshots");
	std::fs::create_dir_all(&dir)?;
	Ok(dir)
}

fn get_instance_snapshot_dir(instance: &str, snapshots_dir: &Path) -> anyhow::Result<PathBuf> {
	let dir = snapshots_dir.join(instance);
	std::fs::create_dir_all(&dir)?;
	Ok(dir)
}

fn get_snapshot_config(
	instance: &InstanceRef,
	ctx: &HookContext<'_, hooks::Subcommand>,
) -> anyhow::Result<Config> {
	let config = ctx.get_custom_config().unwrap_or("{}");
	let mut config: HashMap<String, Config> =
		serde_json::from_str(config).context("Failed to deserialize custom config")?;
	let config = config.remove(&instance.to_string()).unwrap_or_default();
	Ok(config)
}
