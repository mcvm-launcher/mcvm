use anyhow::{anyhow, Context};
use color_print::cformat;
use mcvm_parse::metadata::PackageMetadata;
use mcvm_parse::parse::Parsed;
use mcvm_parse::properties::PackageProperties;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::core::is_core_package;
use super::eval::{EvalData, Routine, EvalInput};
use super::repo::{query_all, PkgRepo};
use super::{Package, PkgKind};
use crate::io::files::paths::Paths;

use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::path::Path;

/// Where a package was requested from
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord)]
pub enum PkgRequestSource {
	UserRequire,
	Bundled(Box<PkgRequest>),
	Dependency(Box<PkgRequest>),
	Refused(Box<PkgRequest>),
	Repository,
}

impl PartialOrd for PkgRequestSource {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.to_num().cmp(&other.to_num()))
	}
}

impl PkgRequestSource {
	/// Gets the source package of this package, if any
	pub fn get_source(&self) -> Option<&PkgRequest> {
		match self {
			Self::Dependency(source) | Self::Bundled(source) => Some(source),
			_ => None,
		}
	}

	/// Gets whether this source list is only bundles that lead up to a UserRequire
	pub fn is_user_bundled(&self) -> bool {
		matches!(self, Self::Bundled(source) if source.source.is_user_bundled())
			|| matches!(self, Self::UserRequire)
	}

	/// Converts to a number, used for ordering
	fn to_num(&self) -> u8 {
		match self {
			Self::UserRequire => 0,
			Self::Bundled(..) => 1,
			Self::Dependency(..) => 2,
			Self::Refused(..) => 3,
			Self::Repository => 4,
		}
	}
}

/// Used to store a request for a package that will be fulfilled later
#[derive(Debug, Clone, PartialOrd, Ord)]
pub struct PkgRequest {
	pub source: PkgRequestSource,
	pub name: String,
}

impl PkgRequest {
	pub fn new(name: &str, source: PkgRequestSource) -> Self {
		Self {
			name: name.to_owned(),
			source,
		}
	}

	/// Create a dependency list for debugging. Recursive, so call with an empty string
	pub fn debug_sources(&self, list: String) -> String {
		match &self.source {
			PkgRequestSource::UserRequire => format!("{}{list}", self.name),
			PkgRequestSource::Dependency(source) => {
				format!("{} -> {}", source.debug_sources(list), self.name)
			}
			PkgRequestSource::Refused(source) => {
				format!("{} =X=> {}", source.debug_sources(list), self.name)
			}
			PkgRequestSource::Bundled(bundler) => {
				format!("{} => {}", bundler.debug_sources(list), self.name)
			}
			PkgRequestSource::Repository => format!("Repository -> {}{list}", self.name),
		}
	}

	/// Print with color formatting
	pub fn disp_with_colors(&self) -> String {
		match self.source {
			PkgRequestSource::UserRequire => cformat!("<y>{}", self.name),
			PkgRequestSource::Bundled(..) => cformat!("<b>{}", self.name),
			PkgRequestSource::Refused(..) => cformat!("<r>{}", self.name),
			PkgRequestSource::Dependency(..) | PkgRequestSource::Repository => {
				cformat!("<c>{}", self.name)
			}
		}
	}
}

impl PartialEq for PkgRequest {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name
	}
}

impl Eq for PkgRequest {}

impl Hash for PkgRequest {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.name.hash(state);
	}
}

impl Display for PkgRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.name)
	}
}

/// What strategy to use for the local caching of package scripts
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CachingStrategy {
	/// Don't cache any packages locally. Fetch them from the repository every time
	None,
	/// Only cache packages when they are requested
	#[default]
	Lazy,
	/// Cache all packages whenever syncing the repositories
	All,
}

#[derive(Debug)]
pub struct PkgRegistry {
	pub repos: Vec<PkgRepo>,
	packages: HashMap<PkgRequest, Package>,
	caching_strategy: CachingStrategy,
}

