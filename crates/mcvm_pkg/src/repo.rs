use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::PackageContentType;

/// An entry in the repository package index that specifies what package versions are available
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RepoPkgEntry {
	/// The URL to the package file
	pub url: String,
	/// Override for the content type of this package
	pub content_type: Option<PackageContentType>,
}

/// JSON format for a repository package index
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RepoPkgIndex {
	/// The packages available from the repository
	pub packages: HashMap<String, RepoPkgEntry>,
}
