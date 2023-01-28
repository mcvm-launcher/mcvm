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
			let (doc, _) = match net::game_files::get_version_json(
				lib::versions::MinecraftVersion::from("1.19.3"), &paths, true
			) {
				Ok(val) => val,
				Err(err) => panic!("{}", err)
			};
			println!("{}", lib::json::access_str(doc.as_object().unwrap(), "mainClass").unwrap());
			let argv_slice = &argv[2..];
			let argc_slice = argc - 2;
			run_command(&argv[1], argc_slice, argv_slice, &paths);
		}
	}
}
