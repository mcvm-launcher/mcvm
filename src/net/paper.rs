use std::fs;
use std::path::{Path, PathBuf};

use crate::util::json;

use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum PaperError {
	#[error("Download failed:\n{}", .0)]
	Download(#[from] reqwest::Error),
	#[error("Failed to evaluate json file:\n{}", .0)]
	ParseError(#[from] json::JsonError),
	#[error("Json operation failed:\n{}", .0)]
	SerdeJson(#[from] serde_json::Error),
	#[error("Build not found")]
	BuildNotFound,
	#[error("Filesystem operation failed:\n{}", .0)]
	Io(#[from] std::io::Error),
}

#[derive(Deserialize)]
struct VersionInfoResponse {
	builds: Vec<u16>,
}

pub async fn get_newest_build(version: &str) -> Result<(u16, Client), PaperError> {
	let url = format!("https://api.papermc.io/v2/projects/paper/versions/{version}");
	let client = Client::new();
	let resp =
		serde_json::from_str::<VersionInfoResponse>(&client.get(url).send().await?.text().await?)?;

	let build = resp.builds.last().ok_or(PaperError::BuildNotFound)?;

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

pub async fn get_jar_file_name(version: &str, build_num: u16) -> Result<String, PaperError> {
	let num_str = build_num.to_string();
	let url =
		format!("https://api.papermc.io/v2/projects/paper/versions/{version}/builds/{num_str}");
	let client = Client::new();
	let resp =
		serde_json::from_str::<BuildInfoResponse>(&client.get(url).send().await?.text().await?)?;

	Ok(resp.downloads.application.name)
}

pub async fn download_server_jar(
	version: &str,
	build_num: u16,
	file_name: &str,
	path: &Path,
) -> Result<PathBuf, PaperError> {
	let num_str = build_num.to_string();
	let file_path = path.join(file_name);
	let url = format!("https://api.papermc.io/v2/projects/paper/versions/{version}/builds/{num_str}/downloads/{file_name}");

	let client = Client::new();
	let bytes = client.get(url).send().await?.error_for_status()?.bytes().await?;
	fs::write(&file_path, bytes)?;

	Ok(file_path)
}
