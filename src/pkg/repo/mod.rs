use crate::io::paths::Paths;
use crate::plugin::PluginManager;
use basic::{BasicPackageRepository, RepoLocation};
use custom::CustomPackageRepository;
use mcvm_pkg::repo::{PackageFlag, RepoMetadata, RepoPkgEntry};
use mcvm_pkg::PackageContentType;

use anyhow::Context;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::translate;
use reqwest::Client;

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use super::core::{
	get_all_core_packages, get_core_package_content_type, get_core_package_count, is_core_package,
};
use super::PkgLocation;

/// Basic index-based repositories
pub mod basic;
/// Custom plugin repositories
pub mod custom;

/// A remote source for MCVM packages
pub enum PackageRepository {
	/// A basic indexed repository
	Basic(BasicPackageRepository),
	/// A custom plugin repository
	Custom(CustomPackageRepository),
	/// The internal core repository
	Core,
}

impl PackageRepository {
	/// Gets the ID of the repository
	pub fn get_id(&self) -> &str {
		match self {
			Self::Basic(repo) => &repo.id,
			Self::Core => "core",
			Self::Custom(repo) => repo.get_id(),
		}
	}

	/// Gets the displayed location of the repository
	pub fn get_displayed_location(&self) -> String {
		match self {
			PackageRepository::Basic(repo) => repo.get_location().to_string(),
			PackageRepository::Core => "Internal".into(),
			PackageRepository::Custom(repo) => {
				format!("Custom plugin: {}", repo.get_plugin_id())
			}
		}
	}

	/// Create the core repository
	pub fn core() -> Self {
		Self::Core
	}

	/// Create the std repository
	pub fn std() -> Self {
		Self::Basic(BasicPackageRepository::new(
			"std",
			RepoLocation::Remote("https://mcvm-launcher.github.io/packages/std".into()),
		))
	}

	/// Get the default set of repositories
	pub fn default_repos(enable_core: bool, enable_std: bool) -> Vec<Self> {
		let mut out = Vec::new();
		// We don't want std overriding core
		if enable_core {
			out.push(Self::core());
		}
		if enable_std {
			out.push(Self::std());
		}
		out
	}

	/// Update the currently cached index file
	pub async fn sync(&mut self, paths: &Paths, client: &Client) -> anyhow::Result<()> {
		match self {
			Self::Basic(repo) => repo.sync(paths, client).await,
			Self::Custom(_) => Ok(()),
			Self::Core => Ok(()),
		}
	}

	/// Ask if the index has a package and return the url and version for that package if it exists
	pub async fn query(
		&mut self,
		id: &str,
		paths: &Paths,
		client: &Client,
		plugins: &PluginManager,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Option<RepoQueryResult>> {
		match self {
			Self::Basic(repo) => repo.query(id, paths, client, o).await,
			Self::Core => {
				if is_core_package(id) {
					Ok(Some(RepoQueryResult {
						location: PkgLocation::Core,
						content_type: get_core_package_content_type(id)
							.expect("Core package exists and should have a content type"),
						flags: HashSet::new(),
					}))
				} else {
					Ok(None)
				}
			}
			Self::Custom(repo) => repo.query(id, plugins, paths, o),
		}
	}

	/// Get all packages from this repo
	pub async fn get_all_packages(
		&mut self,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Vec<(String, RepoPkgEntry)>> {
		match self {
			Self::Basic(repo) => repo.get_all_packages(paths, client, o).await,
			Self::Core => Ok(get_all_core_packages()),
			Self::Custom(_) => Ok(Vec::new()),
		}
	}

	/// Get the number of packages in the repo
	pub async fn get_package_count(
		&mut self,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Option<usize>> {
		match self {
			Self::Basic(repo) => repo.get_package_count(paths, client, o).await.map(Some),
			Self::Core => Ok(Some(get_core_package_count())),
			Self::Custom(_) => Ok(None),
		}
	}

	/// Get the repo's metadata
	pub async fn get_metadata(
		&mut self,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Cow<RepoMetadata>> {
		match self {
			Self::Basic(repo) => repo.get_metadata(paths, client, o).await.map(Cow::Borrowed),
			Self::Core => Ok(Cow::Owned(RepoMetadata {
				name: Some(translate!(o, CoreRepoName)),
				description: Some(translate!(o, CoreRepoDescription)),
				mcvm_version: Some(crate::VERSION.into()),
			})),
			Self::Custom(repo) => Ok(Cow::Borrowed(repo.get_meta())),
		}
	}
}

/// Query a list of repos
pub async fn query_all(
	repos: &mut [PackageRepository],
	id: &str,
	paths: &Paths,
	client: &Client,
	plugins: &PluginManager,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<Option<RepoQueryResult>> {
	for repo in repos {
		let query = match repo.query(id, paths, client, plugins, o).await {
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
	repos: &mut [PackageRepository],
	paths: &Paths,
	client: &Client,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<HashMap<String, RepoPkgEntry>> {
	let mut out = HashMap::new();
	// Iterate in reverse to make sure that repos at the beginning take precendence
	for repo in repos.iter_mut().rev() {
		let packages = repo
			.get_all_packages(paths, client, o)
			.await
			.with_context(|| {
				format!(
					"Failed to get all packages from repository '{}'",
					repo.get_id()
				)
			})?;
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
