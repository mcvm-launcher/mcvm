mod cli;

use std::process::ExitCode;

use cli::commands::run_cli;

#[tokio::main]
async fn main() -> ExitCode {
	let result = run_cli().await;
	if result.is_err() {
		return ExitCode::FAILURE;
	}

	ExitCode::SUCCESS
}
