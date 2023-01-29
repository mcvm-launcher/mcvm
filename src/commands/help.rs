use super::lib::{CmdData, CmdError, COMMAND_MAP};

use color_print::cprintln;

pub fn main_help() {
	cprintln!("Mcvm: <i>A Minecraft launcher for the future");
	cprintln!("<s>Usage:</s> mcvm <k!><<subcommand>> [...]</k!>");
	cprintln!();
	cprintln!("<s>Commands:");
	cprintln!("\t<i>help:</i> show this message");
	cprintln!("\t<i>profile:</i> modify profiles");
}

pub fn help() {
	cprintln!("<i>help:</i> Get information about how to use mcvm");
	cprintln!("<s>Usage:</s> mcvm help <k!>[subcommand]</k!>");
}

pub fn run(argc: usize, argv: &[String], _data: &mut CmdData)
-> Result<(), CmdError> {
	match argc {
		0 => main_help(),
		_ => {
			let cmd_name = &argv[0];
			match COMMAND_MAP.get(cmd_name) {
				Some(cmd) => {
					cprintln!("<b>Help for command {}:", cmd_name);
					cmd.help();
				},
				None => cprintln!("<r>Help: Unknown subcommand {}", cmd_name)
			}
		}
	}

	Ok(())
}
