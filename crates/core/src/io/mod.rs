use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use anyhow::Context;
use serde::de::DeserializeOwned;
use serde::Serialize;

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
	Ok(simd_json::from_reader(file)?)
}

/// Writes JSON to a file with a buffer
pub fn json_to_file<S: Serialize>(path: impl AsRef<Path>, data: &S) -> anyhow::Result<()> {
	let file = BufWriter::new(File::create(path).context("Failed to open file")?);
	simd_json::to_writer(file, data).context("Failed to serialize data to file")?;
	Ok(())
}

/// Writes JSON to a file with a buffer and pretty formatting
pub fn json_to_file_pretty<S: Serialize>(path: impl AsRef<Path>, data: &S) -> anyhow::Result<()> {
	let file = BufWriter::new(File::create(path).context("Failed to open file")?);
	simd_json::to_writer_pretty(file, data).context("Failed to serialize data to file")?;
	Ok(())
}
