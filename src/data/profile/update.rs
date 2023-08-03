use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use color_print::{cformat, cprintln};
use itertools::Itertools;
use mcvm_shared::modifications::ServerType;
use mcvm_shared::pkg::PackageStability;
use reqwest::Client;

use crate::data::config::Config;
use crate::io::files::paths::Paths;
use crate::io::java::{Java, JavaKind};
use crate::io::lock::Lockfile;
use crate::io::options::{read_options, Options};
use crate::io::Later;
use crate::net::fabric_quilt::{self, FabricQuiltMeta};
use crate::net::minecraft::{assets, game_jar, libraries, version_manifest};
use crate::net::paper;
use crate::package::eval::resolve::resolve;
use crate::package::eval::{EvalConstants, EvalInput, EvalParameters, EvalPermissions};
use crate::package::reg::{PkgRegistry, PkgRequest};
use crate::util::print::{ReplPrinter, HYPHEN_POINT};
use crate::util::select_random_n_items_from_list;
use crate::util::versions::MinecraftVersion;
use crate::util::{json, print::PrintOptions};
use mcvm_shared::instance::Side;

use super::{InstanceRegistry, Profile};

/// Requirements for operations that may be shared by multiple instances in a profile
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum UpdateRequirement {
	ClientJson,
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
	pub client_json: Later<Box<json::JsonObject>>,
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
			client_json: Later::new(),
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
	pub async fn fulfill_requirements(
		&mut self,
		paths: &Paths,
		lock: &mut Lockfile,
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
			self.add_requirement(UpdateRequirement::ClientJson);
		}

		if self.has_requirement(UpdateRequirement::ClientJson) {
			if self.print.verbose {
				cprintln!("<s>Obtaining client JSON data...");
			}
			let client_json = version_manifest::get_client_json(
				self.found_version.get(),
				self.version_manifest.get(),
				paths,
				self,
			)
			.await
			.context("Failed to get client JSON")?;
			self.client_json.fill(client_json);
		}

		if self.has_requirement(UpdateRequirement::GameAssets) {
			let files = assets::get(
				self.client_json.get(),
				paths,
				self.found_version.get(),
				self,
			)
			.await
			.context("Failed to get game assets")?;
			self.add_files(files);
		}

		if self.has_requirement(UpdateRequirement::GameLibraries) {
			let client_json = self.client_json.get();
			let files = libraries::get(client_json, paths, self.found_version.get(), self)
				.await
				.context("Failed to get game libraries")?;
			self.add_files(files);
		}

		if java_required {
			let client_json = self.client_json.get();
			let java_vers = json::access_i64(
				json::access_object(client_json, "javaVersion")?,
				"majorVersion",
			)?;

			let mut java_files = HashSet::new();
			for req in self.requirements.iter() {
				if let UpdateRequirement::Java(kind) = req {
					let mut java = Java::new(kind.clone());
					java.add_version(&java_vers.to_string());
					let files = java
						.install(paths, self, lock)
						.await
						.context("Failed to install Java")?;
					java_files.extend(files);
					self.java.fill(java);
				}
			}
			lock.finish(paths).await?;
			self.add_files(java_files);
		}

		if game_jar_required {
			for req in self.requirements.iter() {
				if let UpdateRequirement::GameJar(side) = req {
					game_jar::get(
						*side,
						self.client_json.get(),
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

/// Shared objects for profile updating functions
pub struct ProfileUpdateContext<'a> {
	pub packages: &'a mut PkgRegistry,
	pub instances: &'a mut InstanceRegistry,
	pub paths: &'a Paths,
	pub lock: &'a mut Lockfile,
	pub client: &'a Client,
}

/// Resolve packages and create a mapping of packages to a list of instances.
/// This allows us to update packages in a reasonable order to the user.
/// It also returns a map of instances to packages so that unused packages can be removed
async fn resolve_and_batch<'a>(
	profile: &Profile,
	constants: &EvalConstants,
	ctx: &mut ProfileUpdateContext<'a>,
) -> anyhow::Result<(
	HashMap<PkgRequest, Vec<String>>,
	HashMap<String, Vec<PkgRequest>>,
)> {
	let mut batched: HashMap<PkgRequest, Vec<String>> = HashMap::new();
	let mut resolved = HashMap::new();
	for instance_id in &profile.instances {
		let instance = ctx.instances.get(instance_id).ok_or(anyhow!(
			"Instance '{instance_id}' does not exist in the registry"
		))?;
		let params = EvalParameters {
			side: instance.kind.to_side(),
			features: Vec::new(),
			perms: EvalPermissions::Standard,
			stability: PackageStability::Stable,
		};
		let instance_resolved = resolve(&profile.packages, constants, params, ctx.paths, ctx.packages)
			.await
			.with_context(|| {
				format!("Failed to resolve package dependencies for instance '{instance_id}'")
			})?;
		for package in &instance_resolved {
			if let Some(entry) = batched.get_mut(package) {
				entry.push(instance_id.clone());
			} else {
				batched.insert(package.clone(), vec![instance_id.clone()]);
			}
		}
		resolved.insert(instance_id.clone(), instance_resolved);
	}

	Ok((batched, resolved))
}

/// Install packages on a profile. Returns a set of all unique packages
async fn update_profile_packages<'a>(
	profile: &Profile,
	constants: &EvalConstants,
	ctx: &mut ProfileUpdateContext<'a>,
	force: bool,
) -> anyhow::Result<HashSet<PkgRequest>> {
	let mut printer = ReplPrinter::new(true);
	cprintln!("<s>Collecting package dependencies...");
	let (batched, resolved) = resolve_and_batch(profile, constants, ctx)
		.await
		.context("Failed to resolve dependencies for profile")?;

	cprintln!("<s>Installing packages...");
	for (package, package_instances) in batched.iter().sorted_by_key(|x| x.0) {
		let pkg_version = ctx
			.packages
			.get_version(package, ctx.paths)
			.await
			.context("Failed to get version for package")?;
		let mut notices = Vec::new();
		for instance_id in package_instances {
			let instance = ctx.instances.get(instance_id).ok_or(anyhow!(
				"Instance '{instance_id}' does not exist in the registry"
			))?;
			let params = EvalParameters {
				side: instance.kind.to_side(),
				features: Vec::new(),
				perms: EvalPermissions::Standard,
				stability: PackageStability::Stable,
			};
			printer.print(&format_package_print(
				package,
				Some(instance_id),
				"Installing...",
			));
			let input = EvalInput { constants, params };
			let result = instance
				.install_package(package, pkg_version, input, ctx.packages, ctx.paths, ctx.lock, force, ctx.client)
				.await
				.with_context(|| {
					format!("Failed to install package '{package}' for instance '{instance_id}'")
				})?;
			notices.extend(
				result
					.notices
					.iter()
					.map(|x| (instance_id.clone(), x.to_owned())),
			);
		}
		printer.print(&format_package_print(
			package,
			None,
			&cformat!("<g>Installed."),
		));
		for (instance, notice) in notices {
			printer.print(&format_package_print(
				package,
				Some(&instance),
				&cformat!("<y>Notice: {}", notice),
			));
		}
		printer.newline();
	}
	for (instance_id, packages) in resolved {
		let instance = ctx.instances.get(&instance_id).ok_or(anyhow!(
			"Instance '{instance_id}' does not exist in the registry"
		))?;
		let files_to_remove = ctx
			.lock
			.remove_unused_packages(
				&instance_id,
				&packages
					.iter()
					.map(|x| x.name.clone())
					.collect::<Vec<String>>(),
			)
			.context("Failed to remove unused packages")?;
		for file in files_to_remove {
			instance.remove_addon_file(&file, ctx.paths).with_context(|| {
				format!(
					"Failed to remove addon file {} for instance {}",
					file.display(),
					instance_id
				)
			})?;
		}
	}

	let mut out = HashSet::new();
	out.extend(batched.keys().cloned());

	Ok(out)
}

/// Creates the print message for package installation when updating profiles
fn format_package_print(pkg: &PkgRequest, instance: Option<&str>, message: &str) -> String {
	if let Some(instance) = instance {
		cformat!(
			"{}[{}] (<b!>{}</b!>) {}",
			HYPHEN_POINT,
			pkg.disp_with_colors(),
			instance,
			message
		)
	} else {
		cformat!(
			"{}[<c>{}</c>] {}",
			HYPHEN_POINT,
			pkg.disp_with_colors(),
			message
		)
	}
}

/// Prints support messages about installed packages when updating
pub async fn print_package_support_messages(
	packages: &[PkgRequest],
	reg: &mut PkgRegistry,
	client: &Client,
	paths: &Paths,
) -> anyhow::Result<()> {
	let package_count = 5;
	let packages = select_random_n_items_from_list(packages, package_count);
	let mut links = Vec::new();
	for package in packages {
		if let Some(link) = reg
			.get_metadata(package, paths, client)
			.await?
			.support_link
			.clone()
		{
			links.push((package, link))
		}
	}
	if !links.is_empty() {
		cprintln!("<s>Packages to consider supporting:");
		for (req, link) in links {
			println!(
				"{}",
				&format_package_print(req, None, &cformat!("<m> {}", link),)
			);
		}
	}

	Ok(())
}

/// Update a profile when the Minecraft version has changed
async fn check_profile_version_change<'a>(
	profile: &Profile,
	mc_version: &str,
	paper_properties: Option<(u16, String)>,
	ctx: &mut ProfileUpdateContext<'a>,
) -> anyhow::Result<()> {
	if ctx.lock.update_profile_version(&profile.name, mc_version) {
		cprintln!("<s>Updating profile version...");
		for instance_id in profile.instances.iter() {
			let instance = ctx.instances.get(instance_id).ok_or(anyhow!(
				"Instance '{instance_id}' does not exist in the registry"
			))?;
			instance
				.teardown(ctx.paths, paper_properties.clone())
				.context("Failed to remove old files when updating Minecraft version")?;
		}
	}
	Ok(())
}

/// Get the updated Paper file name and build number for a profile that uses it
async fn get_paper_properties(
	profile: &Profile,
	mc_version: &str,
) -> anyhow::Result<Option<(u16, String)>> {
	let out = if let ServerType::Paper = profile.modifications.server_type {
		let (build_num, ..) = paper::get_newest_build(mc_version)
			.await
			.context("Failed to get the newest Paper build number")?;
		let paper_file_name = paper::get_jar_file_name(mc_version, build_num)
			.await
			.context("Failed to get the name of the Paper Jar file")?;
		Some((build_num, paper_file_name))
	} else {
		None
	};

	Ok(out)
}

/// Remove the old Paper files for a profile if they have updated
async fn check_profile_paper_update<'a>(
	profile: &Profile,
	paper_properties: Option<(u16, String)>,
	ctx: &mut ProfileUpdateContext<'a>,
) -> anyhow::Result<()> {
	if let Some((build_num, file_name)) = paper_properties {
		if ctx
			.lock
			.update_profile_paper_build(&profile.name, build_num)
		{
			for inst in profile.instances.iter() {
				if let Some(inst) = ctx.instances.get(inst) {
					inst.remove_paper(ctx.paths, file_name.clone())
						.context("Failed to remove Paper")?;
				}
			}
		}
	}

	Ok(())
}

