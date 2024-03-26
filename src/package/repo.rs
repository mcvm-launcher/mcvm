use crate::io::files::paths::Paths;
use mcvm_core::net::download;
use mcvm_pkg::repo::{get_api_url, get_index_url, PackageFlag, RepoIndex, RepoPkgEntry};
use mcvm_pkg::PackageContentType;
use mcvm_shared::later::Later;

use anyhow::{bail, Context};
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use reqwest::Client;

use std::collections::HashSet;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::PathBuf;

use super::PkgLocation;

/// A remote source for mcvm packages
#[derive(Debug)]
pub struct PkgRepo {
	/// The identifier for the repository
	pub id: String,
	location: PkgRepoLocation,
	index: Later<RepoIndex>,
}

/// Location for a PkgRepo
#[derive(Debug)]
pub enum PkgRepoLocation {
	/// A repository on a remote device
	Remote(String),
	/// A repository on the local filesystem
	Local(PathBuf),
}

impl Display for PkgRepoLocation {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Remote(url) => write!(f, "{url}"),
			Self::Local(path) => write!(f, "{path:?}"),
		}
	}
}

impl PkgRepo {
	/// Create a new PkgRepo
	pub fn new(id: &str, location: PkgRepoLocation) -> Self {
		Self {
			id: id.to_owned(),
			location,
			index: Later::new(),
		}
	}

	/// The cached path of the index
	pub fn get_path(&self, paths: &Paths) -> PathBuf {
		paths.pkg_index_cache.join(format!("{}.json", &self.id))
	}

	/// Gets the location of the repository
	pub fn get_location(&self) -> &PkgRepoLocation {
		&self.location
	}

	/// Set the index to serialized json text
	fn set_index(&mut self, index: &mut impl std::io::Read) -> anyhow::Result<()> {
		let parsed = serde_json::from_reader(index)?;
		self.index.fill(parsed);
		Ok(())
	}

	/// Update the currently cached index file
	pub async fn sync(&mut self, paths: &Paths, client: &Client) -> anyhow::Result<()> {
		match &self.location {
			PkgRepoLocation::Local(path) => {
				let bytes = tokio::fs::read(path).await?;
				tokio::fs::write(self.get_path(paths), &bytes).await?;
				let mut cursor = Cursor::new(&bytes);
				self.set_index(&mut cursor).context("Failed to set index")?;
			}
			PkgRepoLocation::Remote(url) => {
				let bytes = download::bytes(get_index_url(url), client)
					.await
					.context("Failed to download index")?;
				tokio::fs::write(self.get_path(paths), &bytes)
					.await
					.context("Failed to write index to cached file")?;
				let mut cursor = Cursor::new(&bytes);
				self.set_index(&mut cursor).context("Failed to set index")?;
			}
		}

		Ok(())
	}

	/// Make sure that the repository index is downloaded
	pub async fn ensure_index(&mut self, paths: &Paths, client: &Client) -> anyhow::Result<()> {
		if self.index.is_empty() {
			let path = self.get_path(paths);
			if path.exists() {
				let file = File::open(&path).context("Failed to open cached index")?;
				let mut file = BufReader::new(file);
				match self.set_index(&mut file) {
					Ok(..) => {}
					Err(..) => {
						self.sync(paths, client)
							.await
							.context("Failed to sync index")?;
					}
				};
			} else {
				self.sync(paths, client)
					.await
					.context("Failed to sync index")?;
			}
		}
		Ok(())
	}

	/// Ask if the index has a package and return the url and version for that package if it exists
	pub async fn query(
		&mut self,
		id: &str,
		paths: &Paths,
		client: &Client,
	) -> anyhow::Result<Option<RepoQueryResult>> {
		self.ensure_index(paths, client).await?;
		let index = self.index.get();
		if let Some(entry) = index.packages.get(id) {
			let location = get_package_location(entry, &self.location)
				.context("Failed to get location of package")?;
			return Ok(Some(RepoQueryResult {
				location,
				content_type: get_content_type(entry).await,
				flags: entry.flags.clone(),
			}));
		}

		Ok(None)
	}

	/// Get all packages from this repo
	pub async fn get_all_packages(
		&mut self,
		paths: &Paths,
		client: &Client,
	) -> anyhow::Result<Vec<(String, RepoPkgEntry)>> {
		self.ensure_index(paths, client).await?;
		let index = self.index.get();
		Ok(index
			.packages
			.iter()
			.map(|(id, entry)| (id.clone(), entry.clone()))
			.collect())
	}
}

/// Query a list of repos
pub async fn query_all(
	repos: &mut [PkgRepo],
	id: &str,
	paths: &Paths,
	client: &Client,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<Option<RepoQueryResult>> {
	for repo in repos {
		let query = match repo.query(id, paths, client).await {
			Ok(val) => val,
			Err(e) => {
				o.display(
					MessageContents::Error(e.to_string()),
					MessageLevel::Important,
				);
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
	client: &Client,
) -> anyhow::Result<Vec<(String, RepoPkgEntry)>> {
	// Iterate in reverse to make sure that repos at the beginning take precendence
	let mut out = Vec::new();
	for repo in repos.iter_mut().rev() {
		let packages = repo
			.get_all_packages(paths, client)
			.await
			.with_context(|| format!("Failed to get all packages from repository '{}'", repo.id))?;
		out.extend(packages);
	}

	Ok(out)
}

/// Result from repository querying. This represents an entry
/// for a package that can be accessed
pub struct RepoQueryResult {
	/// The location to copy the package from
	pub location: PkgLocation,
	/// The content type of the package
	pub content_type: PackageContentType,
	/// The flags for the package
	pub flags: HashSet<PackageFlag>,
}

/// Get the content type of a package from the repository
pub async fn get_content_type(entry: &RepoPkgEntry) -> PackageContentType {
	if let Some(content_type) = &entry.content_type {
		*content_type
	} else {
		PackageContentType::Script
	}
}

/// Gets the location of a package from it's repository entry in line with url and path rules
pub fn get_package_location(
	entry: &RepoPkgEntry,
	repo_location: &PkgRepoLocation,
) -> anyhow::Result<PkgLocation> {
	if let Some(url) = &entry.url {
		Ok(PkgLocation::Remote(Some(url.clone())))
	} else if let Some(path) = &entry.path {
		let path = PathBuf::from(path);
		match &repo_location {
			// Relative paths on remote repositories
			PkgRepoLocation::Remote(url) => {
				if path.is_relative() {
					// Trim the Path
					let path = path.to_string_lossy();
					let trimmed = path.trim_start_matches("./");

					let url = get_api_url(url);
					// Ensure a slash at the end
					let url = if url.ends_with('/') {
						url.clone()
					} else {
						url.clone() + "/"
					};
					Ok(PkgLocation::Remote(Some(url.to_owned() + trimmed)))
				} else {
					bail!("Package path on remote repository is non-relative")
				}
			}
			// Local paths
			PkgRepoLocation::Local(repo_path) => {
				let path = if path.is_relative() {
					repo_path.join(path)
				} else {
					path
				};

				Ok(PkgLocation::Local(path))
			}
		}
	} else {
		bail!("Neither url nor path entry present in package")
	}
}
