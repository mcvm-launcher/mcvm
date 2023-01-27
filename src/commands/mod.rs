mod help;
use crate::io::files::Paths;
use help::help_command;
pub use help::help_command_impl;

use std::collections::HashMap;

pub fn run_command(command: &str, argc: u8, argv: &[String], paths: Paths) {
	type McvmCommand = fn(u8, &[String], Paths);
	let command_map: HashMap<&str, McvmCommand> = HashMap::from([
		("help", help_command as McvmCommand)
	]);

	let result = command_map.get(command);
	match result {
		Some(func) => func(argc, argv, paths),
		None => eprintln!("Error: {} is not a valid command.", command)
	}
}
