use super::eval::eval::{EvalConstants, Routine, EvalData};
use super::{Package, PkgKind, PkgError};
use super::repo::{PkgRepo, query_all, RepoError};
use crate::{util::versions::VersionPattern, io::files::paths::Paths};

use std::collections::HashMap;
use std::fmt::Display;
use std::path::Path;

// Used to store a request for a package that will be fulfilled later
#[derive(Debug, Clone, PartialEq)]
pub struct PkgRequest {
	pub name: String,
	pub version: VersionPattern
}

impl PkgRequest {
	pub fn new(name: &str, version: &VersionPattern) -> Self {
		Self {
			name: name.to_owned(),
			version: version.clone()
		}
	}
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

impl PkgIdentifier {
	pub fn new(name: &str, version: &str) -> Self {
		Self {
			name: name.to_owned(),
			version: version.to_owned()
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum RegError {
	#[error("Repository operation failed:\n{}", .0)]
	Repo(#[from] RepoError),
	#[error("Package '{}' with version '{}' not found", .0, .1)]
	NotFound(String, String),
	#[error("Error in package:\n{}", .0)]
	Package(#[from] PkgError),
	#[error("Package '{}' is incompatible with existing package '{}'", .0, .1)]
	Incompatible(PkgRequest, PkgRequest)
}

#[derive(Debug)]
pub struct PkgRegistry {
	pub repos: Vec<PkgRepo>,
	versions: HashMap<String, Vec<String>>,
	packages: HashMap<PkgIdentifier, Package>,
	requests: HashMap<String, Vec<PkgRequest>>
}

impl PkgRegistry {
	pub fn new(repos: Vec<PkgRepo>) -> Self {
		Self {
			repos,
			versions: HashMap::new(),
			packages: HashMap::new(),
			requests: HashMap::new()
		}
	}

	fn insert(&mut self, id: &PkgIdentifier, req: &PkgRequest, profile: &str, pkg: Package) -> &mut Package {
		let versions = self.versions.entry(id.name.clone()).or_insert(Vec::new());
		versions.push(id.version.clone());
		let requests = self.requests.entry(profile.to_owned()).or_insert(Vec::new());
		requests.push(req.clone());
		self.packages.insert(id.clone(), pkg);
		self.packages.get_mut(id).expect("Package was not inserted into map")
	}

	fn query_insert(&mut self, req: &PkgRequest, profile: &str, paths: &Paths) -> Result<&mut Package, RegError> {
		let pkg_name = req.name.clone();
		let pkg_vers = req.version.clone();

		match query_all(&mut self.repos, &pkg_name, &pkg_vers, paths)? {
			Some((url, version)) => {
				let id = PkgIdentifier::new(&pkg_name, &version);
				Ok(self.insert(&id, &req, profile, Package::new(&pkg_name, &version, PkgKind::Remote(Some(url)))))
			}
			None => Err(RegError::NotFound(pkg_name, pkg_vers.as_string().to_owned()))
		}
	}

	fn get(&mut self, req: &PkgRequest, profile: &str, paths: &Paths) -> Result<&mut Package, RegError> {
		let pkg_name = req.name.clone();
		let pkg_vers = req.version.clone();
		match self.versions.get(&pkg_name) {
			Some(versions) => match pkg_vers.get_match(versions) {
				Some(vers) => {
					let key = PkgIdentifier::new(&pkg_name, &vers);
					if self.packages.contains_key(&key) {
						Ok(self.packages.get_mut(&key).unwrap())
					} else {
						self.query_insert(req, profile, paths)
					}
				}
				None => self.query_insert(req, profile, paths)
			}
			None => self.query_insert(req, profile, paths)
		}
	}

	/// Checks if a package version already exists for a profile and is compatible.
	/// Will also update the current version to narrow it to new requirements
	pub fn update(&mut self, req: &PkgRequest, profile: &str, paths: &Paths)
	-> Result<&mut Package, RegError> {
		let pkg_name = req.name.clone();
		let pkg_vers = req.version.clone();
		let mut update = None;
		match self.requests.get_mut(profile) {
			Some(requests) => match requests.iter_mut().find(|x| x.name == req.name) {
				Some(current_req) => {
					let versions = self.versions.get(&pkg_name).expect("Versions do not exist");
					let combo = current_req.version.match_union(&pkg_vers, versions);
					if combo.is_empty() {
						return Err(RegError::Incompatible(req.clone(), current_req.clone()));
					} else {
						if combo.len() <= versions.len() {
							let version = combo.last().cloned().expect("Latest version not available in update");
							*current_req = req.clone();
							update = Some(version);
						}
					}
				}
				None => { let _ = self.query_insert(req, profile, paths)?; }
			}
			None => { let _ = self.query_insert(req, profile, paths)?; }
		};

		if let Some(version) = update {
			let current = self.get(req, profile, paths)?;
			let updated = Package::new(&pkg_name, &version, current.kind.clone());
			*current = updated;
			Ok(current)
		} else {
			self.get(req, profile, paths)
		}
	}

	// Load a package
	pub fn load(&mut self, req: &PkgRequest, profile: &str, paths: &Paths) -> Result<String, RegError> {
		let pkg = self.update(req, profile, paths)?;
		pkg.ensure_loaded(paths)?;
		let contents = pkg.data.as_ref().expect("Package data was not loaded").get_contents();
		Ok(contents)
	}

	// Parse a package
	pub fn parse(&mut self, req: &PkgRequest, profile: &str, paths: &Paths) -> Result<(), RegError> {
		let pkg = self.update(req, profile, paths)?;
		pkg.parse(paths)?;
		Ok(())
	}

	// Evaluate a package
	pub async fn eval(&mut self, req: &PkgRequest, profile: &str, paths: &Paths, routine: Routine, constants: EvalConstants)
	-> Result<EvalData, RegError> {
		let pkg = self.update(req, profile, paths)?;
		return Ok(pkg.eval(paths, routine, constants).await?);
	}

	// Insert a local package into the registry
	pub fn insert_local(&mut self, id: &PkgIdentifier, profile: &str, path: &Path) {
		let req = PkgRequest::new(&id.name, &VersionPattern::Single(id.version.clone()));
		self.insert(id, &req, profile, Package::new(&id.name, &id.version, PkgKind::Local(path.to_path_buf())));
	}

	// Checks if a package is in the registry already
	pub fn has_now(&self, req: &PkgRequest) -> bool {
		if let Some(versions) = self.versions.get(&req.name) {
			req.version.get_match(versions).is_some()
		} else {
			false
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::PathBuf;

	#[test]
	fn test_reg_insert() {
		let mut reg = PkgRegistry::new(vec![]);
		reg.insert_local(&PkgIdentifier::new("test", "1.1"), "profile", &PathBuf::from("./test"));
		let req = PkgRequest::new("test", &VersionPattern::Single(String::from("1.1")));
		assert!(reg.has_now(&req));
		assert!(!reg.has_now(&PkgRequest::new("doesnotexist", &VersionPattern::Single(String::from("foo")))));
		assert!(reg.requests.get("profile").unwrap().contains(&req));
	}
}
