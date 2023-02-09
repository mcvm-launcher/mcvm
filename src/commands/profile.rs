use super::lib::{CmdData, CmdError};
use crate::data::instance::InstKind;
use crate::util::print::HYPHEN_POINT;
use crate::package::PkgKind;

use color_print::{cprintln, cprint};

static INFO_HELP: &str = "View helpful information about a profile";
static LIST_HELP: &str = "List all profiles and their instances";
static UPDATE_HELP: &str = "Update the packages and instances of a profile";
static REINSTALL_HELP: &str = "Force reinstall a profile and all its files";

pub fn help() {
	cprintln!("<i>profile:</i> Manage mcvm profiles");
	cprintln!("<s>Usage:</s> mcvm profile <k!><<subcommand>> [options]</k!>");
	cprintln!();
	cprintln!("<s>Subcommands:");
	cprintln!("{}<i,c>info:</i,c> {}", HYPHEN_POINT, INFO_HELP);
	cprintln!("{}<i,c>list:</i,c> {}", HYPHEN_POINT, LIST_HELP);
	cprintln!("{}<i,c>update:</i,c> {}", HYPHEN_POINT, UPDATE_HELP);
	cprintln!("{}<i,c>reinstall:</i,c> {}", HYPHEN_POINT, REINSTALL_HELP);
}

fn info(data: &mut CmdData, id: &String) -> Result<(), CmdError> {
	data.ensure_paths()?;
	data.ensure_config()?;

	if let Some(config) = &data.config {
		if let Some(profile) = config.profiles.get(id) {
			cprintln!("<s><g>Profile <b>{}", id);
			cprintln!("   <s>Version:</s> <b!>{}", profile.version.as_string());
			cprintln!("   <s>Instances:");
			for inst_id in profile.instances.iter() {
				if let Some(instance) = config.instances.get(inst_id) {
					cprint!("   {}", HYPHEN_POINT);
					match instance.kind {
						InstKind::Client => cprint!("<y!>Client {}", inst_id),
						InstKind::Server => cprint!("<c!>Server {}", inst_id)
					}
					cprintln!();
				}
			}
			cprintln!("   <s>Packages:");
			for pkg_id in profile.packages.iter() {
				if let Some(package) = config.packages.get(pkg_id) {
					cprint!("   {}", HYPHEN_POINT);
					match package.kind {
						PkgKind::Local(..) => cprint!("<m!>{} <g>v<g!>{}", pkg_id, package.version),
						PkgKind::Remote(..) => cprint!("<g!>{} <g>v<g!>{}", pkg_id, package.version)
					}
					cprintln!();
				}
			}
		} else {
			return Err(CmdError::Custom(format!("Unknown profile '{id}'")));
		}
	}
	Ok(())
}

fn list(data: &mut CmdData) -> Result<(), CmdError> {
	data.ensure_config()?;

	if let Some(config) = &data.config {
		cprintln!("<s>Profiles:");
		for (id, profile) in config.profiles.iter() {
			cprintln!("<s><g>   {}", id);
			for inst_id in profile.instances.iter() {
				if let Some(instance) = config.instances.get(inst_id) {
					match instance.kind {
						InstKind::Client => cprintln!("   {}<y!>{}", HYPHEN_POINT, inst_id),
						InstKind::Server => cprintln!("   {}<c!>{}", HYPHEN_POINT, inst_id)
					}
				}
			}
		}
	}
	Ok(())
}

async fn profile_update(data: &mut CmdData, id: &String, force: bool) -> Result<(), CmdError> {
	data.ensure_paths()?;
	data.ensure_config()?;

	if let Some(config) = &mut data.config {
		if let Some(paths) = &data.paths {
			if let Some(profile) = config.profiles.get_mut(id) {
				profile.create_instances(&mut config.instances, paths, true, force).await?;
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
		"info" => match argc {
			1 => cprintln!("{}", INFO_HELP),
			_ => info(data, &argv[1])?
		}
		"update" => match argc {
			1 => cprintln!("{}", UPDATE_HELP),
			_ => profile_update(data, &argv[1], false).await?
		}
		"reinstall" => match argc {
			1 => cprintln!("{}", REINSTALL_HELP),
			_ => profile_update(data, &argv[1], true).await?
		}
		"help" => help(),
		cmd => cprintln!("<r>Unknown subcommand {}", cmd)
	}

	Ok(())
}