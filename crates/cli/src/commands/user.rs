use super::CmdData;
use crate::output::HYPHEN_POINT;
use itertools::Itertools;
use mcvm::core::user::UserKind;

use clap::Subcommand;
use color_print::{cprint, cprintln};

#[derive(Debug, Subcommand)]
pub enum UserSubcommand {
	#[command(about = "List all users")]
	#[clap(alias = "ls")]
	List {
		/// Whether to remove formatting and warnings from the output
		#[arg(short, long)]
		raw: bool,
	},
	#[command(about = "Get current authentication status")]
	Status,
}

pub async fn run(subcommand: UserSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match subcommand {
		UserSubcommand::List { raw } => list(data, raw).await,
		UserSubcommand::Status => status(data).await,
	}
}

async fn list(data: &mut CmdData, raw: bool) -> anyhow::Result<()> {
	data.ensure_config(!raw).await?;
	let config = data.config.get();

	if !raw {
		cprintln!("<s>Users:");
	}
	for (id, user) in config.users.iter_users().sorted_by_key(|x| x.0) {
		cprint!("{}", HYPHEN_POINT);
		if raw {
			println!("{id}");
		} else {
			match user.get_kind() {
				UserKind::Microsoft { .. } => {
					cprintln!("<s><g>{}</g> <k!>({})</k!>", user.get_name(), id)
				}
				UserKind::Demo => cprintln!("<s><c!>{}</c!> <k!>({})</k!>", user.get_name(), id),
				UserKind::Unverified => {
					cprintln!("<s><k!>{}</k!> <k!>({})</k!>", user.get_name(), id)
				}
			}
		}
	}

	Ok(())
}

async fn status(data: &mut CmdData) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get();

	match config.users.get_chosen_user() {
		Some(user) => {
			let user_valid = user.is_auth_valid(&data.paths.core);
			if user_valid {
				cprint!("<g>Logged in as ");
			} else {
				cprint!("<g>User chosen as ");
			}
			let user_name = user.get_name();
			match user.get_kind() {
				UserKind::Microsoft { .. } => cprint!("<s,g!>{}", user_name),
				UserKind::Demo => cprint!("<s,c!>{}", user_name),
				UserKind::Unverified => cprint!("<s,k!>{}", user_name),
			}
			cprint!(" <k!>({})</k!>", user.get_id());

			if !user_valid {
				cprint!(" - <r>Currently logged out");
			}
			cprintln!();
		}
		None => cprintln!("<r>No user chosen"),
	}

	Ok(())
}
