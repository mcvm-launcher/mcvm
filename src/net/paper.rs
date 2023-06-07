use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use reqwest::Client;
use serde::Deserialize;

use super::download;

#[derive(Deserialize)]
struct VersionInfoResponse {
	builds: Vec<u16>,
}

/// Get the newest build number of Paper
pub async fn get_newest_build(version: &str) -> anyhow::Result<(u16, Client)> {
	let url = format!("https://api.papermc.io/v2/projects/paper/versions/{version}");
	let client = Client::new();
	let resp =
		serde_json::from_str::<VersionInfoResponse>(&client.get(url).send().await?.text().await?)?;

	let build = resp
		.builds
		.last()
		.ok_or(anyhow!("Could not find a valid Paper version"))?;

	Ok((*build, client))
}

#[derive(Deserialize)]
struct BuildInfoApplication {
	name: String,
}

#[derive(Deserialize)]
struct BuildInfoDownloads {
	application: BuildInfoApplication,
}

#[derive(Deserialize)]
struct BuildInfoResponse {
	downloads: BuildInfoDownloads,
}

/// Get the name of the Paper jar file
pub async fn get_jar_file_name(version: &str, build_num: u16) -> anyhow::Result<String> {
	let num_str = build_num.to_string();
	let url =
		format!("https://api.papermc.io/v2/projects/paper/versions/{version}/builds/{num_str}");
	let resp =
		serde_json::from_str::<BuildInfoResponse>(&download::text(&url, &Client::new()).await?)?;

	Ok(resp.downloads.application.name)
}

/// Download the Paper server jar
pub async fn download_server_jar(
	version: &str,
	build_num: u16,
	file_name: &str,
	path: &Path,
) -> anyhow::Result<PathBuf> {
	let num_str = build_num.to_string();
	let file_path = path.join(file_name);
	let url = format!("https://api.papermc.io/v2/projects/paper/versions/{version}/builds/{num_str}/downloads/{file_name}");

	download::file(&url, &file_path, &Client::new())
		.await
		.context("Failed to download file")?;

	Ok(file_path)
}
