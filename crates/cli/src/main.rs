mod commands;
mod output;
mod secrets;

use commands::run_cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	run_cli().await
}
