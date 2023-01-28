mod commands;
mod data;
mod io;
mod net;
mod lib;

use std::env;

use commands::{run_command, help_command_impl};
use io::files::Paths;

fn main() {
	let argv: Vec<String> = env::args().collect();
	let argc: usize = argv.len();
	match argc {
		0 => debug_assert!(false),
		1 => help_command_impl(),
		_ => {
			let paths = Paths::new();
			let argv_slice = &argv[2..];
			let argc_slice = argc - 2;
			run_command(&argv[1], argc_slice, argv_slice, &paths);
		}
	}
}
