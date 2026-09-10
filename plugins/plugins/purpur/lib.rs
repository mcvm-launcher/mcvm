use std::{
	fs::File,
	io::{BufReader, BufWriter},
	path::{Path, PathBuf},
};

use anyhow::{Context, bail};
use nitro_plugin::{
	api::wasm::{
		WASMPlugin,
		net::download_file,
		output::WASMPluginOutput,
		sys::{get_data_dir, update_link},
	},
	hook::hooks::OnInstanceSetupResult,
	nitro_wasm_plugin,
};
use nitro_shared::{
	Side,
	output::{MessageContents, NitroOutput},
};
use nitro_shared::{UpdateDepth, loaders::Loader};
use serde::Deserialize;
use wstd::{
	http::{Client, Request, StatusCode},
	runtime::block_on,
};

nitro_wasm_plugin!(main, "purpur");

fn main(plugin: &mut WASMPlugin) -> anyhow::Result<()> {
	plugin.on_instance_setup(|arg| {
		let Some(side) = arg.side else {
			bail!("Instance side is empty");
		};

		let Some(inst_dir) = &arg.inst_dir else {
			return Ok(OnInstanceSetupResult::default());
		};

		if arg.config.custom_launch {
			return Ok(OnInstanceSetupResult::default());
		};

		// Make sure this is a Paper or Folia server instance
		if side != Side::Server || arg.loader != Loader::Purpur {
			return Ok(OnInstanceSetupResult::default());
		}

		let data_dir = get_data_dir();
		let client = Client::new();

		let mut o = WASMPluginOutput::new();

		let mut process = o.get_process();
		process.display(MessageContents::StartProcess("Installing Purpur".into()));

		let builds_path = get_stored_builds_path(&data_dir, &arg.version_info.version);
		let builds = if builds_path.exists() && arg.update_depth == UpdateDepth::Shallow {
			serde_json::from_reader::<_, Vec<String>>(BufReader::new(File::open(builds_path)?))?
		} else {
			let builds = block_on(get_purpur_builds(&arg.version_info.version, &client))
				.context("Failed to get Purpur builds")?;

			if let Some(parent) = builds_path.parent() {
				let _ = std::fs::create_dir_all(parent);
			}
			serde_json::to_writer(BufWriter::new(File::create(builds_path)?), &builds)
				.context("Failed to store Purpur builds")?;

			builds
		};

		let jar_path = data_dir
			.join("internal/jars")
			.join(format!("{}_server_purpur.jar", &arg.version_info.version));

		let build = arg
			.desired_loader_version
			.get_match(&builds)
			.context("No matching Purpur build versions found")?;
		if !jar_path.exists() || arg.update_depth > UpdateDepth::Shallow {
			download_purpur_jar(&arg.version_info.version, &build, &jar_path)
				.context("Failed to download Purpur jar")?;
		}

		// Link the Mojang jar to skip redownload
		let mojang_jar_path = get_cached_mojang_jar(Path::new(inst_dir), &arg.version_info.version);
		if let Some(parent) = mojang_jar_path.parent() {
			let _ = std::fs::create_dir_all(parent);
		}
		let _ = update_link(&Path::new(&arg.game_jar_path), &mojang_jar_path);

		process.display(MessageContents::Success("Purpur Installed".into()));

		Ok(OnInstanceSetupResult {
			main_class_override: Some("org.bukkit.craftbukkit.Main".into()),
			jar_path_override: Some(jar_path.to_string_lossy().to_string()),
			loader_version: Some(build),
			..Default::default()
		})
	})?;

	Ok(())
}

fn get_stored_builds_path(data_dir: &Path, minecraft_version: &str) -> std::path::PathBuf {
	data_dir
		.join("internal/purpur")
		.join(format!("{minecraft_version}_builds.json"))
}

/// Get purpur builds for the given Minecraft version
async fn get_purpur_builds(
	minecraft_version: &str,
	client: &Client,
) -> anyhow::Result<Vec<String>> {
	let request = Request::get(format!(
		"https://api.purpurmc.org/v2/purpur/{minecraft_version}"
	))
	.body("")?;
	let mut response = client
		.send(request)
		.await
		.context("Failed to fetch Purpur builds")?;

	if response.status() == StatusCode::NOT_FOUND {
		bail!("No Purpur builds found for version {minecraft_version}");
	} else if !response.status().is_success() {
		bail!("Failed to fetch Purpur builds: HTTP {}", response.status());
	}

	let body = response.body_mut();
	let builds: PurpurBuildsResponse = body
		.json()
		.await
		.context("Failed to parse Purpur builds JSON")?;

	Ok(builds.builds.all)
}

#[derive(Deserialize)]
struct PurpurBuildsResponse {
	builds: PurpurBuilds,
}

#[derive(Deserialize)]
struct PurpurBuilds {
	// Ordered from oldest to newest
	all: Vec<String>,
}

fn download_purpur_jar(minecraft_version: &str, build: &str, path: &Path) -> anyhow::Result<()> {
	let url = format!("https://api.purpurmc.org/v2/purpur/{minecraft_version}/{build}/download",);
	download_file(&url, path).context("Failed to download Purpur jar")
}

fn get_cached_mojang_jar(inst_dir: &Path, version: &str) -> PathBuf {
	inst_dir.join("cache").join(format!("mojang_{version}.jar"))
}
