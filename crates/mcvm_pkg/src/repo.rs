#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::PackageContentType;

/// JSON format for a repository index
#[derive(Debug, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct RepoIndex {
	/// Metadata for the repository
	#[serde(default)]
	pub metadata: RepoMetadata,
	/// The packages available from the repository
	#[serde(default)]
	pub packages: HashMap<String, RepoPkgEntry>,
}

/// Metadata for a package repository
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct RepoMetadata {
	/// The display name of the repository
	#[serde(default)]
	pub name: Option<String>,
	/// The short description of the repository
	#[serde(default)]
	pub description: Option<String>,
}

/// An entry in the repository index package list that specifies information about the package
#[derive(Debug, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct RepoPkgEntry {
	/// The URL to the package file
	pub url: String,
	/// Override for the content type of this package
	pub content_type: Option<PackageContentType>,
}

/// Get the URL of the repository index file
pub fn get_index_url(base_url: &str) -> String {
	// Remove trailing slash
	let base_url = if base_url.ends_with('/') {
		&base_url[..base_url.len() - 1]
	} else {
		base_url
	};

	base_url.to_string() + "/api/mcvm/index.json"
}
