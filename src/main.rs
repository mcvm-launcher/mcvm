mod cli;

use std::process::ExitCode;

use cli::commands::run_cli;
use color_print::cformat;

#[tokio::main]
async fn main() -> ExitCode {
	let result = run_cli().await;
	if let Err(e) = result {
		eprintln!("{}", cformat!("<r>{:?}", e));
		return ExitCode::FAILURE;
	}

	ExitCode::SUCCESS
}
