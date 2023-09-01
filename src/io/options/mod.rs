/// Options management for the client
pub mod client;
/// Common utilties for reading and parsing options-related files
mod read;
/// Options management for the server
pub mod server;

use std::fs::File;
use std::io::BufReader;
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
	/// Options for the client
	#[serde(default)]
	pub client: Option<ClientOptions>,
	/// Options for the server
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
	let file = File::open(path).context("Failed to open options file")?;
	let mut file = BufReader::new(file);
	Ok(Some(
		parse_options(&mut file).context("Failed to read options")?,
	))
}
