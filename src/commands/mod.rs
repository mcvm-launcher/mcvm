pub mod lib;
pub mod help;
mod profile;
mod user;
mod launch;
mod version;
use lib::{Command, CmdData, COMMAND_MAP};

use color_print::cprintln;

use self::lib::CmdError;

impl Command {
	pub fn run(&self, argc: usize, argv: &[String], data: &mut CmdData)
	-> Result<(), CmdError> {
		match self {
			Self::Help => help::run(argc, argv, data),
			Self::Profile => profile::run(argc, argv, data),
			Self::User => user::run(argc, argv, data),
			Self::Launch => launch::run(argc, argv, data),
			Self::Version => version::run(argc, argv, data)
		}
	}

	pub fn help(&self) {
		match self {
			Self::Help => help::help(),
			Self::Profile => profile::help(),
			Self::User => user::help(),
			Self::Launch => launch::help(),
			Self::Version => version::help()
		}
	}
}

pub fn run_command(command: &str, argc: usize, argv: &[String], data: &mut CmdData) {
	let result = COMMAND_MAP.get(command);
	match result {
		Some(cmd) => match cmd.run(argc, argv, data) {
			Ok(..) => {},
			Err(err) => cprintln!("<r>Error occurred in command:\n{}", err)
		},
		None => cprintln!("<r>Error: {} is not a valid command", command)
	}
}