impl PkgRegistry {
	pub fn new(repos: Vec<PkgRepo>, caching_strategy: CachingStrategy) -> Self {
		Self {
			repos,
			packages: HashMap::new(),
			caching_strategy,
		}
	}

	fn insert(&mut self, req: &PkgRequest, pkg: Package) -> &mut Package {
		self.packages.insert(req.clone(), pkg);
		self.packages
			.get_mut(req)
			.expect("Package was not inserted into map")
	}

	/// Checks if a package is in the registry already
	pub fn has_now(&self, req: &PkgRequest) -> bool {
		self.packages.contains_key(req)
	}

	async fn query_insert(
		&mut self,
		req: &PkgRequest,
		paths: &Paths,
	) -> anyhow::Result<&mut Package> {
		let pkg_name = req.name.clone();

		// First check the remote repositories
		if let Some((url, version)) = query_all(&mut self.repos, &pkg_name, paths).await? {
			return Ok(self.insert(
				req,
				Package::new(&pkg_name, version, PkgKind::Remote(Some(url))),
			));
		}

		// Now check if it exists as a core package
		if is_core_package(&req.name) {
			Ok(self.insert(req, Package::new(&pkg_name, 1, PkgKind::Core)))
		} else {
			Err(anyhow!("Package '{pkg_name}' does not exist"))
		}
	}

	async fn get(&mut self, req: &PkgRequest, paths: &Paths) -> anyhow::Result<&mut Package> {
		if self.has_now(req) {
			Ok(self.packages.get_mut(req).expect("Package does not exist"))
		} else {
			self.query_insert(req, paths).await
		}
	}

	/// Ensure package contents while following the caching strategy
	async fn ensure_package_contents(
		&mut self,
		req: &PkgRequest,
		paths: &Paths,
		client: &Client,
	) -> anyhow::Result<&mut Package> {
		let force = matches!(self.caching_strategy, CachingStrategy::None);
		let pkg = self.get(req, paths).await?;
		pkg.ensure_loaded(paths, force, client).await?;
		Ok(pkg)
	}

	/// Ensure that a package is in the registry
	pub async fn ensure_package(&mut self, req: &PkgRequest, paths: &Paths) -> anyhow::Result<()> {
		self.get(req, paths).await?;

		Ok(())
	}

	/// Get the version of a package
	pub async fn get_version(&mut self, req: &PkgRequest, paths: &Paths) -> anyhow::Result<u32> {
		let pkg = self.get(req, paths).await?;
		Ok(pkg.id.version)
	}

