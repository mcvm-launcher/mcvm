mod commands;
mod data;
mod io;
mod net;
mod util;
mod package;

use std::env;

use commands::{run_command, help, lib::CmdError};
use io::files::paths::Paths;

#[tokio::main]
async fn main() -> Result<(), CmdError> {
	let argv: Vec<String> = env::args().collect();
	let argc: usize = argv.len();
	match argc {
		0 => debug_assert!(false),
		1 => help::main_help(),
		_ => {
			let mut data = commands::lib::CmdData::new();
			let argv_slice = &argv[2..];
			let argc_slice = argc - 2;
			run_command(&argv[1], argc_slice, argv_slice, &mut data).await;
		}
	}
	Ok(())
}
