use clap::Subcommand;
use color_print::cprintln;

use crate::{data::instance::InstKind, util::print::HYPHEN_POINT};

use super::CmdData;

#[derive(Debug, Subcommand)]
pub enum InstanceSubcommand {
	#[command(about = "List all instances in all profiles")]
	#[clap(alias = "ls")]
	List {
		/// Whether to remove formatting from the output
		#[arg(short, long)]
		raw: bool,
	},
}

async fn list(data: &mut CmdData, raw: bool) -> anyhow::Result<()> {
	data.ensure_config().await?;
	let config = data.config.get_mut();
	for (id, instance) in config.instances.iter() {
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

pub async fn run(command: InstanceSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match command {
		InstanceSubcommand::List { raw } => list(data, raw).await,
	}
}
