use anyhow::Context;
use clap::Subcommand;
use reqwest::Client;

use super::CmdData;

#[derive(Debug, Subcommand)]
pub enum ToolSubcommand {
	#[command(about = "Run the debug authentication routine")]
	AuthTest,
	#[command(about = "Query the Modrinth API")]
	Modrinth {
		#[command(subcommand)]
		command: ModrinthSubcommand,
	},
}

#[derive(Debug, Subcommand)]
pub enum ModrinthSubcommand {
	#[command(about = "Get a Modrinth project")]
	GetProject {
		/// The slug or ID of the project
		project: String,
	},
	#[command(about = "Get a Modrinth project version")]
	GetVersion {
		/// The version ID
		version: String,
	},
}

pub async fn run(subcommand: ToolSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match subcommand {
		ToolSubcommand::AuthTest => auth_test(data).await,
		ToolSubcommand::Modrinth { command } => match command {
			ModrinthSubcommand::GetProject { project } => get_modrinth_project(data, project).await,
			ModrinthSubcommand::GetVersion { version } => get_modrinth_version(data, version).await,
		},
	}
}

async fn auth_test(data: &mut CmdData) -> anyhow::Result<()> {
	let client = Client::new();
	let result = mcvm::data::user::auth::authenticate_microsoft_user(
		crate::secrets::get_ms_client_id(),
		&client,
		&mut data.output,
	)
	.await?;
	println!("{}", result.access_token);
	let cert = mcvm::net::minecraft::get_user_certificate(&result.access_token, &client).await?;
	dbg!(cert);

	Ok(())
}

async fn get_modrinth_project(_data: &mut CmdData, project: String) -> anyhow::Result<()> {
	let client = Client::new();

	let project = mcvm::net::modrinth::get_project_raw(&project, &client)
		.await
		.context("Failed to get project")?;
	let project_pretty = mcvm::util::json::format_json(&project);

	let out = if let Ok(val) = project_pretty {
		val
	} else {
		project
	};

	println!("{out}");

	Ok(())
}

async fn get_modrinth_version(_data: &mut CmdData, version: String) -> anyhow::Result<()> {
	let client = Client::new();

	let version = mcvm::net::modrinth::get_version_raw(&version, &client)
		.await
		.context("Failed to get project version")?;
	let version_pretty = mcvm::util::json::format_json(&version);

	let out = if let Ok(val) = version_pretty {
		val
	} else {
		version
	};

	println!("{out}");

	Ok(())
}
