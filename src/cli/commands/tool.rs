use clap::Subcommand;
use reqwest::Client;

use super::CmdData;

#[derive(Debug, Subcommand)]
pub enum ToolSubcommand {
	#[command(about = "Run the debug authentication routine")]
	AuthTest,
}

async fn auth_test(data: &mut CmdData) -> anyhow::Result<()> {
	let client = Client::new();
	let result = mcvm::data::user::auth::authenticate_microsoft_user(
		crate::cli::get_ms_client_id(),
		&client,
		&mut data.output,
	)
	.await?;
	println!("{}", result.access_token);
	let cert = mcvm::net::minecraft::get_user_certificate(&result.access_token, &client).await?;
	dbg!(cert);

	Ok(())
}

pub async fn run(subcommand: ToolSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match subcommand {
		ToolSubcommand::AuthTest => auth_test(data).await,
	}
}
