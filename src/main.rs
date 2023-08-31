mod cli;

use std::process::ExitCode;

use cli::commands::run_cli;
use color_print::cformat;

#[tokio::main]
async fn main() -> ExitCode {
	let result = run_command_with_data().await;
	if let Err(e) = result {
		eprintln!("{}", cformat!("<r>{:?}", e));
		return ExitCode::FAILURE;
	}

	ExitCode::SUCCESS
}

async fn run_command_with_data() -> anyhow::Result<()> {
	let mut data = cli::commands::CmdData::new().await?;
	run_cli(&mut data).await?;

	Ok(())
}
