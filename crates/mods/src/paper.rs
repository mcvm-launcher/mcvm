use std::{fmt::Display, path::PathBuf};

use anyhow::{anyhow, bail, Context};
use mcvm_core::{net::download, MCVMCore};
use mcvm_shared::{output::MCVMOutput, versions::VersionInfo, Side};
use reqwest::Client;
use serde::Deserialize;

use mcvm_core::io::files::paths::Paths;

/// The main class for a Paper/Folia server
pub const PAPER_SERVER_MAIN_CLASS: &str = "io.papermc.paperclip.Main";

/// The main class for the Velocity proxy
pub const VELOCITY_MAIN_CLASS: &str = "com.velocitypowered.proxy.Velocity";

/// Different modes for this module, depending on which project you want to install
#[derive(Debug, Clone, Copy)]
pub enum Mode {
	/// The Paper server
	Paper,
	/// The Folia multithreaded server
	Folia,
	/// The Velocity proxy
	Velocity,
}

impl Mode {
	fn to_str(self) -> &'static str {
		match self {
			Self::Paper => "paper",
			Self::Folia => "folia",
			Self::Velocity => "velocity",
		}
	}
}

impl Display for Mode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Paper => write!(f, "Paper"),
			Self::Folia => write!(f, "Folia"),
			Self::Velocity => write!(f, "Velocity"),
		}
	}
}

/// Install Paper or Folia using the core and information about the version.
/// This function will throw an error if Velocity is passed as a mode.
/// First, create the core and the version you want. Then, get the version info from the version.
/// Finally, run this function. Returns the JAR path and main class to add to the instance you are launching
pub async fn install_from_core(
	core: &mut MCVMCore,
	version_info: &VersionInfo,
	mode: Mode,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<(PathBuf, String)> {
	let _ = o;

	if let Mode::Velocity = mode {
		bail!("Velocity is a proxy and cannot be used in the install_from_core function");
	}

	let build_num = get_newest_build(mode, &version_info.version, core.get_client())
		.await
		.context(format!("Failed to get newest {mode} build"))?;
	let jar_file_name =
		get_jar_file_name(mode, &version_info.version, build_num, core.get_client())
			.await
			.context(format!("Failed to get the API name of the {mode} JAR file"))?;
	download_server_jar(
		mode,
		&version_info.version,
		build_num,
		&jar_file_name,
		core.get_paths(),
		core.get_client(),
	)
	.await
	.context(format!("Failed to download {mode} JAR file"))?;

	Ok((
		get_local_jar_path(mode, &version_info.version, core.get_paths()),
		PAPER_SERVER_MAIN_CLASS.into(),
	))
}

/// Install Velocity, returning the path to the JAR file and the main class
pub async fn install_velocity(paths: &Paths, client: &Client) -> anyhow::Result<(PathBuf, String)> {
	let version = get_newest_version(Mode::Velocity, client)
		.await
		.context("Failed to get newest Velocity version")?;
	let build_num = get_newest_build(Mode::Velocity, &version, client)
		.await
		.context("Failed to get newest Velocity build version")?;
	let file_name = get_jar_file_name(Mode::Velocity, &version, build_num, client)
		.await
		.context("Failed to get Velocity build file name")?;

	download_server_jar(
		Mode::Velocity,
		&version,
		build_num,
		&file_name,
		paths,
		client,
	)
	.await
	.context("Failed to download Velocity JAR")?;

	Ok((
		get_local_jar_path(Mode::Velocity, &version, paths),
		VELOCITY_MAIN_CLASS.into(),
	))
}

/// Get the newest version of a PaperMC project
pub async fn get_newest_version(mode: Mode, client: &Client) -> anyhow::Result<String> {
	let url = format!("https://api.papermc.io/v2/projects/{}", mode.to_str(),);
	let resp =
		serde_json::from_str::<ProjectInfoResponse>(&client.get(url).send().await?.text().await?)?;

	let version = resp
		.versions
		.last()
		.ok_or(anyhow!("Could not find a valid {mode} version"))?;

	Ok(version.clone())
}

#[derive(Deserialize)]
struct ProjectInfoResponse {
	versions: Vec<String>,
}

/// Get the newest build number of a PaperMC project version
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
		.ok_or(anyhow!("Could not find a valid {mode} build version"))?;

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

/// Download the server jar
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
		.context("Failed to download {mode} JAR")?;

	Ok(())
}

/// Get the path to the stored JAR file
pub fn get_local_jar_path(mode: Mode, version: &str, paths: &Paths) -> PathBuf {
	mcvm_core::io::minecraft::game_jar::get_path(Side::Server, version, Some(mode.to_str()), paths)
}
