use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use color_print::{cformat, cprintln};
use mcvm_shared::modifications::PluginLoader;

use crate::data::config::Config;
use crate::data::instance::InstKind;
use crate::io::files::paths::Paths;
use crate::io::java::{Java, JavaKind};
use crate::io::lock::Lockfile;
use crate::io::options::{read_options, Options};
use crate::io::Later;
use crate::net::fabric_quilt::{self, FabricQuiltMeta};
use crate::net::minecraft::{assets, game_jar, libraries, version_manifest};
use crate::net::paper;
use crate::package::eval::{EvalConstants, EvalPermissions};
use crate::package::reg::{PkgRequest, PkgRegistry};
use crate::package::eval::resolve::resolve;
use crate::util::print::{ReplPrinter, HYPHEN_POINT};
use crate::util::versions::MinecraftVersion;
use crate::util::{json, print::PrintOptions};
use mcvm_shared::instance::Side;

use super::{InstanceRegistry, Profile};

/// Requirements for operations that may be shared by multiple instances in a profile
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum UpdateRequirement {
	VersionJson,
	GameAssets,
	GameLibraries,
	Java(JavaKind),
	GameJar(Side),
	Options,
	FabricQuilt(fabric_quilt::Mode, Side),
}

/// Manager for when we are updating profile files.
/// It will keep track of files we have already downloaded, manage task requirements, etc
#[derive(Debug)]
pub struct UpdateManager {
	pub print: PrintOptions,
	pub force: bool,
	/// Whether we will prioritize local files instead of remote ones
	pub allow_offline: bool,
	requirements: HashSet<UpdateRequirement>,
	// File paths that are added when they have been updated by other functions
	files: HashSet<PathBuf>,
	version_manifest: Later<Box<json::JsonObject>>,
	pub version_json: Later<Box<json::JsonObject>>,
	pub java: Later<Java>,
	pub options: Option<Options>,
	pub version_list: Later<Vec<String>>,
	pub found_version: Later<String>,
	pub fq_meta: Later<FabricQuiltMeta>,
}

impl UpdateManager {
	pub fn new(print: PrintOptions, force: bool, allow_offline: bool) -> Self {
		Self {
			print,
			force,
			allow_offline,
			requirements: HashSet::new(),
			files: HashSet::new(),
			version_manifest: Later::new(),
			version_json: Later::new(),
			java: Later::new(),
			options: None,
			version_list: Later::new(),
			found_version: Later::new(),
			fq_meta: Later::new(),
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
		let manifest = version_manifest::get(paths, self)
			.await
			.context("Failed to get version manifest")?;

		self.version_list.fill(
			version_manifest::make_version_list(&manifest)
				.context("Failed to compose a list of versions")?,
		);

		let found_version = version
			.get_version(&manifest)
			.context("Failed to find the requested Minecraft version")?;

		self.found_version.fill(found_version);
		self.version_manifest.fill(manifest);

		Ok(())
	}

	/// Run all of the operations that are part of the requirements.
	pub async fn fulfill_requirements(&mut self, paths: &Paths) -> anyhow::Result<()> {
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

		let fq_required = matches!(
			self.requirements
				.iter()
				.find(|x| matches!(x, UpdateRequirement::FabricQuilt(..))),
			Some(..)
		);

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
			let version_json = version_manifest::get_version_json(
				self.found_version.get(),
				self.version_manifest.get(),
				paths,
				self,
			)
			.await
			.context("Failed to get version json")?;
			self.version_json.fill(version_json);
		}

		if self.has_requirement(UpdateRequirement::GameAssets) {
			let files = assets::get(
				self.version_json.get(),
				paths,
				self.found_version.get(),
				self,
			)
			.await
			.context("Failed to get game assets")?;
			self.add_files(files);
		}

		if self.has_requirement(UpdateRequirement::GameLibraries) {
			let version_json = self.version_json.get();
			let files = libraries::get(version_json, paths, self.found_version.get(), self)
				.await
				.context("Failed to get game libraries")?;
			self.add_files(files);
		}

		if java_required {
			let version_json = self.version_json.get();
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
					self.java.fill(java);
				}
			}

