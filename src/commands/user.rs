use super::lib::{CmdData, CmdError};
use crate::data::user::UserKind;

use color_print::{cprintln, cprint};

static LIST_HELP: &str = "List all users";
static AUTH_HELP: &str = "Show your current user";

pub fn help() {
	cprintln!("<i>user:</i> Manage mcvm users");
	cprintln!("<s>Usage:</s> mcvm user <k!><<subcommand>> [options]</k!>");
	cprintln!();
	cprintln!("<s>Subcommands:");
	cprintln!("\t <i>list:</i> {}", LIST_HELP);
	cprintln!("\t <i>auth:</i> {}", AUTH_HELP);
}

fn list(data: &mut CmdData) -> Result<(), CmdError> {
	data.ensure_config()?;

	if let Some(config) = &data.config {
		cprintln!("<s>Users:");
		for (id, user) in config.auth.users.iter() {
			cprint!("<k!> - </k!>");
			match user.kind {
				UserKind::Microsoft => cprintln!("<s><g>{}</g> <k!>({})</k!>", user.name, id),
				UserKind::Demo => cprintln!("<s><y!>{}</y!> <k!>({})</k!>", user.name, id)
			}
		}
	}
	Ok(())
}

fn auth(data: &mut CmdData) -> Result<(), CmdError> {
	data.ensure_config()?;

	if let Some(config) = &data.config {
		match config.auth.get_user() {
			Some(user) => {
				cprint!("<g>Logged in as ");
				match user.kind {
					UserKind::Microsoft => cprint!("<s,g!>{}", &user.name),
					UserKind::Demo => cprint!("<s,k!>{}", &user.name),
				}
				cprintln!(" <k!>({})</k!>", user.id);
			}
			None => cprintln!("<r>Currently logged out")
		}
	}
	Ok(())
}

pub fn run(argc: usize, argv: &[String], data: &mut CmdData)
-> Result<(), CmdError> {
	if argc == 0 {
		help();
		return Ok(());
	}

	match argv[0].as_str() {
		"list" => list(data)?,
		"auth" => auth(data)?,
		"help" => help(),
		cmd => cprintln!("<r>Unknown subcommand {}", cmd)
	}

	Ok(())
}
