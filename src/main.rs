mod commands;
mod data;
mod io;
mod net;
mod package;
mod util;

use std::env;

use commands::{help, run_command};
use io::files::paths::Paths;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