			self.add_files(java_files);
		}

		if game_jar_required {
			for req in self.requirements.iter() {
				if let UpdateRequirement::GameJar(side) = req {
					game_jar::get(
						*side,
						self.version_json.get(),
						self.found_version.get(),
						paths,
						self,
					)
					.await
					.context("Failed to get the game JAR file")?;
				}
			}
		}

		if fq_required {
			for req in self.requirements.iter() {
				if let UpdateRequirement::FabricQuilt(mode, side) = req {
					if self.fq_meta.is_empty() {
						let meta =
							fabric_quilt::get_meta(self.found_version.get(), mode, paths, self)
								.await
								.context("Failed to download Fabric/Quilt metadata")?;
						fabric_quilt::download_files(&meta, paths, *mode, self)
							.await
							.context("Failed to download common Fabric/Quilt files")?;
						self.fq_meta.fill(meta);
					}

					fabric_quilt::download_side_specific_files(
						self.fq_meta.get(),
						paths,
						*side,
						self,
					)
					.await
					.context("Failed to download {mode} files for {side}")?;
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

/// Install packages on a profile
async fn update_profile_packages(
	profile: &Profile,
	paths: &Paths,
	reg: &mut PkgRegistry,
	instances: &InstanceRegistry,
	constants: &EvalConstants,
	lock: &mut Lockfile,
) -> anyhow::Result<()> {
	let mut printer = ReplPrinter::new(true);
	let resolved = resolve(&profile.packages, constants, paths, reg)
		.await
		.context("Failed to resolve package dependencies")?;
	for pkg in &resolved {
		let pkg_version = reg
			.get_version(pkg, paths)
			.await
			.context("Failed to get version for package")?;
		for instance_id in &profile.instances {
			if let Some(instance) = instances.get(instance_id) {
				if let InstKind::Client { .. } = instance.kind {
					printer.print(&format_package_print(pkg, Some(instance_id), "Installing..."));
					instance
						.install_package(
							pkg,
							pkg_version,
							constants,
							reg,
							paths,
							lock,
						)
						.await
						.with_context(|| {
							format!("Failed to install package '{pkg}' for instance '{instance_id}'")
						})?;
				}
			}
		}
		printer.print(&format_package_print(pkg, None, &cformat!("<g>Installed.")));
		printer.newline();
	}
	for instance_id in profile.instances.iter() {
		if let Some(instance) = instances.get(instance_id) {
			let addons_to_remove = lock
				.remove_unused_packages(
					instance_id,
					&resolved
						.iter()
						.map(|x| x.name.clone())
						.collect::<Vec<String>>(),
				)
				.context("Failed to remove unused packages")?;
			for addon in addons_to_remove {
				instance.remove_addon(&addon, paths).with_context(|| {
					format!(
						"Failed to remove addon {} for instance {}",
						addon.id, instance_id
					)
				})?;
			}
		}
	}
	Ok(())
}

/// Creates the print message for package installation when updating profiles
fn format_package_print(pkg: &PkgRequest, instance: Option<&str>, message: &str) -> String {
	if let Some(instance) = instance {
		cformat!("{}[<c>{}</c>] (<b!>{}</b!>) {}", HYPHEN_POINT, pkg, instance, message)
	} else {
		cformat!("{}[<c>{}</c>] {}", HYPHEN_POINT, pkg, message)
	}
}

/// Update a list of profiles
pub async fn update_profiles(
	paths: &Paths,
	config: &mut Config,
	ids: &[String],
	force: bool,
) -> anyhow::Result<()> {
	for id in ids {
		if let Some(profile) = config.profiles.get_mut(id) {
			cprintln!("<s,g>Updating profile <b>{}</b>", id);

			let print_options = PrintOptions::new(true, 0);
			let mut manager = UpdateManager::new(print_options, force, false);
			manager
				.fulfill_version_manifest(paths, &profile.version)
				.await
				.context("Failed to get version information")?;
			let version = manager.found_version.get().clone();

			let (paper_build_num, paper_file_name) =
				if let PluginLoader::Paper = profile.plugin_loader {
					let (build_num, ..) = paper::get_newest_build(&version)
						.await
						.context("Failed to get the newest Paper build number")?;
					let paper_file_name = paper::get_jar_file_name(&version, build_num)
						.await
						.context("Failed to get the name of the Paper Jar file")?;
					(Some(build_num), Some(paper_file_name))
				} else {
					(None, None)
				};
			let mut lock = Lockfile::open(paths).context("Failed to open lockfile")?;
			if lock.update_profile_version(id, &version) {
				cprintln!("<s>Updating profile version...");
				for inst in profile.instances.iter() {
					if let Some(inst) = config.instances.get(inst) {
						inst.teardown(paths, paper_file_name.clone()).context(
							"Failed to remove old files when updating Minecraft version",
						)?;
					}
				}
			}
			if let Some(build_num) = paper_build_num {
				if let Some(file_name) = paper_file_name {
					if lock.update_profile_paper_build(id, build_num) {
						for inst in profile.instances.iter() {
							if let Some(inst) = config.instances.get(inst) {
								inst.remove_paper(paths, file_name.clone())
									.context("Failed to remove Paper")?;
							}
						}
					}
				}
			}

			lock.finish(paths)
				.await
				.context("Failed to finish using lockfile")?;

			if !profile.instances.is_empty() {
				let version_list = profile
					.create_instances(&mut config.instances, paths, manager)
					.await
					.context("Failed to create profile instances")?;

				if !profile.packages.is_empty() {
					cprintln!("<s>Updating packages");
					
					// Make sure all packages in the profile are in the registry first
					for pkg in &profile.packages {
						config.packages.ensure_package(&pkg.req, paths).await?;
					}
					// Resolve for both client and server
					let mut constants = EvalConstants {
						version: version.clone(),
						modloader: profile.modloader.clone(),
						plugin_loader: profile.plugin_loader.clone(),
						side: Side::Client,
						features: vec![],
						versions: version_list.clone(),
						perms: EvalPermissions::Standard,
					};
					update_profile_packages(profile, paths, &mut config.packages, &config.instances, &constants, &mut lock).await?;
					constants.side = Side::Server;
					update_profile_packages(profile, paths, &mut config.packages, &config.instances, &constants, &mut lock).await?;
					cprintln!("<g>All packages installed.");
				}
			}

			lock.finish(paths)
				.await
				.context("Failed to finish using lockfile")?;
		} else {
			bail!("Unknown profile '{id}'");
		}
	}

	Ok(())
}
