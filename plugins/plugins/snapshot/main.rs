mod snapshot;

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use color_print::cprintln;
use mcvm_plugin::api::{CustomPlugin, HookContext};
use mcvm_plugin::hooks;
use mcvm_shared::id::InstanceRef;
use snapshot::{get_snapshot_directory, Config, Index, DEFAULT_GROUP};

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
			Subcommand::List {
				raw,
				instance,
				group,
			} => list(&ctx, raw, &instance, group.as_deref()),
			Subcommand::Create { instance, group } => create(&ctx, &instance, group.as_deref()),
			Subcommand::Remove {
				instance,
				group,
				snapshot,
			} => remove(&ctx, &instance, group.as_deref(), &snapshot),
			Subcommand::Restore {
				instance,
				group,
				snapshot,
			} => restore(&ctx, &instance, group.as_deref(), &snapshot),
			Subcommand::Info {
				instance,
				group,
				snapshot,
			} => info(&ctx, &instance, group.as_deref(), &snapshot),
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
		/// The group to list snapshots for
		group: Option<String>,
	},
	#[command(about = "Create a snapshot")]
	Create {
		/// The instance to create a snapshot for
		instance: String,
		/// The group to create the snapshot for
		group: Option<String>,
	},
	#[command(about = "Remove an existing snapshot")]
	Remove {
		/// The instance the snapshot is in
		instance: String,
		/// The group the snapshot is in
		group: Option<String>,
		/// The snapshot to remove
		snapshot: String,
	},
	#[command(about = "Restore an existing snapshot")]
	Restore {
		/// The instance the snapshot is in
		instance: String,
		/// The group the snapshot is in
		group: Option<String>,
		/// The snapshot to restore
		snapshot: String,
	},
	#[command(about = "Print information about a specific snapshot")]
	Info {
		/// The instance the snapshot is in
		instance: String,
		/// The group the snapshot is in
		group: Option<String>,
		/// The snapshot to get info about
		snapshot: String,
	},
}

fn list(
	ctx: &HookContext<'_, hooks::Subcommand>,
	raw: bool,
	instance: &str,
	group: Option<&str>,
) -> anyhow::Result<()> {
	let inst_ref =
		InstanceRef::parse(instance.into()).context("Failed to parse instance reference")?;
	let group = group.unwrap_or(DEFAULT_GROUP);

	let index = get_index(ctx, &inst_ref)?;
	let group = index
		.contents
		.groups
		.get(group)
		.context("Group does not exist")?;

	for snapshot in &group.snapshots {
		if raw {
			println!("{}", snapshot.id);
		} else {
			cprintln!("<k!> - </>{}", snapshot.id);
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
	let inst_ref =
		InstanceRef::parse(instance.into()).context("Failed to parse instance reference")?;
	let group = group.unwrap_or(DEFAULT_GROUP);

	let mut index = get_index(ctx, &inst_ref)?;

	let inst_dir = ctx
		.get_data_dir()?
		.join("instances")
		.join(inst_ref.profile.to_string())
		.join(&inst_ref.instance.to_string());

	index.create_snapshot(SnapshotKind::User, Some(group), &inst_dir)?;

	index.finish()?;

	cprintln!("<g>Snapshot created.");

	Ok(())
}

fn remove(
	ctx: &HookContext<'_, hooks::Subcommand>,
	instance: &str,
	group: Option<&str>,
	snapshot: &str,
) -> anyhow::Result<()> {
	let inst_ref =
		InstanceRef::parse(instance.into()).context("Failed to parse instance reference")?;
	let group = group.unwrap_or(DEFAULT_GROUP);

	let mut index = get_index(ctx, &inst_ref)?;

	index.remove_snapshot(group, snapshot)?;
	index.finish()?;

	cprintln!("<g>Snapshot removed.");

	Ok(())
}

fn restore(
	ctx: &HookContext<'_, hooks::Subcommand>,
	instance: &str,
	group: Option<&str>,
	snapshot: &str,
) -> anyhow::Result<()> {
	let inst_ref =
		InstanceRef::parse(instance.into()).context("Failed to parse instance reference")?;
	let group = group.unwrap_or(DEFAULT_GROUP);

	let index = get_index(ctx, &inst_ref)?;

	let inst_dir = ctx
		.get_data_dir()?
		.join("instances")
		.join(inst_ref.profile.to_string())
		.join(&inst_ref.instance.to_string());

	index.restore_snapshot(group, snapshot, &inst_dir)?;
	index.finish()?;

	cprintln!("<g>Snapshot restored.");

	Ok(())
}

fn info(
	ctx: &HookContext<'_, hooks::Subcommand>,
	instance: &str,
	group: Option<&str>,
	snapshot_id: &str,
) -> anyhow::Result<()> {
	let inst_ref =
		InstanceRef::parse(instance.into()).context("Failed to parse instance reference")?;
	let group = group.unwrap_or(DEFAULT_GROUP);

	let index = get_index(ctx, &inst_ref)?;

	let snapshot = index.get_snapshot(group, snapshot_id)?;

	cprintln!(
		"<s>Snapshot <b>{}</b> in instance <g>{}</g>:",
		snapshot_id,
		inst_ref
	);
	cprintln!("<k!> - </>Date created: <c>{}", snapshot.date);

	Ok(())
}

fn get_index(
	ctx: &HookContext<'_, hooks::Subcommand>,
	inst_ref: &InstanceRef,
) -> anyhow::Result<Index> {
	let dir = get_snapshot_directory(&get_snapshots_dir(ctx)?, inst_ref);
	Index::open(&dir, inst_ref.clone(), &get_snapshot_config(inst_ref, ctx)?)
}

fn get_snapshots_dir(ctx: &HookContext<'_, hooks::Subcommand>) -> anyhow::Result<PathBuf> {
	let dir = ctx.get_data_dir()?.join("snapshots");
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
