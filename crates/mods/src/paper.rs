use std::{fmt::Display, path::PathBuf};

use anyhow::{anyhow, Context};
use mcvm_core::{net::download, MCVMCore};
use mcvm_shared::{output::MCVMOutput, versions::VersionInfo, Side};
use reqwest::Client;
use serde::Deserialize;

use mcvm_core::io::files::paths::Paths;

/// The main class for a Paper/Folia server
pub const PAPER_SERVER_MAIN_CLASS: &str = "io.papermc.paperclip.Main";

/// Different modes for this module, either Paper or Folia,
/// depending on the one you want to install
#[derive(Debug, Clone, Copy)]
pub enum Mode {
	/// The Paper server
	Paper,
	/// The Folia multithreaded server
	Folia,
}

impl Mode {
	fn to_str(self) -> &'static str {
		match self {
			Self::Paper => "paper",
			Self::Folia => "folia",
		}
	}
}

impl Display for Mode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Paper => write!(f, "Paper"),
			Self::Folia => write!(f, "Folia"),
		}
	}
}

/// Install Paper or Folia using the core and information about the version.
/// First, create the core and the version you want. Then, get the version info from the version.
/// Finally, run this function. Returns the JAR path and main class to add to the instance you are launching
pub async fn install_from_core(
	core: &mut MCVMCore,
	version_info: &VersionInfo,
	mode: Mode,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<(PathBuf, String)> {
	let _ = o;

	let build_num = get_newest_build(mode, &version_info.version, core.get_client())
		.await
		.context("Failed to get newest Paper/Folia build")?;
	let jar_file_name =
		get_jar_file_name(mode, &version_info.version, build_num, core.get_client())
			.await
			.context("Failed to get the API name of the Paper/Folia JAR file")?;
	download_server_jar(
		mode,
		&version_info.version,
		build_num,
		&jar_file_name,
		core.get_paths(),
		core.get_client(),
	)
	.await
	.context("Failed to download Paper/Folia JAR file")?;

	Ok((
		get_local_jar_path(mode, &version_info.version, core.get_paths()),
		PAPER_SERVER_MAIN_CLASS.into(),
	))
}

/// Get the newest build number of a PaperMC project
pub async fn get_newest_build(mode: Mode, version: &str, client: &Client) -> anyhow::Result<u16> {
	let url = format!(
		"https://api.papermc.io/v2/projects/{}/versions/{version}",
		mode.to_str(),
	);
	let resp =
		serde_json::from_str::<VersionInfoResponse>(&client.get(url).send().await?.text().await?)?;

	let build = resp
		.builds
		.iter()
		.max()
		.ok_or(anyhow!("Could not find a valid Paper/Folia version"))?;

	Ok(*build)
}

#[derive(Deserialize)]
struct VersionInfoResponse {
	builds: Vec<u16>,
}

/// Get the name of the Paper JAR file in the API.
/// This does not represent the name of the file when downloaded
/// as it will be stored in the core JAR location
pub async fn get_jar_file_name(
	mode: Mode,
	version: &str,
	build_num: u16,
	client: &Client,
) -> anyhow::Result<String> {
	let num_str = build_num.to_string();
	let url = format!(
		"https://api.papermc.io/v2/projects/{}/versions/{version}/builds/{num_str}",
		mode.to_str(),
	);
	let resp = serde_json::from_str::<BuildInfoResponse>(&download::text(&url, client).await?)?;

	Ok(resp.downloads.application.name)
}

#[derive(Deserialize)]
struct BuildInfoResponse {
	downloads: BuildInfoDownloads,
}

#[derive(Deserialize)]
struct BuildInfoDownloads {
	application: BuildInfoApplication,
}

#[derive(Deserialize)]
struct BuildInfoApplication {
	name: String,
}

/// Download the Paper server jar
pub async fn download_server_jar(
	mode: Mode,
	version: &str,
	build_num: u16,
	file_name: &str,
	paths: &Paths,
	client: &Client,
) -> anyhow::Result<()> {
	let num_str = build_num.to_string();
	let url = format!("https://api.papermc.io/v2/projects/{}/versions/{version}/builds/{num_str}/downloads/{file_name}", mode.to_str());

	let file_path = get_local_jar_path(mode, version, paths);
	download::file(&url, &file_path, client)
		.await
		.context("Failed to download Paper JAR")?;

	Ok(())
}

/// Get the path to the stored Paper JAR file
pub fn get_local_jar_path(mode: Mode, version: &str, paths: &Paths) -> PathBuf {
	mcvm_core::io::minecraft::game_jar::get_path(Side::Server, version, Some(mode.to_str()), paths)
}
