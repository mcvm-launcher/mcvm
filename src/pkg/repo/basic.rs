use std::{
	fmt::Display,
	fs::File,
	io::{BufReader, Cursor},
	path::PathBuf,
};

use anyhow::{bail, Context};
use mcvm_net::download;
use mcvm_pkg::repo::{get_api_url, get_index_url, RepoIndex, RepoMetadata, RepoPkgEntry};
use mcvm_shared::{
	later::Later,
	output::{MCVMOutput, MessageContents, MessageLevel},
	translate,
};
use reqwest::Client;

use crate::{io::paths::Paths, pkg::PkgLocation};

use super::{get_content_type, RepoQueryResult};

/// A basic repository using a package index
#[derive(Debug)]
pub struct BasicPackageRepository {
	/// The identifier for the repository
	pub id: String,
	location: RepoLocation,
	index: Later<RepoIndex>,
}

/// Location for a BasicPackageRepository
#[derive(Debug)]
pub enum RepoLocation {
	/// A repository on a remote device
	Remote(String),
	/// A repository on the local filesystem
	Local(PathBuf),
}

impl Display for RepoLocation {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Remote(url) => write!(f, "{url}"),
			Self::Local(path) => write!(f, "{path:?}"),
		}
	}
}

impl BasicPackageRepository {
	/// Create a new PkgRepo
	pub fn new(id: &str, location: RepoLocation) -> Self {
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
	pub fn get_location(&self) -> &RepoLocation {
		&self.location
	}

	/// Set the index to serialized json text
	fn set_index(&mut self, index: &mut impl std::io::Read) -> anyhow::Result<()> {
		let parsed = simd_json::from_reader(index)?;
		self.index.fill(parsed);
		Ok(())
	}

	/// Update the currently cached index file
	pub async fn sync(&mut self, paths: &Paths, client: &Client) -> anyhow::Result<()> {
		match &self.location {
			RepoLocation::Local(path) => {
				let bytes = tokio::fs::read(path).await?;
				tokio::fs::write(self.get_path(paths), &bytes).await?;
				let mut cursor = Cursor::new(&bytes);
				self.set_index(&mut cursor).context("Failed to set index")?;
			}
			RepoLocation::Remote(url) => {
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
	pub async fn ensure_index(
		&mut self,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
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

			self.check_index(o);
		}

		Ok(())
	}

	/// Checks the index. It must be already loaded.
	fn check_index(&self, o: &mut impl MCVMOutput) {
		let repo_version = &self.index.get().metadata.mcvm_version;
		if let Some(repo_version) = repo_version {
			let repo_version = version_compare::Version::from(repo_version);
			let program_version = version_compare::Version::from(crate::VERSION);
			if let (Some(repo_version), Some(program_version)) = (repo_version, program_version) {
				if repo_version > program_version {
					o.display(
						MessageContents::Warning(translate!(
							o,
							RepoVersionWarning,
							"repo" = &self.id
						)),
						MessageLevel::Important,
					);
				}
			}
		}
	}

	/// Ask if the index has a package and return the url and version for that package if it exists
	pub async fn query(
		&mut self,
		id: &str,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Option<RepoQueryResult>> {
		self.ensure_index(paths, client, o).await?;
		let index = self.index.get();
		if let Some(entry) = index.packages.get(id) {
			let location = get_package_location(entry, &self.location, &self.id)
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
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Vec<(String, RepoPkgEntry)>> {
		self.ensure_index(paths, client, o).await?;

		let index = self.index.get();
		Ok(index
			.packages
			.iter()
			.map(|(id, entry)| (id.clone(), entry.clone()))
			.collect())
	}

	/// Get the number of packages in the repo
	pub async fn get_package_count(
		&mut self,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<usize> {
		self.ensure_index(paths, client, o).await?;

		Ok(self.index.get().packages.len())
	}

	/// Get the repo's metadata
	pub async fn get_metadata(
		&mut self,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<&RepoMetadata> {
		self.ensure_index(paths, client, o).await?;

		Ok(&self.index.get().metadata)
	}
}

/// Gets the location of a package from it's repository entry in line with url and path rules
pub fn get_package_location(
	entry: &RepoPkgEntry,
	repo_location: &RepoLocation,
	repo_id: &str,
) -> anyhow::Result<PkgLocation> {
	if let Some(url) = &entry.url {
		Ok(PkgLocation::Remote {
			url: Some(url.clone()),
			repo_id: repo_id.to_string(),
		})
	} else if let Some(path) = &entry.path {
		let path = PathBuf::from(path);
		match &repo_location {
			// Relative paths on remote repositories
			RepoLocation::Remote(url) => {
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
					Ok(PkgLocation::Remote {
						url: Some(url.to_owned() + trimmed),
						repo_id: repo_id.to_string(),
					})
				} else {
					bail!("Package path on remote repository is non-relative")
				}
			}
			// Local paths
			RepoLocation::Local(repo_path) => {
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
