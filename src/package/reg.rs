use anyhow::{bail, Context};
use mcvm_parse::metadata::PackageMetadata;

use super::eval::{EvalConstants, EvalData, Routine};
use super::repo::{query_all, PkgRepo};
use super::{Package, PkgKind};
use crate::io::files::paths::Paths;

use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::path::Path;

/// Where a package was requested from
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PkgRequestSource {
	UserRequire,
	Dependency(Box<PkgRequest>),
}

impl PkgRequestSource {
	/// Gets the source package of this package, if any
	pub fn get_source<'a>(&'a self) -> Option<&'a PkgRequest> {
		match self {
			Self::Dependency(source) => Some(&source),
			_ => None,
		}
	}
}

/// Used to store a request for a package that will be fulfilled later
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PkgRequest {
	pub name: String,
	pub source: PkgRequestSource,
}

impl PkgRequest {
	pub fn new(name: &str, source: PkgRequestSource) -> Self {
		Self {
			name: name.to_owned(),
			source,
		}
	}

	/// Checks if two PkgRequests request the same package
	pub fn same_as(&self, other: &Self) -> bool {
		self.name == other.name
	}
}

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

#[derive(Debug)]
pub struct PkgRegistry {
	pub repos: Vec<PkgRepo>,
	packages: HashMap<PkgRequest, Package>,
}

impl PkgRegistry {
	pub fn new(repos: Vec<PkgRepo>) -> Self {
		Self {
			repos,
			packages: HashMap::new(),
		}
	}

	fn insert(&mut self, req: &PkgRequest, pkg: Package) -> &mut Package {
		self.packages.insert(req.clone(), pkg);
		self.packages
			.get_mut(req)
			.expect("Package was not inserted into map")
	}

	async fn query_insert(
		&mut self,
		req: &PkgRequest,
		paths: &Paths,
	) -> anyhow::Result<&mut Package> {
		let pkg_name = req.name.clone();

		match query_all(&mut self.repos, &pkg_name, paths).await? {
			Some((url, version)) => Ok(self.insert(
				req,
				Package::new(&pkg_name, version, PkgKind::Remote(Some(url))),
			)),
			None => bail!("Package {pkg_name} was not found"),
		}
	}

	async fn get(&mut self, req: &PkgRequest, paths: &Paths) -> anyhow::Result<&mut Package> {
		if self.packages.contains_key(req) {
			Ok(self.packages.get_mut(req).expect("Package does not exist"))
		} else {
			self.query_insert(req, paths).await
		}
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
	) -> anyhow::Result<&'a PackageMetadata> {
		let pkg = self.get(req, paths).await?;
		Ok(pkg
			.get_metadata(paths)
			.await
			.context("Failed to get metadata from package")?)
	}

	/// Load a package
	pub async fn load(
		&mut self,
		req: &PkgRequest,
		force: bool,
		paths: &Paths,
	) -> anyhow::Result<String> {
		let pkg = self.get(req, paths).await?;
		pkg.ensure_loaded(paths, force).await?;
		let contents = pkg.data.get().get_contents();
		Ok(contents)
	}

	/// Evaluate a package
	pub async fn eval(
		&mut self,
		req: &PkgRequest,
		paths: &Paths,
		routine: Routine,
		constants: &EvalConstants,
	) -> anyhow::Result<EvalData> {
		let pkg = self.get(req, paths).await?;
		let eval = pkg.eval(paths, routine, constants).await?;
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

	/// Checks if a package is in the registry already
	#[cfg(test)]
	pub fn has_now(&self, req: &PkgRequest) -> bool {
		self.packages.contains_key(req)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::PathBuf;

	#[test]
	fn test_reg_insert() {
		let mut reg = PkgRegistry::new(vec![]);
		reg.insert_local(
			&PkgRequest::new("test", PkgRequestSource::UserRequire),
			1,
			&PathBuf::from("./test"),
		);
		let req = PkgRequest::new("test", PkgRequestSource::UserRequire);
		assert!(reg.has_now(&req));
		assert!(!reg.has_now(&PkgRequest::new(
			"doesnotexist",
			PkgRequestSource::UserRequire
		)));
	}
}
