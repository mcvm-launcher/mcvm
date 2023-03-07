use std::{path::{Path, PathBuf}, fs};

use crate::util::json;
use super::download::DownloadError;

use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum PaperError {
	#[error("Download failed:\n{}", .0)]
	Download(#[from] DownloadError),
	#[error("Download failed:\n{}", .0)]
	Reqwest(#[from] reqwest::Error),
	#[error("Failed to evaluate json file:\n{}", .0)]
	ParseError(#[from] json::JsonError),
	#[error("Json operation failed:\n{}", .0)]
	SerdeJson(#[from] serde_json::Error),
	#[error("Build not found")]
	BuildNotFound,
	#[error("Filesystem operation failed:\n{}", .0)]
	Io(#[from] std::io::Error)
}

#[derive(Deserialize)]
struct VersionInfoResponse {
	builds: Vec<u16>
}

pub async fn get_newest_build(version: &str) -> Result<(u16, Client), PaperError> {
	let url = format!("https://api.papermc.io/v2/projects/paper/versions/{version}");
	let client = Client::new();
	let resp = serde_json::from_str::<VersionInfoResponse>(
		&client.get(url).send().await?.text().await?
	)?;

	let build = resp.builds.last().ok_or(PaperError::BuildNotFound)?;

	Ok((build.clone(), client))
}

#[derive(Deserialize)]
struct BuildInfoApplication {
	name: String
}

#[derive(Deserialize)]
struct BuildInfoDownloads {
	application: BuildInfoApplication
}

#[derive(Deserialize)]
struct BuildInfoResponse {
	downloads: BuildInfoDownloads
}

pub async fn download_server_jar(version: &str, build_num: u16, path: &Path) -> Result<PathBuf, PaperError> {
	let num_str = build_num.to_string();
	let url = format!("https://api.papermc.io/v2/projects/paper/versions/{version}/builds/{num_str}");
	let client = Client::new();
	let resp = serde_json::from_str::<BuildInfoResponse>(
		&client.get(url).send().await?.text().await?
	)?;
	let file_name = resp.downloads.application.name;
	let file_path = path.join(&file_name);

	let url = format!("https://api.papermc.io/v2/projects/paper/versions/{version}/builds/{num_str}/downloads/{file_name}");
	let bytes = client.get(url).send().await?.bytes().await?;
	fs::write(&file_path, bytes)?;

	Ok(file_path)
}