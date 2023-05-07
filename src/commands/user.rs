use super::CmdData;
use crate::data::user::UserKind;
use crate::util::print::HYPHEN_POINT;

use clap::Subcommand;
use color_print::{cprint, cprintln};

#[derive(Debug, Subcommand)]
pub enum UserSubcommand {
	#[command(about = "List all users")]
	#[clap(alias = "ls")]
	List,
	#[command(about = "Get current authentication status")]
	Status,
}

async fn list(data: &mut CmdData) -> anyhow::Result<()> {
	data.ensure_config().await?;
	let config = data.config.get();

	cprintln!("<s>Users:");
	for (id, user) in config.auth.users.iter() {
		cprint!("{}", HYPHEN_POINT);
		match user.kind {
			UserKind::Microsoft => cprintln!("<s><g>{}</g> <k!>({})</k!>", user.name, id),
			UserKind::Demo => cprintln!("<s><c!>{}</c!> <k!>({})</k!>", user.name, id),
			UserKind::Unverified => cprintln!("<s><k!>{}</k!> <k!>({})</k!>", user.name, id),
		}
	}

	Ok(())
}

async fn status(data: &mut CmdData) -> anyhow::Result<()> {
	data.ensure_config().await?;
	let config = data.config.get();

	match config.auth.get_user() {
		Some(user) => {
			cprint!("<g>Logged in as ");
			let user_name = &user.name;
			match user.kind {
				UserKind::Microsoft => cprint!("<s,g!>{}", user_name),
				UserKind::Demo => cprint!("<s,c!>{}", user_name),
				UserKind::Unverified => cprint!("<s,k!>{}", user_name),
			}
			cprintln!(" <k!>({})</k!>", user.id);
		}
		None => cprintln!("<r>Currently logged out"),
	}

	Ok(())
}

pub async fn run(subcommand: UserSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match subcommand {
		UserSubcommand::List => list(data).await,
		UserSubcommand::Status => status(data).await,
	}
}
