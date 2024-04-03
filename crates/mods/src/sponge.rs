use std::{collections::HashMap, path::PathBuf};

use anyhow::{anyhow, Context};
use mcvm_core::{net::download, MCVMCore};
use mcvm_shared::{output::MCVMOutput, versions::VersionInfo, Side};
use reqwest::Client;
use serde::Deserialize;

use mcvm_core::io::files::paths::Paths;

/// The main class for a Sponge server
pub const SPONGE_SERVER_MAIN_CLASS: &str = "org.spongepowered.server.launch.VersionCheckingMain";

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

/// Install Sponge using the core and information about the version.
/// First, create the core and the version you want. Then, get the version info from the version.
/// Finally, run this function. Returns the JAR path and main class to add to the instance you are launching
pub async fn install_from_core(
	core: &mut MCVMCore,
	version_info: &VersionInfo,
	mode: Mode,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<(PathBuf, String)> {
	let _ = o;

	let sponge_version = get_newest_version(mode, &version_info.version, core.get_client())
		.await
		.context("Failed to get latest version for Sponge")?;
	download_server_jar(
		mode,
		&version_info.version,
		&sponge_version,
		core.get_paths(),
		core.get_client(),
	)
	.await
	.context("Failed to download Sponge JAR file")?;

	Ok((
		get_local_jar_path(mode, &version_info.version, core.get_paths()),
		SPONGE_SERVER_MAIN_CLASS.into(),
	))
}

/// Get the newest version of a Sponge project
pub async fn get_newest_version(
	mode: Mode,
	version: &str,
	client: &Client,
) -> anyhow::Result<Version> {
	let url = format!(
		"https://dl-api.spongepowered.org/v1/org.spongepowered/{}/downloads?type=stable&minecraft={version}",
		mode.to_str(),
	);
	let resp: Vec<Version> = download::json(url, client).await?;

	let version = resp
		.first()
		.ok_or(anyhow!("Could not find a valid Sponge version"))?;

	Ok(version.clone())
}

/// Information about a version of a Sponge project, from their API
#[derive(Deserialize, Clone)]
pub struct Version {
	artifacts: HashMap<String, Artifact>,
}

/// A single download artifact
#[derive(Deserialize, Debug, Clone)]
pub struct Artifact {
	/// URL to download the artifact from
	url: String,
}

/// Download the Sponge server jar
pub async fn download_server_jar(
	mode: Mode,
	version: &str,
	sponge_version: &Version,
	paths: &Paths,
	client: &Client,
) -> anyhow::Result<()> {
	// For some reason this is what the artifact is called
	let artifact = sponge_version
		.artifacts
		.get(":")
		.context("Sponge version missing JAR artifact")?;

	let file_path = get_local_jar_path(mode, version, paths);
	download::file(&artifact.url, &file_path, client)
		.await
		.context("Failed to download Sponge JAR file")?;

	Ok(())
}

/// Get the path to the stored Sponge JAR file
pub fn get_local_jar_path(mode: Mode, version: &str, paths: &Paths) -> PathBuf {
	mcvm_core::io::minecraft::game_jar::get_path(Side::Server, version, Some(mode.to_str()), paths)
}
