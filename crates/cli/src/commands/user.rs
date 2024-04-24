use super::CmdData;
use crate::{output::HYPHEN_POINT, secrets::get_ms_client_id};
use anyhow::{bail, Context};
use itertools::Itertools;
use mcvm::core::user::UserKind;

use clap::Subcommand;
use color_print::{cprint, cprintln};
use reqwest::Client;

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
	#[command(about = "Update the passkey for a user")]
	Passkey {
		/// The user to update the passkey for. If not specified, uses the default user
		#[arg(short, long)]
		user: Option<String>,
	},
	#[command(about = "Ensure that a user is authenticated")]
	Auth {
		/// The user to authenticate. If not specified, uses the default user
		#[arg(short, long)]
		user: Option<String>,
	},
}

pub async fn run(subcommand: UserSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match subcommand {
		UserSubcommand::List { raw } => list(data, raw).await,
		UserSubcommand::Status => status(data).await,
		UserSubcommand::Passkey { user } => passkey(data, user).await,
		UserSubcommand::Auth { user } => auth(data, user).await,
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
					cprintln!("<s><g>{}</g>", id)
				}
				UserKind::Demo => cprintln!("<s><c!>{}</c!>", id),
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
			match user.get_kind() {
				UserKind::Microsoft { .. } => cprint!("<s,g!>{}", user.get_id()),
				UserKind::Demo => cprint!("<s,c!>{}", user.get_id()),
			}

			if !user_valid {
				cprint!(" - <r>Currently logged out");
			}
			cprintln!();
		}
		None => cprintln!("<r>No user chosen"),
	}

	Ok(())
}

async fn passkey(data: &mut CmdData, user: Option<String>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get();
	let user = if let Some(user) = user {
		config.users.get_user(&user)
	} else {
		config.users.get_chosen_user()
	};
	let Some(user) = user else {
		bail!("Specified user does not exist");
	};

	user.update_passkey(&data.paths.core, &mut data.output)
		.context("Failed to update passkey")?;

	Ok(())
}

async fn auth(data: &mut CmdData, user: Option<String>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();
	let user = if let Some(user) = user {
		config.users.get_user_mut(&user)
	} else {
		config.users.get_chosen_user_mut()
	};
	let Some(user) = user else {
		bail!("Specified user does not exist");
	};

	let client = Client::new();
	user.authenticate(
		true,
		false,
		get_ms_client_id(),
		&data.paths.core,
		&client,
		&mut data.output,
	)
	.await
	.context("Failed to update passkey")?;

	Ok(())
}
