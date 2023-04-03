mod read;
pub mod client;
pub mod server;

use std::collections::HashMap;
use std::fs::File;
use std::path::{PathBuf, Path};

use anyhow::Context;
use itertools::Itertools;
use serde::Deserialize;

use self::read::parse_options;
use client::ClientOptions;
use server::ServerOptions;
use super::files::paths::Paths;

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
	Ok(Some(parse_options(&mut file).context("Failed to read options")?))
}

/// Write options.txt to a file
pub fn write_options_txt(
	options: &HashMap<String, String>,
	path: &Path,
) -> anyhow::Result<()> {
	let mut file = File::create(path).context("Failed to open file")?;
	for (key, value) in options.iter().sorted_by_key(|x| x.0) {
		client::write_key(key, value, &mut file)
			.with_context(|| format!("Failed to write line for option {key} with value {value}"))?;
	}
	
	Ok(())
}

/// Write server.properties to a file
pub fn write_server_properties(
	options: &HashMap<String, String>,
	path: &Path,
) -> anyhow::Result<()> {
	let mut file = File::create(path).context("Failed to open file")?;
	for (key, value) in options.iter().sorted_by_key(|x| x.0) {
		server::write_key(key, value, &mut file)
			.with_context(|| format!("Failed to write line for option {key} with value {value}"))?;
	}

	Ok(())
}
