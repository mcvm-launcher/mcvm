use super::lib::{CmdData, CmdError, COMMAND_MAP};
use crate::util::print::HYPHEN_POINT;

use color_print::cprintln;

pub fn main_help() {
	cprintln!("mcvm: <i>A Minecraft launcher for the future");
	cprintln!("<s>Usage:</s> mcvm <k!><<command>> [...]</k!>");
	cprintln!();
	cprintln!("<s>Commands:");
	cprintln!("{}<i,c>help:</i,c> show this message", HYPHEN_POINT);
	cprintln!(
		"{}<i,c>version:</i,c> show mcvm's current version",
		HYPHEN_POINT
	);
	cprintln!("{}<i,c>profile:</i,c> modify profiles", HYPHEN_POINT);
	cprintln!("{}<i,c>user:</i,c> modify users", HYPHEN_POINT);
	cprintln!("{}<i,c>launch:</i,c> play the game", HYPHEN_POINT);
	cprintln!("{}<i,c>package:</i,c> manage packages", HYPHEN_POINT);
	cprintln!("{}<i,c>files:</i,c> work with internal files", HYPHEN_POINT);
}

pub fn help() {
	cprintln!("<i>help:</i> Get information about how to use mcvm");
	cprintln!("<s>Usage:</s> mcvm help <k!>[subcommand]</k!>");
}

pub fn run(argc: usize, argv: &[String], _data: &mut CmdData) -> Result<(), CmdError> {
	match argc {
		0 => main_help(),
		_ => {
			let cmd_name = &argv[0];
			match COMMAND_MAP.get(cmd_name) {
				Some(cmd) => {
					cprintln!("<b>Help for command {}:", cmd_name);
					cmd.help();
				}
				None => cprintln!("<r>Help: Unknown subcommand {}", cmd_name),
			}
		}
	}

	Ok(())
}
