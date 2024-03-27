use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::Context;
use serde::de::DeserializeOwned;

/// Utilities for dealing with the filesystem
pub mod files;
/// Interaction with some of Java's formats
pub mod java;
/// I/O with Minecraft data formats
pub mod minecraft;
/// Use of a file for persistent data
pub mod persistent;
/// Management of file updates
pub mod update;

/// Reads JSON from a file with a buffer
pub fn json_from_file<D: DeserializeOwned>(path: impl AsRef<Path>) -> anyhow::Result<D> {
	let file = BufReader::new(File::open(path).context("Failed to open file")?);
	Ok(serde_json::from_reader(file)?)
}
