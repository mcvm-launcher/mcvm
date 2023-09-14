use anyhow::anyhow;
use clap::Subcommand;
use color_print::cprintln;
use mcvm::{io::snapshot::SnapshotKind, util::print::HYPHEN_POINT};

use super::CmdData;

#[derive(Debug, Subcommand)]
pub enum SnapshotSubcommand {
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

pub async fn run(subcommand: SnapshotSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match subcommand {
		SnapshotSubcommand::List { raw, instance } => list(data, raw, &instance).await,
		SnapshotSubcommand::Create { instance, snapshot } => {
			create(data, &instance, &snapshot).await
		}
		SnapshotSubcommand::Remove { instance, snapshot } => {
			remove(data, &instance, &snapshot).await
		}
		SnapshotSubcommand::Restore { instance, snapshot } => {
			restore(data, &instance, &snapshot).await
		}
		SnapshotSubcommand::Info { instance, snapshot } => info(data, &instance, &snapshot).await,
	}
}

async fn list(data: &mut CmdData, raw: bool, instance: &str) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let instance = config
		.instances
		.get(instance)
		.ok_or(anyhow!("Instance does not exist"))?;
	let (snapshot_dir, index) = instance.open_snapshot_index(&data.paths)?;

	for snapshot in &index.snapshots {
		if raw {
			println!("{}", snapshot.id);
		} else {
			cprintln!("{}{}", HYPHEN_POINT, snapshot.id);
		}
	}

	index.finish(&snapshot_dir)?;
	Ok(())
}

async fn create(data: &mut CmdData, instance: &str, snapshot: &str) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let instance = config
		.instances
		.get(instance)
		.ok_or(anyhow!("Instance does not exist"))?;

	let (snapshot_dir, index) = instance.open_snapshot_index(&data.paths)?;
	if index.snapshot_exists(snapshot) {
		cprintln!("<y>Warning: Overwriting existing snapshot with same ID");
	}
	index.finish(&snapshot_dir)?;

	instance.create_snapshot(snapshot.to_string(), SnapshotKind::User, &data.paths)?;

	cprintln!("<g>Snapshot created.");

	Ok(())
}

async fn remove(data: &mut CmdData, instance: &str, snapshot: &str) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let instance = config
		.instances
		.get(instance)
		.ok_or(anyhow!("Instance does not exist"))?;
	instance.remove_snapshot(snapshot, &data.paths)?;

	cprintln!("<g>Snapshot removed.");

	Ok(())
}

async fn restore(data: &mut CmdData, instance: &str, snapshot: &str) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let instance = config
		.instances
		.get(instance)
		.ok_or(anyhow!("Instance does not exist"))?;
	instance.restore_snapshot(snapshot, &data.paths).await?;

	cprintln!("<g>Snapshot restored.");

	Ok(())
}

async fn info(data: &mut CmdData, instance_id: &str, snapshot_id: &str) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let instance = config
		.instances
		.get(instance_id)
		.ok_or(anyhow!("Instance does not exist"))?;
	let (snapshot_dir, index) = instance.open_snapshot_index(&data.paths)?;

	let snapshot = index
		.snapshots
		.iter()
		.find(|x| x.id == snapshot_id)
		.ok_or(anyhow!("Snapshot does not exist"))?;

	cprintln!(
		"<s>Snapshot <b>{}</b> in instance <g>{}</g>:",
		snapshot_id,
		instance_id
	);
	cprintln!("{} Date created: <c>{}", HYPHEN_POINT, snapshot.date);

	index.finish(&snapshot_dir)?;
	Ok(())
}
