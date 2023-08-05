use serde::{Deserialize, Serialize};

use std::collections::HashMap;

/// An entry in the repository package index that specifies what package versions are available
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RepoPkgEntry {
	/// The latest package version available from this repository.
	pub version: u32,
	pub url: String,
}

/// JSON format for a repository package index
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RepoPkgIndex {
	pub packages: HashMap<String, RepoPkgEntry>,
}
