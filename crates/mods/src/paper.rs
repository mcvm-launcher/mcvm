use std::{fmt::Display, path::PathBuf};

use anyhow::{Context, anyhow, bail};
use nitro_core::{NitroCore, net::download};
use nitro_shared::{Side, output::NitroOutput, versions::VersionInfo};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use nitro_core::io::files::paths::Paths;

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
	/// Convert this mode to a lowercase string
	pub fn to_str(self) -> &'static str {
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
	core: &mut NitroCore,
	version_info: &VersionInfo,
	mode: Mode,
	o: &mut impl NitroOutput,
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

/// Get all versions of a PaperMC project
pub async fn get_all_versions(mode: Mode, client: &Client) -> anyhow::Result<Vec<VersionResponse>> {
	let url = format!(
		"https://fill.papermc.io/v3/projects/{}/versions",
		mode.to_str()
	);
	let resp: VersionsResponse = download::json(url, client).await?;
	Ok(resp.versions)
}

/// Get the newest version of a PaperMC project
pub async fn get_newest_version(mode: Mode, client: &Client) -> anyhow::Result<String> {
	let versions = get_all_versions(mode, client).await?;

	let version = versions
		.first()
		.ok_or(anyhow!("Could not find a valid {mode} version"))?;

	Ok(version.version.id.clone())
}

/// Get all available build numbers of a PaperMC project version
pub async fn get_builds(mode: Mode, version: &str, client: &Client) -> anyhow::Result<Vec<u16>> {
	let url = format!(
		"https://fill.papermc.io/v3/projects/{}/versions/{version}/builds",
		mode.to_str(),
	);
	let resp: Vec<BuildInfoResponse> = download::json(url, client).await?;

	Ok(resp.into_iter().map(|build| build.id as u16).collect())
}

/// Get the newest build number of a PaperMC project version
pub async fn get_newest_build(mode: Mode, version: &str, client: &Client) -> anyhow::Result<u16> {
	let builds = get_builds(mode, version, client).await?;

	let build = builds
		.iter()
		.max()
		.ok_or(anyhow!("Could not find a valid {mode} build version"))?;

	Ok(*build)
}

/// Info about a PaperMC project version
#[derive(Serialize, Deserialize)]
pub struct VersionsResponse {
	/// The project versions
	pub versions: Vec<VersionResponse>,
}

#[derive(Serialize, Deserialize)]
/// Details about a PaperMC project version and its builds
pub struct VersionResponse {
	/// The version details
	pub version: Version,
	/// The list of available build numbers
	pub builds: Vec<u16>,
}

#[derive(Serialize, Deserialize)]
/// Details about a PaperMC project version
pub struct Version {
	/// The Minecraft version
	pub id: String,
}

/// Gets info from the given build
pub async fn get_build_info(
	mode: Mode,
	version: &str,
	build_num: u16,
	client: &Client,
) -> anyhow::Result<BuildInfoResponse> {
	let num_str = build_num.to_string();
	let url = format!(
		"https://fill.papermc.io/v3/projects/{}/versions/{version}/builds/{num_str}",
		mode.to_str(),
	);
	let resp: BuildInfoResponse = download::json(url, client).await?;

	Ok(resp)
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
	let info = get_build_info(mode, version, build_num, client).await?;

	Ok(info.downloads.application.name)
}

/// Response from the build info API
#[derive(Serialize, Deserialize)]
pub struct BuildInfoResponse {
	/// The build number
	pub id: u32,
	/// The list of downloads
	pub downloads: BuildInfoDownloads,
}

/// Downloads for a build
#[derive(Serialize, Deserialize)]
pub struct BuildInfoDownloads {
	/// Server application info for the download
	#[serde(rename = "server:default")]
	pub application: BuildInfoApplication,
}

/// Application info for a build download
#[derive(Serialize, Deserialize)]
pub struct BuildInfoApplication {
	/// The name of the JAR file
	pub name: String,
	/// The direct URL of the JAR file
	pub url: String,
}

/// Download the server jar
pub async fn download_server_jar(
	mode: Mode,
	version: &str,
	_build_num: u16,
	file_url: &str,
	paths: &Paths,
	client: &Client,
) -> anyhow::Result<()> {
	let file_path = get_local_jar_path(mode, version, paths);
	download::file(file_url, &file_path, client)
		.await
		.context("Failed to download {mode} JAR")?;

	Ok(())
}

/// Get the path to the stored JAR file
pub fn get_local_jar_path(mode: Mode, version: &str, paths: &Paths) -> PathBuf {
	nitro_core::io::minecraft::game_jar::get_path(
		Side::Server,
		version,
		Some(mode.to_str()),
		&paths.jars,
	)
}
