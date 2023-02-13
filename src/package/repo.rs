use crate::net::download::{Download, DownloadError};
use crate::io::files::paths::Paths;
use super::{Package, PkgKind, PKG_EXTENSION};
use crate::util::versions::VersionPattern;

use serde::Deserialize;

use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
	#[error("File operation failed:\n{}", .0)]
	Io(#[from] std::io::Error),
	#[error("Failed to parse json file:\n{}", .0)]
	Parse(#[from] serde_json::Error),
	#[error("Download failed:\n{}", .0)]
	Download(#[from] DownloadError)
}

// An entry in the index that specifies what package versions are available
#[derive(Debug, Deserialize)]
pub struct PkgEntry {
	// A list of package versions available from this repository. Ordered from oldest to newest
	versions: Vec<String>
}

#[derive(Debug, Deserialize)]
pub struct RepoIndex {
	packages: HashMap<String, PkgEntry>
}

#[derive(Debug)]
pub struct PkgRepo {
	id: String,
	url: String,
	contents: Option<RepoIndex>
}

impl PkgRepo {
	pub fn new(id: &str, url: &str) -> Self {
		Self {
			id: id.to_owned(),
			url: url.to_owned() + "/api/mcvm",
			contents: None
		}
	}

	pub fn get_path(&self, paths: &Paths) -> PathBuf {
		paths.pkg_index_cache.join(&self.id)
	}

	fn set_contents(&mut self, contents: &str) -> Result<(), ApiError> {
		let parsed = serde_json::from_str::<RepoIndex>(contents)?;
		self.contents = Some(parsed);
		Ok(())
	}

	pub fn sync(&mut self, paths: &Paths) -> Result<(), ApiError> {
		let mut dwn = Download::new();
		dwn.url(&self.url)?;
		dwn.add_file(&self.get_path(paths))?;
		dwn.add_str();
		dwn.perform()?;
		self.set_contents(&dwn.get_str()?)?;
		Ok(())
	}

	pub fn ensure_contents(&mut self, paths: &Paths) -> Result<(), ApiError> {
		if self.contents.is_none() {
			let path = self.get_path(paths);
			if path.exists() {
				self.set_contents(&fs::read_to_string(path)?)?;
			} else {
				self.sync(paths)?;
			}
		}
		Ok(())
	}

	fn index_url(&self) -> String {
		self.url.to_owned() + "/index.json"
	}

	fn get_package_url(&self, id: &str, version: &str) -> String {
		format!("{}/pkg/{id}_{version}{}", self.url, PKG_EXTENSION)
	}

	pub fn query(&mut self, id: &str, version: &VersionPattern, paths: &Paths) -> Result<Option<Box<Package>>, ApiError> {
		self.ensure_contents(paths)?;
		if let Some(contents) = &self.contents {
			if let Some(entry) = contents.packages.get(id) {
				if let Some(found_version) = version.matches(&entry.versions) {
					let url = self.get_package_url(id, &found_version);
					let package = Package::new(id, &found_version, PkgKind::Remote(url));
					return Ok(Some(Box::new(package)));
				}
			}
		}
		Ok(None)
	}
}
