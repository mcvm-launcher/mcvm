use crate::net::download::{Download, DownloadError};
use crate::io::files::paths::Paths;
use crate::util::versions::VersionPattern;

use serde::Deserialize;

use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
	#[error("File operation failed:\n{}", .0)]
	Io(#[from] std::io::Error),
	#[error("Failed to parse json file:\n{}", .0)]
	Parse(#[from] serde_json::Error),
	#[error("Download failed:\n{}", .0)]
	Download(#[from] DownloadError)
}

// An entry in the list of versions for a package
#[derive(Debug, Deserialize)]
struct PkgVersionEntry {
	name: String,
	url: String
}

// An entry in the index that specifies what package versions are available
#[derive(Debug, Deserialize)]
pub struct PkgEntry {
	// A list of package versions available from this repository. Ordered from oldest to newest
	versions: Vec<PkgVersionEntry>
}

// JSON format for a repository index
#[derive(Debug, Deserialize)]
pub struct RepoIndex {
	packages: HashMap<String, PkgEntry>
}

// A remote source for mcvm packages
#[derive(Debug)]
pub struct PkgRepo {
	pub id: String,
	url: String,
	index: Option<RepoIndex>
}

impl PkgRepo {
	pub fn new(id: &str, url: &str) -> Self {
		Self {
			id: id.to_owned(),
			url: url.to_owned(),
			index: None
		}
	}

	// The cached path of the index
	pub fn get_path(&self, paths: &Paths) -> PathBuf {
		paths.pkg_index_cache.join(&self.id)
	}

	// Set the index to serialized json text
	fn set_index(&mut self, index: &str) -> Result<(), RepoError> {
		let parsed = serde_json::from_str::<RepoIndex>(index)?;
		self.index = Some(parsed);
		Ok(())
	}

	// Update the currently cached index file
	pub fn sync(&mut self, paths: &Paths) -> Result<(), RepoError> {
		let mut dwn = Download::new();
		dwn.url(&self.index_url())?;
		dwn.add_file(&self.get_path(paths))?;
		dwn.add_str();
		dwn.perform()?;
		self.set_index(&dwn.get_str()?)?;
		Ok(())
	}

	// Make sure that the repository index is downloaded
	pub fn ensure_index(&mut self, paths: &Paths) -> Result<(), RepoError> {
		if self.index.is_none() {
			let path = self.get_path(paths);
			if path.exists() {
				self.set_index(&fs::read_to_string(path)?)?;
			} else {
				self.sync(paths)?;
			}
		}
		Ok(())
	}

	fn index_url(&self) -> String {
		self.url.clone() + "/api/mcvm/index.json"
	}

	// Ask if the index has a package and return the url for that package if it exists
	pub fn query(&mut self, id: &str, version: &VersionPattern, paths: &Paths)
	-> Result<Option<(String, String)>, RepoError> {
		self.ensure_index(paths)?;
		if let Some(index) = &self.index {
			if let Some(entry) = index.packages.get(id) {
				let versions_vec = Vec::from_iter(entry.versions.iter().map(|entry| {
					entry.name.clone()
				}));

				if let Some(found_version) = version.matches(&versions_vec) {
					let url = &entry.versions.iter().find(|entry| {
						entry.name == found_version
					}).expect("Failed to locate url for version").url;

					return Ok(Some((url.clone(), found_version)));
				}
			}
		}
		Ok(None)
	}
}

// Query a list of repos
pub fn query_all(repos: &mut [PkgRepo], id: &str, version: &VersionPattern, paths: &Paths)
-> Result<Option<(String, String)>, RepoError> {
	for repo in repos {
		if let Some(result) = repo.query(id, version, paths)? {
			return Ok(Some(result));
		}
	}
	Ok(None)
}
