use super::eval::eval::{EvalConstants, EvalData, Routine};
use super::repo::{query_all, PkgRepo, RepoError};
use super::{Package, PkgError, PkgKind};
use crate::io::files::paths::Paths;
use crate::io::lock::LockfileError;

use std::collections::HashMap;
use std::fmt::Display;
use std::path::Path;

// Used to store a request for a package that will be fulfilled later
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PkgRequest {
	pub name: String,
}

impl PkgRequest {
	pub fn new(name: &str) -> Self {
		Self {
			name: name.to_owned(),
		}
	}
}

impl Display for PkgRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.name)
	}
}

// A known identifier for a package
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct PkgIdentifier {
	pub name: String,
	pub version: String,
}

impl PkgIdentifier {
	pub fn new(name: &str, version: &str) -> Self {
		Self {
			name: name.to_owned(),
			version: version.to_owned(),
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum RegError {
	#[error("Repository operation failed:\n{}", .0)]
	Repo(#[from] RepoError),
	#[error("Package '{}' not found", .0)]
	NotFound(String),
	#[error("Error in package:\n{}", .0)]
	Package(#[from] PkgError),
	#[error("Failed to access lockfile:\n{}", .0)]
	Lock(#[from] LockfileError),
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

	fn query_insert(&mut self, req: &PkgRequest, paths: &Paths) -> Result<&mut Package, RegError> {
		let pkg_name = req.name.clone();

		match query_all(&mut self.repos, &pkg_name, paths)? {
			Some((url, version)) => Ok(self.insert(
				req,
				Package::new(&pkg_name, &version, PkgKind::Remote(Some(url))),
			)),
			None => Err(RegError::NotFound(pkg_name)),
		}
	}

	fn get(&mut self, req: &PkgRequest, paths: &Paths) -> Result<&mut Package, RegError> {
		if self.packages.contains_key(req) {
			Ok(self.packages.get_mut(req).expect("Package does not exist"))
		} else {
			self.query_insert(req, paths)
		}
	}

	// Get the version of a package
	pub fn get_version(&mut self, req: &PkgRequest, paths: &Paths) -> Result<String, RegError> {
		let pkg = self.get(req, paths)?;
		Ok(pkg.id.version.clone())
	}

	// Load a package
	pub fn load(
		&mut self,
		req: &PkgRequest,
		force: bool,
		paths: &Paths,
	) -> Result<String, RegError> {
		let pkg = self.get(req, paths)?;
		pkg.ensure_loaded(paths, force)?;
		let contents = pkg
			.data
			.as_ref()
			.expect("Package data was not loaded")
			.get_contents();
		Ok(contents)
	}

	// Parse a package
	pub fn parse(&mut self, req: &PkgRequest, paths: &Paths) -> Result<(), RegError> {
		let pkg = self.get(req, paths)?;
		pkg.parse(paths)?;
		Ok(())
	}

	// Evaluate a package
	pub async fn eval(
		&mut self,
		req: &PkgRequest,
		paths: &Paths,
		routine: Routine,
		constants: EvalConstants,
	) -> Result<EvalData, RegError> {
		let pkg = self.get(req, paths)?;
		let eval = pkg.eval(paths, routine, constants).await?;
		Ok(eval)
	}

	// Remove a cached package
	pub fn remove_cached(&mut self, req: &PkgRequest, paths: &Paths) -> Result<(), RegError> {
		let pkg = self.get(req, paths)?;
		pkg.remove_cached(paths)?;
		Ok(())
	}

	// Insert a local package into the registry
	pub fn insert_local(&mut self, req: &PkgRequest, version: &str, path: &Path) {
		self.insert(
			req,
			Package::new(&req.name, version, PkgKind::Local(path.to_path_buf())),
		);
	}

	// Checks if a package is in the registry already
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
		reg.insert_local(&PkgRequest::new("test"), "1.1", &PathBuf::from("./test"));
		let req = PkgRequest::new("test");
		assert!(reg.has_now(&req));
		assert!(!reg.has_now(&PkgRequest::new("doesnotexist")));
	}
}
