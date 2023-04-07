use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::Context;
use color_print::cprintln;

use crate::data::instance::Side;
use crate::io::files::paths::Paths;
use crate::io::java::{Java, JavaKind};
use crate::io::options::{read_options, Options};
use crate::net::minecraft::{
	get_assets, get_game_jar, get_libraries, get_version_json, get_version_manifest,
	make_version_list,
};
use crate::util::versions::MinecraftVersion;
use crate::util::{json, print::PrintOptions};

/// Requirements for operations that may be shared by multiple instances in a profile
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum UpdateRequirement {
	VersionJson,
	GameAssets,
	GameLibraries,
	Java(JavaKind),
	GameJar(Side),
	Options,
}

/// Manager for when we are updating profile files.
/// It will keep track of files we have already downloaded, manage task requirements, etc
#[derive(Debug)]
pub struct UpdateManager {
	pub print: PrintOptions,
	pub force: bool,
	requirements: HashSet<UpdateRequirement>,
	// File paths that are added when they have been updated by other functions
	files: HashSet<PathBuf>,
	version_manifest: Option<Box<json::JsonObject>>,
	pub version_json: Option<Box<json::JsonObject>>,
	pub java: Option<Java>,
	pub options: Option<Options>,
	pub version_list: Option<Vec<String>>,
	pub found_version: Option<String>,
}

impl UpdateManager {
	pub fn new(print: PrintOptions, force: bool) -> Self {
		Self {
			print,
			force,
			requirements: HashSet::new(),
			files: HashSet::new(),
			version_manifest: None,
			version_json: None,
			java: None,
			options: None,
			version_list: None,
			found_version: None,
		}
	}

	/// Add a single requirement
	pub fn add_requirement(&mut self, req: UpdateRequirement) {
		self.requirements.insert(req);
	}

	/// Add multiple requirements
	pub fn add_requirements(&mut self, reqs: HashSet<UpdateRequirement>) {
		self.requirements.extend(reqs);
	}

	/// Check if a requirement is held
	pub fn has_requirement(&self, req: UpdateRequirement) -> bool {
		self.requirements.contains(&req)
	}

	/// Add tracked files to the manager
	pub fn add_files(&mut self, files: HashSet<PathBuf>) {
		self.files.extend(files);
	}

	/// Whether a file needs to be updated
	pub fn should_update_file(&self, file: &Path) -> bool {
		if self.force {
			!self.files.contains(file) || !file.exists()
		} else {
			!file.exists()
		}
	}

	/// Get the version manifest and fulfill the found version and version list fields.
	/// Must be called before fulfill_requirements.
	pub async fn fulfill_version_manifest(
		&mut self,
		paths: &Paths,
		version: &MinecraftVersion,
	) -> anyhow::Result<()> {
		if self.print.verbose {
			cprintln!("<s>Obtaining version index...");
		}
		let manifest = get_version_manifest(paths)
			.await
			.context("Failed to get version manifest")?;

		
		self.version_list =
			Some(make_version_list(&manifest).context("Failed to compose a list of versions")?);
		
		let found_version = version.get_version(&manifest)
			.context("Failed to find the requested Minecraft version")?;
	
		self.found_version = Some(found_version);
		self.version_manifest = Some(manifest);

		Ok(())
	}

	/// Run all of the operations that are part of the requirements.
	pub async fn fulfill_requirements(
		&mut self,
		paths: &Paths,
	) -> anyhow::Result<()> {
		let java_required = matches!(
			self.requirements
				.iter()
				.find(|x| matches!(x, UpdateRequirement::Java(..))),
			Some(..)
		);

		let game_jar_required = matches!(
			self.requirements
				.iter()
				.find(|x| matches!(x, UpdateRequirement::GameJar(..))),
			Some(..)
		);

		if java_required {
			self.add_requirement(UpdateRequirement::VersionJson);
		}

		if java_required
			|| game_jar_required
			|| self.has_requirement(UpdateRequirement::GameAssets)
			|| self.has_requirement(UpdateRequirement::GameLibraries)
		{
			self.add_requirement(UpdateRequirement::VersionJson);
		}

		if self.has_requirement(UpdateRequirement::VersionJson) {
			if self.print.verbose {
				cprintln!("<s>Obtaining version json...");
			}
			let version_json = get_version_json(
				self.found_version.as_ref().expect("Found version missing"),
				self.version_manifest.as_ref().expect("Version manifest missing"),
				paths,
			).await.context("Failed to get version json")?;
			self.version_json = Some(version_json);
		}

		if self.has_requirement(UpdateRequirement::GameAssets) {
			let version_json = self.version_json.as_ref().expect("Version json missing");
			let files = get_assets(version_json, paths, self.found_version.as_ref().expect("Found version missing"), self)
				.await
				.context("Failed to get game assets")?;
			self.add_files(files);
		}

		if self.has_requirement(UpdateRequirement::GameLibraries) {
			let version_json = self.version_json.as_ref().expect("Version json missing");
			let files = get_libraries(version_json, paths, self.found_version.as_ref().expect("Found version missing"), self)
				.await
				.context("Failed to get game libraries")?;
			self.add_files(files);
		}

		if java_required {
			let version_json = self.version_json.as_ref().expect("Version json missing");
			let java_vers = json::access_i64(
				json::access_object(version_json, "javaVersion")?,
				"majorVersion",
			)?;

			let mut java_files = HashSet::new();
			for req in self.requirements.iter() {
				if let UpdateRequirement::Java(kind) = req {
					let mut java = Java::new(kind.clone());
					java.add_version(&java_vers.to_string());
					let files = java
						.install(paths, self)
						.await
						.context("Failed to install Java")?;
					java_files.extend(files);
					self.java = Some(java);
				}
			}

			self.add_files(java_files);
		}

		if game_jar_required {
			let version_json = self.version_json.as_ref().expect("Version json missing");
			for req in self.requirements.iter() {
				if let UpdateRequirement::GameJar(side) = req {
					get_game_jar(side.clone(), version_json, self.found_version.as_ref().expect("Found version missing"), paths, self)
						.await
						.context("Failed to get the game JAR file")?;
				}
			}
		}

		if self.has_requirement(UpdateRequirement::Options) {
			let options = read_options(paths)
				.await
				.context("Failed to read options.json")?;
			self.options = options;
		}

		Ok(())
	}
}
