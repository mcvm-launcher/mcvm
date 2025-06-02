use anyhow::{anyhow, Context};
use mcvm_core::net::download;
use mcvm_pkg::metadata::PackageMetadata;
use mcvm_pkg::parse_and_validate;
use mcvm_pkg::properties::PackageProperties;
use mcvm_pkg::repo::PackageFlag;
use mcvm_pkg::PackageContentType;
use mcvm_pkg::PkgRequest;
use mcvm_pkg::PkgRequestSource;
use mcvm_shared::output::MCVMOutput;
use mcvm_shared::pkg::ArcPkgReq;
use reqwest::Client;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use super::eval::{EvalData, EvalInput, Routine};
use super::repo::{query_all, PackageRepository};
use super::{Package, PkgContents};
use crate::io::paths::Paths;
use crate::plugin::PluginManager;

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

/// An object used to store and cache all of the packages that we are working with.
/// It queries repositories automatically when asking for a package that isn't in the
/// registry, and prevents having a bunch of copies of packages everywhere.
pub struct PkgRegistry {
	/// The package repositories that the user has configured
	pub repos: Vec<PackageRepository>,
	packages: HashMap<ArcPkgReq, Package>,
	plugins: PluginManager,
}

impl PkgRegistry {
	/// Create a new PkgRegistry with repositories
	pub fn new(repos: Vec<PackageRepository>, plugins: &PluginManager) -> Self {
		Self {
			repos,
			packages: HashMap::new(),
			plugins: plugins.clone(),
		}
	}

	/// Clear the registry
	pub fn clear(&mut self) {
		self.packages.clear();
	}

	/// Insert a package into the registry and return a mutable reference to the
	/// newly inserted package
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

	/// Query repositories to insert a package
	async fn query_insert(
		&mut self,
		req: &ArcPkgReq,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<&mut Package> {
		// First check the remote repositories
		let query = query_all(&mut self.repos, req, paths, client, &self.plugins, o)
			.await
			.context("Failed to query remote repositories")?;
		if let Some(result) = query {
			return Ok(self.insert(
				req.clone(),
				Package::new(
					req.id.clone(),
					result.location,
					result.content_type,
					result.flags,
				),
			));
		} else {
			Err(anyhow!("Package '{req}' does not exist"))
		}
	}

	/// Get a package from the map if it exists, and query insert it otherwise
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
		let pkg = self
			.get(req, paths, client, o)
			.await
			.with_context(|| format!("Failed to get package {req}"))?;
		pkg.ensure_loaded(paths, false, client)
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

	/// Get the content type of a package
	pub async fn get_content_type<'a>(
		&'a mut self,
		req: &ArcPkgReq,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<PackageContentType> {
		let pkg = self.ensure_package_contents(req, paths, client, o).await?;
		Ok(pkg.content_type)
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
		Ok(contents.to_string())
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
	#[allow(clippy::too_many_arguments)]
	pub async fn eval<'a>(
		&mut self,
		req: &ArcPkgReq,
		paths: &'a Paths,
		routine: Routine,
		input: EvalInput<'a>,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<EvalData<'a>> {
		let plugins = self.plugins.clone();
		let pkg = self.ensure_package_contents(req, paths, client, o).await?;
		let eval = pkg.eval(paths, routine, input, client, plugins).await?;
		Ok(eval)
	}

	/// Get the content type of a package
	pub async fn content_type<'a>(
		&mut self,
		req: &ArcPkgReq,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<PackageContentType> {
		let pkg = self.ensure_package_contents(req, paths, client, o).await?;
		Ok(pkg.content_type)
	}

	/// Get the flags of a package
	pub async fn flags<'a>(
		&'a mut self,
		req: &ArcPkgReq,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<&'a HashSet<PackageFlag>> {
		let pkg = self.ensure_package_contents(req, paths, client, o).await?;
		Ok(&pkg.flags)
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

	/// Iterator over all package requests in the registry
	pub fn iter_requests(&self) -> impl Iterator<Item = &ArcPkgReq> {
		self.packages.keys()
	}

	/// Get all of the package requests in the registry in an owned manner
	pub fn get_all_packages(&self) -> Vec<ArcPkgReq> {
		self.iter_requests().cloned().collect()
	}

	/// Get all of the available package requests from the repos
	pub async fn get_all_available_packages(
		&mut self,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Vec<ArcPkgReq>> {
		let out = super::repo::get_all_packages(&mut self.repos, paths, client, o)
			.await
			.context("Failed to retrieve all packages from repos")?
			.iter()
			.map(|(id, ..)| Arc::new(PkgRequest::any(id.as_ref(), PkgRequestSource::Repository)))
			.collect();

		Ok(out)
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
		let packages = self
			.get_all_available_packages(paths, client, o)
			.await
			.context("Failed to get list of available packages")?;

		self.remove_cached_packages(packages.iter(), paths, client, o)
			.await
			.context("Failed to remove all cached packages")?;

		// Redownload all the packages
		let mut tasks = JoinSet::new();
		let semaphore = Arc::new(Semaphore::new(download::get_transfer_limit()));
		for package in packages {
			let pkg = self
				.get(&package, paths, client, o)
				.await
				.with_context(|| format!("Failed to get package {package}"))?;

			if let Some(task) = pkg.get_download_task(paths, true, client) {
				let semaphore = semaphore.clone();
				let task = async move {
					let _ = semaphore.acquire_owned().await;
					task.await
				};
				tasks.spawn(task);
			}
		}

		while let Some(res) = tasks.join_next().await {
			res??;
		}

		Ok(())
	}

	/// Gets the repositories stored in this registry in their correct order
	pub fn get_repos(&self) -> &[PackageRepository] {
		&self.repos
	}
}
