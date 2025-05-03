use std::{collections::HashMap, path::PathBuf};

use anyhow::Context;
use mcvm_core::net::download;
use mcvm_shared::Side;
use reqwest::Client;
use serde::Deserialize;

use mcvm_core::io::files::paths::Paths;

/// The main class for a Sponge server
pub const SPONGE_SERVER_MAIN_CLASS: &str =
	"org.spongepowered.vanilla.installer.VersionCheckingMain";

/// Different modes for this module,
/// depending on the one you want to install
#[derive(Debug, Clone, Copy)]
pub enum Mode {
	/// Vanilla with Sponge API
	Vanilla,
}

impl Mode {
	fn to_str(self) -> &'static str {
		match self {
			Self::Vanilla => "spongevanilla",
		}
	}
}

/// Get the available artifacts of a Sponge project
pub async fn get_artifacts(
	mode: Mode,
	version: &str,
	client: &Client,
) -> anyhow::Result<Vec<String>> {
	let url = format!(
		"https://dl-api.spongepowered.org/v2/groups/org.spongepowered/artifacts/{}/versions?tags=,minecraft:{version}",
		mode.to_str(),
	);
	let resp: Versions = download::json(url, client).await?;

	let artifacts = resp.artifacts.into_keys();
	Ok(artifacts.collect())
}

#[derive(Deserialize, Clone)]
struct Versions {
	artifacts: HashMap<String, Artifact>,
}

/// A single download artifact
#[derive(Deserialize, Debug, Clone)]
struct Artifact {}

/// Fetches information about an artifact from the API
pub async fn get_artifact_info(
	mode: Mode,
	artifact: &str,
	client: &Client,
) -> anyhow::Result<ArtifactInfo> {
	let url = format!(
		"https://dl-api.spongepowered.org/v2/groups/org.spongepowered/artifacts/{}/versions/{artifact}",
		mode.to_str()
	);
	let resp = download::json(url, client).await?;

	Ok(resp)
}

/// Download the Sponge server jar
pub async fn download_server_jar(
	mode: Mode,
	version: &str,
	artifact_info: &ArtifactInfo,
	paths: &Paths,
	client: &Client,
) -> anyhow::Result<()> {
	let asset = artifact_info
		.assets
		.iter()
		.find(|x| x.classifier == "universal" && x.extension == "jar")
		.context("Failed to find server JAR file in the asset list")?;

	let file_path = get_local_jar_path(mode, version, paths);
	download::file(&asset.download_url, &file_path, client)
		.await
		.context("Failed to download Sponge JAR file")?;

	Ok(())
}

/// Information about a Sponge artifact
#[derive(Deserialize, Clone)]
pub struct ArtifactInfo {
	/// The assets associated with this artifact
	assets: Vec<Asset>,
}

/// A single downloadable asset
#[derive(Deserialize, Clone, Debug)]
pub struct Asset {
	/// The URL to download the asset at
	#[serde(rename = "downloadUrl")]
	download_url: String,
	/// Says what this asset is for
	classifier: String,
	/// The file extension
	extension: String,
}

/// Get the path to the stored Sponge JAR file
pub fn get_local_jar_path(mode: Mode, version: &str, paths: &Paths) -> PathBuf {
	mcvm_core::io::minecraft::game_jar::get_path(Side::Server, version, Some(mode.to_str()), paths)
}
