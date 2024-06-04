use super::CmdData;
use crate::output::{icons_enabled, HYPHEN_POINT, STAR};
use anyhow::{bail, Context};
use itertools::Itertools;
use mcvm::config::modifications::{apply_modifications_and_write, ConfigModification};
use mcvm::config::user::{UserConfig, UserVariant};
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
	#[command(about = "Log out a user")]
	Logout {
		/// The user to log out. If not specified, uses the default user
		#[arg(short, long)]
		user: Option<String>,
	},
	#[command(about = "Add new users to your config")]
	Add {},
}

pub async fn run(subcommand: UserSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match subcommand {
		UserSubcommand::List { raw } => list(data, raw).await,
		UserSubcommand::Status => status(data).await,
		UserSubcommand::Passkey { user } => passkey(data, user).await,
		UserSubcommand::Auth { user } => auth(data, user).await,
		UserSubcommand::Logout { user } => logout(data, user).await,
		UserSubcommand::Add {} => add(data).await,
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
					cprint!("<s><g>{}</g>", id)
				}
				UserKind::Demo => cprint!("<s><c!>{}</c!>", id),
				UserKind::Unknown(other) => cprint!("<s><k!>({other}) {}</k!>", id),
			}
			if let Some(chosen) = config.users.get_chosen_user() {
				if chosen.get_id() == id {
					if icons_enabled() {
						cprint!("<y> {}", STAR);
					} else {
						cprint!("<s> (Default)");
					}
				}
			}
			println!();
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
				UserKind::Unknown(other) => cprint!("<s,k!>({other}) {}", user.get_id()),
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
	if let Some(user) = user {
		config.users.choose_user(&user)?;
	}

	let client = Client::new();
	config
		.users
		.authenticate(&data.paths.core, &client, &mut data.output)
		.await
		.context("Failed to authenticate")?;

	Ok(())
}

async fn logout(data: &mut CmdData, user: Option<String>) -> anyhow::Result<()> {
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

	user.logout(&data.paths.core)
		.context("Failed to logout user")?;

	Ok(())
}

async fn add(data: &mut CmdData) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let mut config = data.get_raw_config()?;

	// Build the user
	let id = inquire::Text::new("What is the ID for the user?").prompt()?;

	let options = vec![UserVariant::Microsoft {}, UserVariant::Demo {}];
	let kind = inquire::Select::new("What kind of user is this?", options).prompt()?;

	let user = UserConfig { variant: kind };

	apply_modifications_and_write(
		&mut config,
		vec![ConfigModification::AddUser(id, user)],
		&data.paths,
	)
	.context("Failed to write modified config")?;

	cprintln!("<g>User added.");

	Ok(())
}
