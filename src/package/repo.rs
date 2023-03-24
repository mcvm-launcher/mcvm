use crate::io::files::paths::Paths;
use crate::net::download::download_text;
use crate::skip_fail;

use serde::Deserialize;

use std::collections::HashMap;
use std::path::PathBuf;

// An entry in the index that specifies what package versions are available
#[derive(Debug, Deserialize)]
pub struct PkgEntry {
	// The latest package version available from this repository.
	version: String,
	url: String,
}

// JSON format for a repository index
#[derive(Debug, Deserialize)]
pub struct RepoIndex {
	packages: HashMap<String, PkgEntry>,
}

// A remote source for mcvm packages
#[derive(Debug)]
pub struct PkgRepo {
	pub id: String,
	url: String,
	index: Option<RepoIndex>,
}

impl PkgRepo {
	pub fn new(id: &str, url: &str) -> Self {
		Self {
			id: id.to_owned(),
			url: url.to_owned(),
			index: None,
		}
	}

	// The cached path of the index
	pub fn get_path(&self, paths: &Paths) -> PathBuf {
		paths.pkg_index_cache.join(&self.id)
	}

	// Set the index to serialized json text
	fn set_index(&mut self, index: &str) -> anyhow::Result<()> {
		let parsed = serde_json::from_str::<RepoIndex>(index)?;
		self.index = Some(parsed);
		Ok(())
	}

	// Update the currently cached index file
	pub async fn sync(&mut self, paths: &Paths) -> anyhow::Result<()> {
		let text = download_text(&self.index_url()).await?;
		tokio::fs::write(self.get_path(paths), &text).await?;
		self.set_index(&text)?;

		Ok(())
	}

	// Make sure that the repository index is downloaded
	pub async fn ensure_index(&mut self, paths: &Paths) -> anyhow::Result<()> {
		if self.index.is_none() {
			let path = self.get_path(paths);
			if path.exists() {
				match self.set_index(&tokio::fs::read_to_string(&path).await?) {
					Ok(..) => {}
					Err(..) => {
						self.sync(paths).await?;
						self.set_index(&tokio::fs::read_to_string(&path).await?)?;
					}
				};
			} else {
				self.sync(paths).await?;
			}
		}
		Ok(())
	}

	fn index_url(&self) -> String {
		self.url.clone() + "/api/mcvm/index.json"
	}

	// Ask if the index has a package and return the url for that package if it exists
	pub async fn query(
		&mut self,
		id: &str,
		paths: &Paths,
	) -> anyhow::Result<Option<(String, String)>> {
		self.ensure_index(paths).await?;
		if let Some(index) = &self.index {
			if let Some(entry) = index.packages.get(id) {
				return Ok(Some((entry.url.clone(), entry.version.clone())));
			}
		}
		Ok(None)
	}
}

// Query a list of repos
pub async fn query_all(
	repos: &mut [PkgRepo],
	name: &str,
	paths: &Paths,
) -> anyhow::Result<Option<(String, String)>> {
	for repo in repos {
		if let Some(result) = skip_fail!(repo.query(name, paths).await) {
			return Ok(Some(result));
		}
	}
	Ok(None)
}