/// Update a list of profiles
pub async fn update_profiles(
	paths: &Paths,
	config: &mut Config,
	ids: &[String],
	force: bool,
	update_packages: bool,
) -> anyhow::Result<()> {
	let mut all_packages = HashSet::new();
	let client = Client::new();
	let mut lock = Lockfile::open(paths).context("Failed to open lockfile")?;

	let mut ctx = ProfileUpdateContext {
		packages: &mut config.packages,
		instances: &mut config.instances,
		paths,
		lock: &mut lock,
		client: &client,
	};

	for id in ids {
		let profile = config
			.profiles
			.get_mut(id)
			.ok_or(anyhow!("Unknown profile '{id}'"))?;
		cprintln!("<s,g>Updating profile <b>{}</b>", id);

		let print_options = PrintOptions::new(true, 0);
		let mut manager = UpdateManager::new(print_options, force, false);
		manager
			.fulfill_version_manifest(paths, &profile.version)
			.await
			.context("Failed to get version information")?;
		let mc_version = manager.found_version.get().clone();

		let paper_properties = get_paper_properties(profile, &mc_version)
			.await
			.context("Failed to get Paper build number and filename")?;

		check_profile_version_change(profile, &mc_version, paper_properties.clone(), &mut ctx)
			.await
			.context("Failed to check for a profile version update")?;

		check_profile_paper_update(profile, paper_properties, &mut ctx)
			.await
			.context("Failed to check for Paper updates")?;

		ctx.lock
			.finish(paths)
			.await
			.context("Failed to finish using lockfile")?;

		if !update_packages {
			return Ok(());
		}

		if !profile.instances.is_empty() {
			let version_list = profile
				.create_instances(ctx.instances, paths, manager, ctx.lock)
				.await
				.context("Failed to create profile instances")?;

			if !profile.packages.is_empty() {
				cprintln!("<s>Updating packages");
			}

			// Make sure all packages in the profile are in the registry first
			for pkg in &profile.packages {
				ctx.packages.ensure_package(&pkg.req, paths).await?;
			}

			let constants = EvalConstants {
				version: mc_version.to_string(),
				modifications: profile.modifications.clone(),
				features: vec![],
				version_list: version_list.clone(),
				perms: EvalPermissions::Standard,
				language: config.prefs.language,
			};

			let packages = update_profile_packages(profile, &constants, &mut ctx, force).await?;
			cprintln!("<g>All packages installed.");
			all_packages.extend(packages);
		}

		ctx.lock
			.finish(paths)
			.await
			.context("Failed to finish using lockfile")?;
	}

	let all_packages = Vec::from_iter(all_packages);
	print_package_support_messages(&all_packages, &mut config.packages, &client, paths)
		.await
		.context("Failed to print support messages")?;

	Ok(())
}
