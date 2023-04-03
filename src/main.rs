mod commands;
mod data;
mod io;
mod net;
mod package;
mod util;

use color_print::cformat;
use commands::run_cli;
use io::files::paths::Paths;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let mut data = commands::CmdData::new();
	match run_cli(&mut data).await {
		Ok(()) => {}
		Err(e) => eprintln!("{}", cformat!("<r>{:?}", e)),
	}

	Ok(())
}
