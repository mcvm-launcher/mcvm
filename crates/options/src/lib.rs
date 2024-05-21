#![warn(missing_docs)]

//! This library is used by MCVM to provide structure for specifying
//! version-agnostic game options for both Minecraft clients and Minecraft
//! servers. It also generates the options files in a way that is non-destructive
//! to existing settings

/// Options management for the client
pub mod client;
/// Common utilties for reading and parsing options-related files
mod read;
/// Options management for the server
pub mod server;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::Context;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::Deserialize;

use self::read::parse_options;
use client::ClientOptions;
use server::ServerOptions;

/// General options structure used to produce options for both client and server
#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Options {
	/// Options for the client
	#[serde(default)]
	pub client: Option<ClientOptions>,
	/// Options for the server
	#[serde(default)]
	pub server: Option<ServerOptions>,
}

/// Read the options.json file
pub fn read_options(path: &Path) -> anyhow::Result<Option<Options>> {
	if !path.exists() {
		return Ok(None);
	}
	let file = File::open(path).context("Failed to open options file")?;
	let mut file = BufReader::new(file);
	Ok(Some(
		parse_options(&mut file).context("Failed to read options")?,
	))
}

macro_rules! match_key {
	($out:ident, $option:expr, $key:expr) => {
		if let Some(value) = $option {
			$out.insert($key.into(), value.to_string());
		}
	};

	($out:ident, $option:expr, $key:expr, $version:expr) => {
		if $version {
			match_key!($out, $option, $key)
		}
	};
}

macro_rules! match_key_int {
	($out:ident, $option:expr, $key:expr) => {
		if let Some(value) = $option {
			$out.insert($key.into(), value.to_int().to_string());
		}
	};

	($out:ident, $option:expr, $key:expr, $version:expr) => {
		if $version {
			match_key_int!($out, $option, $key)
		}
	};
}

pub(crate) use match_key;
pub(crate) use match_key_int;
