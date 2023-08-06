use crate::io::files::paths::Paths;
use crate::net::download;
use crate::util::print::print_err;
use mcvm_pkg::repo::{RepoPkgEntry, RepoPkgIndex};
use mcvm_pkg::PackageContentType;
use mcvm_shared::later::Later;

use anyhow::Context;
use reqwest::Client;

use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;

/// A remote source for mcvm packages
#[derive(Debug)]
pub struct PkgRepo {
	pub id: String,
	url: String,
	index: Later<RepoPkgIndex>,
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
		paths.pkg_index_cache.join(format!("{}.json", &self.id))
	}

	/// Set the index to serialized json text
	fn set_index(&mut self, index: &mut impl std::io::Read) -> anyhow::Result<()> {
		let parsed = serde_json::from_reader(index)?;
		self.index.fill(parsed);
		Ok(())
	}

	/// Update the currently cached index file
	pub async fn sync(&mut self, paths: &Paths) -> anyhow::Result<()> {
		let bytes = download::bytes(&self.index_url(), &Client::new())
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

	/// Ask if the index has a package and return the url and version for that package if it exists
	pub async fn query(
		&mut self,
		id: &str,
		paths: &Paths,
	) -> anyhow::Result<Option<RepoQueryResult>> {
		self.ensure_index(paths).await?;
		let index = self.index.get();
		if let Some(entry) = index.packages.get(id) {
			return Ok(Some(RepoQueryResult {
				url: entry.url.clone(),
				version: entry.version,
				content_type: get_content_type(entry).await,
			}));
		}

		Ok(None)
	}

	/// Get all packages from this repo
	pub async fn get_all_packages(
		&mut self,
		paths: &Paths,
	) -> anyhow::Result<Vec<(String, RepoPkgEntry)>> {
		self.ensure_index(paths).await?;
		let index = self.index.get();
		Ok(index
			.packages
			.iter()
			.map(|(name, entry)| (name.clone(), entry.clone()))
			.collect())
	}
}

/// Result from repository querying
pub struct RepoQueryResult {
	pub url: String,
	pub version: u32,
	pub content_type: PackageContentType,
}

/// Get the content type of a package from the repository
pub async fn get_content_type(entry: &RepoPkgEntry) -> PackageContentType {
	if let Some(content_type) = &entry.content_type {
		content_type.clone()
	} else {
		PackageContentType::Script
	}
}

/// Query a list of repos
pub async fn query_all(
	repos: &mut [PkgRepo],
	name: &str,
	paths: &Paths,
) -> anyhow::Result<Option<RepoQueryResult>> {
	for repo in repos {
		let query = match repo.query(name, paths).await {
			Ok(val) => val,
			Err(e) => {
				print_err(e);
				continue;
			}
		};
		if query.is_some() {
			return Ok(query);
		}
	}
	Ok(None)
}

/// Get all packages from a list of repositories with the normal priority order
pub async fn get_all_packages(
	repos: &mut [PkgRepo],
	paths: &Paths,
) -> anyhow::Result<Vec<(String, RepoPkgEntry)>> {
	// Iterate in reverse to make sure that repos at the beginning take precendence
	let mut out = Vec::new();
	for repo in repos.iter_mut().rev() {
		let packages = repo
			.get_all_packages(paths)
			.await
			.with_context(|| format!("Failed to get all packages from repository '{}'", repo.id))?;
		out.extend(packages);
	}

	Ok(out)
}
