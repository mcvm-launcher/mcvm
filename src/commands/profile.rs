use super::lib::{CmdData, CmdError};
use crate::data::instance::InstKind;
use crate::util::print::HYPHEN_POINT;

use color_print::cprintln;

static LIST_HELP: &str = "List all profiles and their instances";
static UPDATE_HELP: &str = "Update the packages and instances of a profile";

pub fn help() {
	cprintln!("<i>profile:</i> Manage mcvm profiles");
	cprintln!("<s>Usage:</s> mcvm profile <k!><<subcommand>> [options]</k!>");
	cprintln!();
	cprintln!("<s>Subcommands:");
	cprintln!("{}<i,c>list:</i,c> {}", HYPHEN_POINT, LIST_HELP);
	cprintln!("{}<i,c>update:</i,c> {}", HYPHEN_POINT, UPDATE_HELP);
}

fn list(data: &mut CmdData) -> Result<(), CmdError> {
	data.ensure_config()?;

	if let Some(config) = &data.config {
		cprintln!("<s>Profiles:");
		for (id, profile) in config.profiles.iter() {
			cprintln!("\t<s><g>{}", id);
			for inst_id in profile.instances.iter() {
				if let Some(instance) = config.instances.get(inst_id) {
					match instance.kind {
						InstKind::Client => cprintln!("\t{}<y!>{}", HYPHEN_POINT, inst_id),
						InstKind::Server => cprintln!("\t{}<c!>{}", HYPHEN_POINT, inst_id)
					}
				}
			}
		}
	}
	Ok(())
}

async fn update(data: &mut CmdData, id: &String) -> Result<(), CmdError> {
	data.ensure_paths()?;
	data.ensure_config()?;

	if let Some(config) = &mut data.config {
		if let Some(paths) = &data.paths {
			if let Some(profile) = config.profiles.get_mut(id) {
				profile.create_instances(&mut config.instances, paths, true, false).await?;
			} else {
				return Err(CmdError::Custom(format!("Unknown profile '{id}'")));
			}
		}
	}
	Ok(())
}

pub async fn run(argc: usize, argv: &[String], data: &mut CmdData)
-> Result<(), CmdError> {
	if argc == 0 {
		help();
		return Ok(());
	}

	match argv[0].as_str() {
		"list" => list(data)?,
		"update" => match argc {
			1 => cprintln!("{}", LIST_HELP),
			_ => update(data, &argv[1]).await?,
		}
		"help" => help(),
		cmd => cprintln!("<r>Unknown subcommand {}", cmd)
	}

	Ok(())
}