	/// Get the metadata of a package
	pub async fn get_metadata<'a>(
		&'a mut self,
		req: &PkgRequest,
		paths: &Paths,
		client: &Client,
	) -> anyhow::Result<&'a PackageMetadata> {
		let pkg = self.ensure_package_contents(req, paths, client).await?;
		pkg.get_metadata(paths, client)
			.await
			.context("Failed to get metadata from package")
	}

	/// Get the properties of a package
	pub async fn get_properties<'a>(
		&'a mut self,
		req: &PkgRequest,
		paths: &Paths,
		client: &Client,
	) -> anyhow::Result<&'a PackageProperties> {
		let pkg = self.ensure_package_contents(req, paths, client).await?;
		pkg.get_properties(paths, client)
			.await
			.context("Failed to get properties from package")
	}

	/// Load the contents of a package
	pub async fn load(
		&mut self,
		req: &PkgRequest,
		paths: &Paths,
		client: &Client,
	) -> anyhow::Result<String> {
		let pkg = self.ensure_package_contents(req, paths, client).await?;
		let contents = pkg.data.get().get_contents();
		Ok(contents)
	}

	/// Parse a package
	pub async fn parse<'a>(
		&'a mut self,
		req: &PkgRequest,
		paths: &Paths,
		client: &Client,
	) -> anyhow::Result<&'a Parsed> {
		let pkg = self.ensure_package_contents(req, paths, client).await?;
		pkg.parse(paths, client)
			.await
			.context("Failed to parse package")?;
		Ok(pkg.data.get().parsed.get())
	}

	/// Evaluate a package
	pub async fn eval<'a>(
		&mut self,
		req: &PkgRequest,
		paths: &Paths,
		routine: Routine,
		input: EvalInput<'a>,
		client: &Client,
	) -> anyhow::Result<EvalData<'a>> {
		let pkg = self.ensure_package_contents(req, paths, client).await?;
		let eval = pkg.eval(paths, routine, input, client).await?;
		Ok(eval)
	}

	/// Remove a cached package
	pub async fn remove_cached(&mut self, req: &PkgRequest, paths: &Paths) -> anyhow::Result<()> {
		let pkg = self.get(req, paths).await?;
		pkg.remove_cached(paths)?;
		Ok(())
	}

	/// Insert a local package into the registry
	pub fn insert_local(&mut self, req: &PkgRequest, version: u32, path: &Path) {
		self.insert(
			req,
			Package::new(&req.name, version, PkgKind::Local(path.to_path_buf())),
		);
	}

	/// Iterator over all package requests in the registry
	pub fn iter_requests(&self) -> impl Iterator<Item = &PkgRequest> {
		self.packages.keys()
	}

	/// Get all of the package requests in the registry in an owned manner
	pub fn get_all_packages(&self) -> Vec<PkgRequest> {
		self.iter_requests().cloned().collect()
	}

	/// Remove cached packages
	async fn remove_cached_packages(
		&mut self,
		packages: impl Iterator<Item = &PkgRequest>,
		paths: &Paths,
	) -> anyhow::Result<()> {
		for package in packages {
			self.remove_cached(package, paths)
				.await
				.with_context(|| format!("Failed to remove cached package '{package}'"))?;
		}

		Ok(())
	}

	/// Update cached package scripts based on the caching strategy
	pub async fn update_cached_packages(&mut self, paths: &Paths) -> anyhow::Result<()> {
		let packages = super::repo::get_all_packages(&mut self.repos, paths)
			.await
			.context("Failed to retrieve all packages from repos")?
			.iter()
			.map(|(name, ..)| PkgRequest::new(name, PkgRequestSource::Repository))
			.collect::<Vec<_>>();
		self.remove_cached_packages(packages.iter(), paths)
			.await
			.context("Failed to remove all cached packages")?;

		if let CachingStrategy::All = self.caching_strategy {
			for package in packages {
				self.ensure_package(&package, paths)
					.await
					.with_context(|| {
						format!("Failed to get cached contents of package '{package}'")
					})?;
			}
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::PathBuf;

	#[test]
	fn test_reg_insert() {
		let mut reg = PkgRegistry::new(vec![], CachingStrategy::Lazy);
		reg.insert_local(
			&PkgRequest::new("test", PkgRequestSource::UserRequire),
			1,
			&PathBuf::from("./test"),
		);
		let req = PkgRequest::new(
			"test",
			PkgRequestSource::Dependency(Box::new(PkgRequest::new(
				"hello",
				PkgRequestSource::UserRequire,
			))),
		);
		assert!(reg.has_now(&req));
		assert!(!reg.has_now(&PkgRequest::new(
			"doesnotexist",
			PkgRequestSource::UserRequire
		)));
	}

	#[test]
	fn test_request_source_debug() {
		let req = PkgRequest::new(
			"foo",
			PkgRequestSource::Dependency(Box::new(PkgRequest::new(
				"bar",
				PkgRequestSource::Dependency(Box::new(PkgRequest::new(
					"baz",
					PkgRequestSource::Repository,
				))),
			))),
		);
		let debug = req.debug_sources(String::new());
		assert_eq!(debug, "Repository -> baz -> bar -> foo");
	}
}
