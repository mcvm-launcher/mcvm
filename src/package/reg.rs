use anyhow::{anyhow, Context};
use mcvm_pkg::metadata::PackageMetadata;
use mcvm_pkg::parse_and_validate;
use mcvm_pkg::properties::PackageProperties;
use mcvm_pkg::PackageContentType;
use mcvm_pkg::PkgRequest;
use mcvm_pkg::PkgRequestSource;
use mcvm_shared::output::MCVMOutput;
use mcvm_shared::pkg::ArcPkgReq;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::core::{get_core_package_content_type, is_core_package};
use super::eval::{EvalData, EvalInput, Routine};
use super::repo::{query_all, PkgRepo};
use super::{Package, PkgContents, PkgLocation};
use crate::io::files::paths::Paths;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// An object used to store and cache all of the packages that we are working with.
/// It queries repositories automatically when asking for a package that isn't in the
/// registry, and prevents having a bunch of copies of packages everywhere.
#[derive(Debug)]
pub struct PkgRegistry {
	/// The package repositories that the user has configured
	pub repos: Vec<PkgRepo>,
	packages: HashMap<ArcPkgReq, Package>,
	caching_strategy: CachingStrategy,
}

impl PkgRegistry {
	/// Create a new PkgRegistry
	pub fn new(repos: Vec<PkgRepo>, caching_strategy: CachingStrategy) -> Self {
		Self {
			repos,
			packages: HashMap::new(),
			caching_strategy,
		}
	}

	fn insert(&mut self, req: ArcPkgReq, pkg: Package) -> &mut Package {
		self.packages.insert(req.clone(), pkg);
		self.packages
			.get_mut(&req)
			.expect("Package was not inserted into map")
	}

	/// Checks if a package is in the registry already
	pub fn has_now(&self, req: &PkgRequest) -> bool {
		self.packages.contains_key(req)
	}

