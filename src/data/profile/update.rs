use std::{collections::HashSet, path::{PathBuf, Path}};

use color_print::cprintln;

use crate::{io::{java::{JavaKind, Java}, files::paths::Paths}, util::json, data::instance::create::CreateError, net::minecraft::{get_version_manifest, get_version_json, make_version_list, get_assets}};

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {

}

// Requirements for operations that may be shared by multiple instances in a profile
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum UpdateRequirement {
	VersionJson,
	GameAssets,
	Java(JavaKind)
}

/// Manager for when we are updating profile files.
/// It will keep track of files we have already downloaded, manage task requirements, etc
#[derive(Debug)]
pub struct UpdateManager {
	pub verbose: bool,
	pub force: bool,
	requirements: HashSet<UpdateRequirement>,
	// File paths that are added when they have been updated by other functions
	files: HashSet<PathBuf>,
	pub version_json: Option<Box<json::JsonObject>>
}

impl UpdateManager {
	pub fn new(verbose: bool, force: bool) -> Self {
		Self {
			verbose,
			force,
			requirements: HashSet::new(),
			files: HashSet::new(),
			version_json: None,
		}
	}

	pub fn add_requirement(&mut self, req: UpdateRequirement) {
		self.requirements.insert(req);
	}

	pub fn add_requirements(&mut self, reqs: HashSet<UpdateRequirement>) {
		self.requirements.extend(reqs);
	}

	pub fn has_requirement(&self, req: UpdateRequirement) -> bool {
		self.requirements.contains(&req)
	}

	pub fn add_file(&mut self, file: &Path) {
		self.files.insert(file.to_owned());
	}

	pub fn add_files(&mut self, files: HashSet<PathBuf>) {
		self.files.extend(files);
	}

	/// Whether a file needs to be updated
	pub fn should_update_file(&self, file: &Path) -> bool {
		if self.force {
			!self.files.contains(file) || !file.exists()
		} else {
			file.exists()
		}
	}

	/// Run all of the operations that are part of the requirements.
	/// Returns the version list
	pub async fn fulfill_requirements(
		&mut self,
		paths: &Paths,
		version: &str,
	) -> Result<Vec<String>, CreateError> {
		let mut out = Vec::new();

		let java_required = matches!(
			self.requirements.iter().find(|x| matches!(x, UpdateRequirement::Java(..))),
			Some(..)
		);

		if java_required {
			self.add_requirement(UpdateRequirement::VersionJson);
		}

		if self.has_requirement(UpdateRequirement::GameAssets) {
			self.add_requirement(UpdateRequirement::VersionJson);
		}

		if self.has_requirement(UpdateRequirement::VersionJson) {
			if self.verbose {
				cprintln!("<s>Obtaining version index...");
			}
			let (manifest, ..) = get_version_manifest(paths)?;
			let (version_json, ..) = get_version_json(version, &manifest, paths)?;
			self.version_json = Some(version_json);
			out = make_version_list(&manifest)?;
		}

		if self.has_requirement(UpdateRequirement::GameAssets) {
			let version_json = self.version_json.as_ref().expect("Version json missing");
			get_assets(&version_json, paths, version, &self).await?;
		}

		if java_required {
			let version_json = self.version_json.as_ref().expect("Version json missing");
			let java_vers = json::access_i64(
				json::access_object(version_json, "javaVersion")?,
				"majorVersion",
			)?;

			for req in self.requirements.iter() {
				if let UpdateRequirement::Java(kind) = req {
					let mut java = Java::new(kind.clone());
					java.add_version(&java_vers.to_string());
					java.install(paths, self.verbose, self.force)?;
				}
			}
		}

		Ok(out)
	}
}
