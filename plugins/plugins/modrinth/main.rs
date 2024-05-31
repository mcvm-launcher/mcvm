use anyhow::Context;
use clap::Parser;
use mcvm_core::net::download::Client;
use mcvm_plugin::api::CustomPlugin;

fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::new("modrinth")?;
	plugin.subcommand(|_, args| {
		let Some(subcommand) = args.first() else {
			return Ok(());
		};
		if subcommand != "modrinth" && subcommand != "mr" {
			return Ok(());
		}
		// Trick the parser to give it the right bin name
		let it = std::iter::once(format!("mcvm {subcommand}")).chain(args.into_iter().skip(1));
		let cli = Cli::parse_from(it);

		let runtime = tokio::runtime::Runtime::new()?;
		runtime.block_on(async {
			match cli.subcommand {
				Subcommand::GetProject { project } => get_modrinth_project(project).await,
				Subcommand::GetVersion { version } => get_modrinth_version(version).await,
			}
		})?;

		Ok(())
	})?;

	Ok(())
}

#[derive(clap::Parser)]
struct Cli {
	#[command(subcommand)]
	subcommand: Subcommand,
}

#[derive(Debug, clap::Subcommand)]
enum Subcommand {
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

async fn get_modrinth_project(project: String) -> anyhow::Result<()> {
	let client = Client::new();

	let project = mcvm_net::modrinth::get_project_raw(&project, &client)
		.await
		.context("Failed to get project")?;
	let project_pretty = mcvm::core::util::json::format_json(&project);

	let out = if let Ok(val) = project_pretty {
		val
	} else {
		project
	};

	println!("{out}");

	Ok(())
}

async fn get_modrinth_version(version: String) -> anyhow::Result<()> {
	let client = Client::new();

	let version = mcvm_net::modrinth::get_version_raw(&version, &client)
		.await
		.context("Failed to get project version")?;
	let version_pretty = mcvm::core::util::json::format_json(&version);

	let out = if let Ok(val) = version_pretty {
		val
	} else {
		version
	};

	println!("{out}");

	Ok(())
}
