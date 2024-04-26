mod commands;
mod output;
mod secrets;

use std::process::ExitCode;

use commands::run_cli;

#[tokio::main]
async fn main() -> ExitCode {
	let result = run_cli().await;
	if result.is_err() {
		return ExitCode::FAILURE;
	}

	ExitCode::SUCCESS
}
