use anyhow::Context;
use clap::Parser;
use mcvm_core::net::download::Client;
use mcvm_plugin::api::CustomPlugin;

fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::from_manifest_file("smithed_api", include_str!("plugin.json"))?;
	plugin.subcommand(|_, args| {
		let Some(subcommand) = args.first() else {
			return Ok(());
		};
		if subcommand != "smithed" {
			return Ok(());
		}
		// Trick the parser to give it the right bin name
		let it = std::iter::once(format!("mcvm {subcommand}")).chain(args.into_iter().skip(1));
		let cli = Cli::parse_from(it);

		let runtime = tokio::runtime::Runtime::new()?;
		runtime.block_on(async {
			match cli.subcommand {
				Subcommand::GetPack { pack } => get_smithed_pack(pack).await,
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
	#[command(about = "Get a Smithed pack")]
	GetPack {
		/// The slug or ID of the pack
		pack: String,
	},
}

async fn get_smithed_pack(pack: String) -> anyhow::Result<()> {
	let client = Client::new();

	let pack = mcvm_net::smithed::get_pack(&pack, &client)
		.await
		.context("Failed to get pack")?;
	let pack_pretty = serde_json::to_string_pretty(&pack)?;

	println!("{pack_pretty}");

	Ok(())
}
