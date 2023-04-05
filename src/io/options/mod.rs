pub mod client;
mod read;
pub mod server;

use std::fs::File;
use std::path::PathBuf;

use anyhow::Context;
use serde::Deserialize;

use self::read::parse_options;
use super::files::paths::Paths;
use client::ClientOptions;
use server::ServerOptions;

/// General options structure used to produce options for both client and server
#[derive(Deserialize, Debug, Clone)]
pub struct Options {
	#[serde(default)]
	pub client: Option<ClientOptions>,
	#[serde(default)]
	pub server: Option<ServerOptions>,
}

/// Get the path to the options file
pub fn get_path(paths: &Paths) -> PathBuf {
	paths.project.config_dir().join("options.json")
}

/// Read the options.json file
pub async fn read_options(paths: &Paths) -> anyhow::Result<Option<Options>> {
	let path = get_path(paths);
	if !path.exists() {
		return Ok(None);
	}
	let mut file = File::open(path).context("Failed to open options file")?;
	Ok(Some(
		parse_options(&mut file).context("Failed to read options")?,
	))
}