	async fn query_insert(
		&mut self,
		req: &ArcPkgReq,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<&mut Package> {
		let pkg_id = req.id.clone();

		// First check the remote repositories
		if let Some(result) = query_all(&mut self.repos, &pkg_id, paths, client, o).await? {
			return Ok(self.insert(
				req.clone(),
				Package::new(
					pkg_id,
					PkgLocation::Remote(Some(result.url)),
					result.content_type,
				),
			));
		}

		// Now check if it exists as a core package
		if is_core_package(&req.id) {
			let content_type =
				get_core_package_content_type(&pkg_id).expect("Core package should exist");
			Ok(self.insert(
				req.clone(),
				Package::new(pkg_id, PkgLocation::Core, content_type),
			))
		} else {
			Err(anyhow!("Package '{pkg_id}' does not exist"))
		}
	}

	async fn get(
		&mut self,
		req: &ArcPkgReq,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<&mut Package> {
		if self.has_now(req) {
			Ok(self.packages.get_mut(req).expect("Package does not exist"))
		} else {
			self.query_insert(req, paths, client, o).await
		}
	}

	/// Ensure package contents while following the caching strategy
	async fn ensure_package_contents(
		&mut self,
		req: &ArcPkgReq,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<&mut Package> {
		let force = matches!(self.caching_strategy, CachingStrategy::None);
		let pkg = self
			.get(req, paths, client, o)
			.await
			.with_context(|| format!("Failed to get package {req}"))?;
		pkg.ensure_loaded(paths, force, client)
			.await
			.with_context(|| format!("Failed to load package {req}"))?;
		Ok(pkg)
	}

	/// Ensure that a package is in the registry
	pub async fn ensure_package(
		&mut self,
		req: &ArcPkgReq,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		self.get(req, paths, client, o)
			.await
			.with_context(|| format!("Failed to get package {req}"))?;

		Ok(())
	}

	/// Get the metadata of a package
	pub async fn get_metadata<'a>(
		&'a mut self,
		req: &ArcPkgReq,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<&'a PackageMetadata> {
		let pkg = self.ensure_package_contents(req, paths, client, o).await?;
		pkg.get_metadata(paths, client)
			.await
			.context("Failed to get metadata from package")
	}

	/// Get the properties of a package
	pub async fn get_properties<'a>(
		&'a mut self,
		req: &ArcPkgReq,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<&'a PackageProperties> {
		let pkg = self.ensure_package_contents(req, paths, client, o).await?;
		pkg.get_properties(paths, client)
			.await
			.context("Failed to get properties from package")
	}

	/// Load the contents of a package
	pub async fn load(
		&mut self,
		req: &ArcPkgReq,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<String> {
		let pkg = self.ensure_package_contents(req, paths, client, o).await?;
		let contents = pkg.data.get().get_text();
		Ok(contents)
	}

	/// Parse and validate a package
	pub async fn parse_and_validate(
		&mut self,
		req: &ArcPkgReq,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		let pkg = self.ensure_package_contents(req, paths, client, o).await?;
		let contents = &pkg.data.get().get_text();

		parse_and_validate(contents, pkg.content_type)?;

		Ok(())
	}

	/// Parse a package and get the contents
	pub async fn parse<'a>(
		&'a mut self,
		req: &ArcPkgReq,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<&'a PkgContents> {
		let pkg = self.ensure_package_contents(req, paths, client, o).await?;
		pkg.parse(paths, client)
			.await
			.context("Failed to parse package")?;
		Ok(pkg.data.get().contents.get())
	}

	/// Evaluate a package
	pub async fn eval<'a>(
		&mut self,
		req: &ArcPkgReq,
		paths: &Paths,
		routine: Routine,
		input: EvalInput<'a>,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<EvalData<'a>> {
		let pkg = self.ensure_package_contents(req, paths, client, o).await?;
		let eval = pkg.eval(paths, routine, input, client).await?;
		Ok(eval)
	}

	/// Remove a cached package
	pub async fn remove_cached(
		&mut self,
		req: &ArcPkgReq,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		let pkg = self
			.get(req, paths, client, o)
			.await
			.with_context(|| format!("Failed to get package {req}"))?;
		pkg.remove_cached(paths)?;
		Ok(())
	}

	/// Insert a local package into the registry
	pub fn insert_local(&mut self, req: &ArcPkgReq, path: &Path, content_type: PackageContentType) {
		self.insert(
			req.clone(),
			Package::new(
				req.id.clone(),
				PkgLocation::Local(path.to_path_buf()),
				content_type,
			),
		);
	}

	/// Iterator over all package requests in the registry
	pub fn iter_requests(&self) -> impl Iterator<Item = &ArcPkgReq> {
		self.packages.keys()
	}

	/// Get all of the package requests in the registry in an owned manner
	pub fn get_all_packages(&self) -> Vec<ArcPkgReq> {
		self.iter_requests().cloned().collect()
	}

	/// Remove cached packages
	async fn remove_cached_packages(
		&mut self,
		packages: impl Iterator<Item = &ArcPkgReq>,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		for package in packages {
			self.remove_cached(package, paths, client, o)
				.await
				.with_context(|| format!("Failed to remove cached package '{package}'"))?;
		}

		Ok(())
	}

	/// Update cached package scripts based on the caching strategy
	pub async fn update_cached_packages(
		&mut self,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		let packages = super::repo::get_all_packages(&mut self.repos, paths, client)
			.await
			.context("Failed to retrieve all packages from repos")?
			.iter()
			.map(|(id, ..)| Arc::new(PkgRequest::new(id.clone(), PkgRequestSource::Repository)))
			.collect::<Vec<_>>();
		self.remove_cached_packages(packages.iter(), paths, client, o)
			.await
			.context("Failed to remove all cached packages")?;

		if let CachingStrategy::All = self.caching_strategy {
			for package in packages {
				self.ensure_package(&package, paths, client, o)
					.await
					.with_context(|| {
						format!("Failed to get cached contents of package '{package}'")
					})?;
			}
		}

		Ok(())
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

#[cfg(test)]
mod tests {
	use super::*;
	use std::{path::PathBuf, sync::Arc};

	#[test]
	fn test_reg_insert() {
		let mut reg = PkgRegistry::new(vec![], CachingStrategy::Lazy);
		reg.insert_local(
			&Arc::new(PkgRequest::new("test", PkgRequestSource::UserRequire)),
			&PathBuf::from("./test"),
			PackageContentType::Script,
		);
		let req = PkgRequest::new(
			"test",
			PkgRequestSource::Dependency(Arc::new(PkgRequest::new(
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
			PkgRequestSource::Dependency(Arc::new(PkgRequest::new(
				"bar",
				PkgRequestSource::Dependency(Arc::new(PkgRequest::new(
					"baz",
					PkgRequestSource::Repository,
				))),
			))),
		);
		let debug = req.debug_sources(String::new());
		assert_eq!(debug, "Repository -> baz -> bar -> foo");
	}
}
