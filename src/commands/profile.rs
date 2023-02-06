use super::lib::{CmdData, CmdError};
use crate::data::instance::InstKind;

use color_print::cprintln;

static LIST_HELP: &str = "List all profiles and their instances";
static UPDATE_HELP: &str = "Update the packages and instances of a profile";

pub fn help() {
	cprintln!("<i>profile:</i> Manage mcvm profiles");
	cprintln!("<s>Usage:</s> mcvm profile <k!><<subcommand>> [options]</k!>");
	cprintln!();
	cprintln!("<s>Subcommands:");
	cprintln!("\t <i>list:</i> {}", LIST_HELP);
	cprintln!("\t <i>update:</i> {}", UPDATE_HELP);
}

fn list(data: &mut CmdData) -> Result<(), CmdError> {
	data.config.load()?;
	if let Some(config) = &data.config.data {
		cprintln!("<s>Profiles:");
		for (id, profile) in config.profiles.iter() {
			cprintln!("\t<s><g>{}", id);
			for inst_id in profile.instances.iter() {
				if let Some(instance) = config.instances.get(inst_id) {
					match instance.kind {
						InstKind::Client => cprintln!("\t<k!> - </k!><y!>{}", inst_id),
						InstKind::Server => cprintln!("\t<k!> - </k!><c!>{}", inst_id)
					}
				}
			}
		}
	}
	Ok(())
}

fn update(data: &mut CmdData, id: &String) -> Result<(), CmdError> {
	data.config.load()?;
	if let Some(config) = &mut data.config.data {
		if let Some(profile) = config.profiles.get_mut(id) {
			profile.create_instances(&mut config.instances, &data.paths, true, false)?;
		} else {
			return Err(CmdError::Custom(format!("Unknown profile '{id}'")));
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
		"update" => match argc {
			1 => cprintln!("{}", LIST_HELP),
			_ => update(data, &argv[1])?,
		}
		"help" => help(),
		cmd => cprintln!("<r>Unknown subcommand {}", cmd)
	}

	Ok(())
}
