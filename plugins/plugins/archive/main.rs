use std::{
	collections::{HashMap, HashSet},
	path::Path,
};

use anyhow::Context;
use clap::Parser;
use color_print::cprintln;
use mcvm_core::{io::json_from_file, net::game_files::assets::AssetIndex};
use mcvm_plugin::api::CustomPlugin;

fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::from_manifest_file("archive", include_str!("plugin.json"))?;
	plugin.subcommand(|ctx, args| {
		let Some(subcommand) = args.first() else {
			return Ok(());
		};
		if subcommand != "archive" {
			return Ok(());
		}
		// Trick the parser to give it the right bin name
		let it = std::iter::once(format!("mcvm {subcommand}")).chain(args.into_iter().skip(1));
		let cli = Cli::parse_from(it);

		let data_dir = ctx.get_data_dir()?;

		let runtime = tokio::runtime::Runtime::new()?;
		runtime.block_on(async {
			match cli.subcommand {
				Subcommand::Version { version } => archive_version(&data_dir, &version).await,
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
	#[command(about = "Remove the assets for a Minecraft version")]
	Version {
		/// The Minecraft version
		version: String,
	},
}

async fn archive_version(data_dir: &Path, version: &str) -> anyhow::Result<()> {
	// First load all of the asset indexes
	let mut indexes = HashMap::new();
	for entry in data_dir
		.join("internal/assets/indexes")
		.read_dir()
		.context("Failed to read asset index directory")?
	{
		let entry = entry?;
		let path = entry.path();
		let Some(file_stem) = path.file_stem() else {
			continue;
		};

		let version = file_stem.to_string_lossy().to_string();

		let data: AssetIndex = json_from_file(&path)
			.with_context(|| format!("Failed to read asset index for version {version}"))?;

		indexes.insert(version, (data, path));
	}

	// Get the index for the version we want to remove
	let (version_index, version_index_path) = indexes
		.remove(version)
		.context("Version not found in asset indexes. Are you sure it is installed?")?;
	cprintln!("<s>Comparing assets...");
	let mut unique_assets = HashSet::with_capacity(version_index.objects.len());
	unique_assets.extend(version_index.objects.values().map(|x| x.hash.clone()));
	for (index, _) in indexes.values() {
		for object in index.objects.values() {
			unique_assets.remove(&object.hash);
		}
	}

	cprintln!("<s>Removing {} assets...", unique_assets.len());
	for hash in unique_assets {
		let subpath = format!("{}/{hash}", &hash[0..2]);
		let path = data_dir.join("internal/assets/objects").join(subpath);
		if path.exists() {
			let _ = std::fs::remove_file(path);
		}
	}

	// Remove the index so it doesn't affect other indexes anymore
	std::fs::remove_file(version_index_path).context("Failed to remove asset index")?;

	cprintln!("<s><g>Done.");

	Ok(())
}
