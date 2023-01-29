pub mod help;
mod profile;
use crate::io::files::Paths;

use phf_macros::phf_map;
use color_print::cprintln;

pub enum Command {
	Help,
	Profile
}

impl Command {
	pub fn run(&self, argc: usize, argv: &[String], paths: &Paths)
	-> Result<(), Box<dyn std::error::Error>> {
		match self {
			Self::Help => match argc {
				0 => help::run(argc, argv, paths),
				_ => {
					let command = &argv[0];
					match COMMAND_MAP.get(&command) {
						Some(cmd) => match cmd {
							Command::Help => help::help(),
							Command::Profile => profile::help()
						},
						None => {
							cprintln!("<r>Unknown command {}", command);
							help::help();
						}
					}
					Ok(())
				}
			},
			Self::Profile => profile::run(argc, argv, paths)
		}
	}
}

static COMMAND_MAP: phf::Map<&'static str, Command> = phf_map! {
	"help" => Command::Help,
	"profile" => Command::Profile
};

pub fn run_command(command: &str, argc: usize, argv: &[String], paths: &Paths) {
	let result = COMMAND_MAP.get(command);
	match result {
		Some(cmd) => match cmd.run(argc, argv, paths) {
			Ok(..) => {},
			Err(err) => cprintln!("<r>Error occurred in command:\n{}", err)
		},
		None => cprintln!("<r>Error: {} is not a valid command", command)
	}
}
