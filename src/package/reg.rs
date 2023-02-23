use super::{Package, PkgKind, PkgError};
use super::repo::{PkgRepo, query_all, RepoError};
use crate::{util::versions::VersionPattern, io::files::paths::Paths};

use std::collections::HashMap;
use std::fmt::Display;
use std::path::Path;

// Used to store a request for a package that will be fulfilled later
#[derive(Debug)]
pub struct PkgRequest {
	pub name: String,
	pub version: VersionPattern
}

impl Display for PkgRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}@{}", self.name, self.version.as_string())
	}
}

// A known identifier for a package
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct PkgIdentifier {
	pub name: String,
	pub version: String
}

#[derive(Debug, thiserror::Error)]
pub enum RegError {
	#[error("Repository operation failed:\n{}", .0)]
	Repo(#[from] RepoError),
	#[error("Package {} not found", .0)]
	NotFound(String),
	#[error("Error in package:\n{}", .0)]
	Package(#[from] PkgError)
}

#[derive(Debug)]
pub struct PkgRegistry {
	pub repos: Vec<PkgRepo>,
	versions: HashMap<String, Vec<String>>,
	packages: HashMap<PkgIdentifier, Package>
}

impl PkgRegistry {
	pub fn new(repos: Vec<PkgRepo>) -> Self {
		Self {
			repos,
			versions: HashMap::new(),
			packages: HashMap::new()
		}
	}

	fn insert(&mut self, id: &PkgIdentifier, pkg: Package) -> &mut Package {
		let versions = self.versions.entry(id.name.clone()).or_insert(Vec::new());
		versions.push(id.version.clone());
		self.packages.insert(id.clone(), pkg);
		self.packages.get_mut(id).expect("Package was not inserted into map")
	}

	fn query_insert(&mut self, req: &PkgRequest, paths: &Paths) -> Result<&mut Package, RegError> {
		let pkg_name = req.name.clone();
		let pkg_vers = req.version.clone();

		match query_all(&mut self.repos, &pkg_name, &pkg_vers, paths)? {
			Some((url, version)) => {
				let id = PkgIdentifier {name: pkg_name.clone(), version: version.clone()};
				Ok(self.insert(&id, Package::new(&pkg_name, &version, PkgKind::Remote(Some(url)))))
			}
			None => Err(RegError::NotFound(pkg_name))
		}
	}

	fn get(&mut self, req: &PkgRequest, paths: &Paths) -> Result<&mut Package, RegError> {
		let pkg_name = req.name.clone();
		let pkg_vers = req.version.clone();
		match self.versions.get(&pkg_name) {
			Some(versions) => match pkg_vers.matches(versions) {
				Some(vers) => {
					let key = PkgIdentifier {name: pkg_name.clone(), version: vers};
					if self.packages.contains_key(&key) {
						Ok(self.packages.get_mut(&key).unwrap())
					} else {
						self.query_insert(req, paths)
					}
				}
				None => self.query_insert(req, paths)
			}
			None => self.query_insert(req, paths)
		}
	}

	// Load a package
	pub fn load(&mut self, req: &PkgRequest, paths: &Paths) -> Result<String, RegError> {
		let pkg = self.get(req, paths)?;
		pkg.ensure_loaded(paths)?;
		let contents = pkg.data.as_ref().expect("Package data was not loaded").get_contents();
		Ok(contents)
	}

	// Insert a local package into the registry
	pub fn insert_local(&mut self, id: &PkgIdentifier, path: &Path) {
		self.insert(id, Package::new(&id.name, &id.version, PkgKind::Local(path.to_path_buf())));
	}
}
