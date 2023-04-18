use crate::io::files::paths::Paths;
use crate::io::Later;
use crate::net::download;
use crate::skip_fail;

use anyhow::Context;
use serde::Deserialize;

use std::collections::HashMap;
use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;

/// An entry in the index that specifies what package versions are available
#[derive(Debug, Deserialize)]
pub struct PkgEntry {
	/// The latest package version available from this repository.
	version: u32,
	url: String,
}

/// JSON format for a repository index
#[derive(Debug, Deserialize)]
pub struct RepoIndex {
	packages: HashMap<String, PkgEntry>,
}

/// A remote source for mcvm packages
#[derive(Debug)]
pub struct PkgRepo {
	pub id: String,
	url: String,
	index: Later<RepoIndex>,
}

impl PkgRepo {
	pub fn new(id: &str, url: &str) -> Self {
		Self {
			id: id.to_owned(),
			url: url.to_owned(),
			index: Later::new(),
		}
	}

	/// The cached path of the index
	pub fn get_path(&self, paths: &Paths) -> PathBuf {
		paths.pkg_index_cache.join(&self.id)
	}

	/// Set the index to serialized json text
	fn set_index(&mut self, index: &mut impl std::io::Read) -> anyhow::Result<()> {
		let parsed = serde_json::from_reader(index)?;
		self.index.fill(parsed);
		Ok(())
	}

	/// Update the currently cached index file
	pub async fn sync(&mut self, paths: &Paths) -> anyhow::Result<()> {
		let bytes = download::bytes(&self.index_url())
			.await
			.context("Failed to download index")?;
		let mut cursor = Cursor::new(&bytes);
		tokio::fs::write(self.get_path(paths), &bytes)
			.await
			.context("Failed to write index to cached file")?;
		self.set_index(&mut cursor).context("Failed to set index")?;

		Ok(())
	}

	/// Make sure that the repository index is downloaded
	pub async fn ensure_index(&mut self, paths: &Paths) -> anyhow::Result<()> {
		if self.index.is_empty() {
			let path = self.get_path(paths);
			if path.exists() {
				let mut file = File::open(&path).context("Failed to open cached index")?;
				match self.set_index(&mut file) {
					Ok(..) => {}
					Err(..) => {
						self.sync(paths).await.context("Failed to sync index")?;
					}
				};
			} else {
				self.sync(paths).await.context("Failed to sync index")?;
			}
		}
		Ok(())
	}

	fn index_url(&self) -> String {
		self.url.clone() + "/api/mcvm/index.json"
	}

	/// Ask if the index has a package and return the url for that package if it exists
	pub async fn query(
		&mut self,
		id: &str,
		paths: &Paths,
	) -> anyhow::Result<Option<(String, u32)>> {
		self.ensure_index(paths).await?;
		let index = self.index.get();
		if let Some(entry) = index.packages.get(id) {
			return Ok(Some((entry.url.clone(), entry.version)));
		}

		Ok(None)
	}
}

/// Query a list of repos
pub async fn query_all(
	repos: &mut [PkgRepo],
	name: &str,
	paths: &Paths,
) -> anyhow::Result<Option<(String, u32)>> {
	for repo in repos {
		if let Some(result) = skip_fail!(repo.query(name, paths).await) {
			return Ok(Some(result));
		}
	}
	Ok(None)
}
