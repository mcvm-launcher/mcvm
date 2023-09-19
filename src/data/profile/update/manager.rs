use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::Context;
use mcvm_shared::later::Later;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::versions::VersionInfo;
use mcvm_shared::Side;
use reqwest::Client;

use crate::io::files::paths::Paths;
use crate::io::java::install::{JavaInstallation, JavaInstallationKind};
use crate::io::lock::Lockfile;
use crate::io::options::{read_options, Options};
use crate::net::fabric_quilt::{self, FabricQuiltMeta};
use crate::net::game_files::client_meta::{self, ClientMeta};
use crate::net::game_files::version_manifest::VersionManifest;
use crate::net::game_files::{assets, game_jar, libraries, log_config, version_manifest};
use crate::util::{print::PrintOptions, versions::MinecraftVersion};
use crate::RcType;

/// Requirements for operations that may be shared by multiple instances in a profile
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum UpdateRequirement {
	/// The client metadata file
	ClientMeta,
	/// Assets for the client
	ClientAssets,
	/// Libraries for the client
	ClientLibraries,
	/// A Java installation
	Java(JavaInstallationKind),
	/// The game JAR for a specific side
	GameJar(Side),
	/// Game options
	Options,
	/// Fabric and Quilt
	FabricQuilt(fabric_quilt::Mode, Side),
	/// Client logging configuration
	ClientLoggingConfig,
}

/// Manager for when we are updating profile files.
/// It will keep track of files we have already downloaded, manage task requirements, etc
#[derive(Debug)]
pub struct UpdateManager {
	/// Options for printing / output
	pub print: PrintOptions,
	/// Whether to force file updates
	pub force: bool,
	/// Whether we will prioritize local files instead of remote ones
	pub allow_offline: bool,
	requirements: HashSet<UpdateRequirement>,
	/// File paths that are added when they have been updated by other functions
	files: HashSet<PathBuf>,
	/// The version manifest to be fulfilled later
	version_manifest: Later<VersionManifest>,
	/// The client meta to be fulfilled later
	pub client_meta: Later<RcType<ClientMeta>>,
	/// The Java installation to be fulfilled later
	pub java: Later<JavaInstallation>,
	/// The game options to be fulfilled later
	pub options: Option<Options>,
	/// The version info to be fulfilled later
	pub version_info: Later<VersionInfo>,
	/// The Fabric/Quilt metadata to be fulfilled later
	pub fq_meta: Later<FabricQuiltMeta>,
}

impl UpdateManager {
	/// Create a new UpdateManager
	pub fn new(print: PrintOptions, force: bool, allow_offline: bool) -> Self {
		Self {
			print,
			force,
			allow_offline,
			requirements: HashSet::new(),
			files: HashSet::new(),
			version_manifest: Later::new(),
			client_meta: Later::new(),
			java: Later::new(),
			options: None,
			version_info: Later::Empty,
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

	/// Adds an UpdateMethodResult to the UpdateManager
	pub fn add_result(&mut self, result: UpdateMethodResult) {
		self.add_files(result.files_updated);
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
		version: &MinecraftVersion,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		o.start_process();
		o.display(
			MessageContents::StartProcess("Obtaining version index".into()),
			MessageLevel::Important,
		);

		let manifest = version_manifest::get(paths, self, client, o)
			.await
			.context("Failed to get version manifest")?;

		let version_list = version_manifest::make_version_list(&manifest)
			.context("Failed to compose a list of versions")?;

		let found_version = version
			.get_version(&manifest)
			.context("Failed to find the requested Minecraft version")?;

		self.version_info.fill(VersionInfo {
			version: found_version,
			versions: version_list,
		});
		self.version_manifest.fill(manifest);

		o.display(
			MessageContents::Success("Version index obtained".into()),
			MessageLevel::Important,
		);
		o.end_process();

		Ok(())
	}

	/// Run all of the operations that are part of the requirements.
	pub async fn fulfill_requirements(
		&mut self,
		paths: &Paths,
		lock: &mut Lockfile,
		client: &Client,
		o: &mut impl MCVMOutput,
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
			|| self.has_requirement(UpdateRequirement::ClientAssets)
			|| self.has_requirement(UpdateRequirement::ClientLibraries)
		{
			self.add_requirement(UpdateRequirement::ClientMeta);
		}

		if self.has_requirement(UpdateRequirement::ClientMeta) {
			o.start_process();
			o.display(
				MessageContents::StartProcess("Obtaining client metadata".into()),
				MessageLevel::Important,
			);

			let client_meta = client_meta::get(
				&self.version_info.get().version,
				self.version_manifest.get(),
				paths,
				self,
				client,
			)
			.await
			.context("Failed to get client meta")?;
			self.client_meta.fill(RcType::new(client_meta));

			o.display(
				MessageContents::Success("client meta obtained".into()),
				MessageLevel::Important,
			);
			o.end_process();
		}

		if self.has_requirement(UpdateRequirement::ClientAssets) {
			let result = assets::get(
				self.client_meta.get(),
				paths,
				self.version_info.get(),
				self,
				client,
				o,
			)
			.await
			.context("Failed to get game assets")?;
			self.add_result(result);
		}

		if self.has_requirement(UpdateRequirement::ClientLibraries) {
			let client_meta = self.client_meta.get();
			let result = libraries::get(
				client_meta,
				paths,
				&self.version_info.get().version,
				self,
				client,
				o,
			)
			.await
			.context("Failed to get game libraries")?;
			self.add_result(result);
		}

		if java_required {
			let client_meta = self.client_meta.get();
			let java_vers = client_meta.java_info.major_version;

			let mut java_result = UpdateMethodResult::new();
			for req in self.requirements.iter() {
				if let UpdateRequirement::Java(kind) = req {
					let mut java = JavaInstallation::new(kind.clone());
					java.add_version(&java_vers.0.to_string());
					let result = java
						.install(paths, self, lock, client, o)
						.await
						.context("Failed to install Java")?;
					java_result.merge(result);
					self.java.fill(java);
				}
			}
			lock.finish(paths).await?;
			self.add_result(java_result);
		}

		if game_jar_required {
			for req in self.requirements.iter() {
				if let UpdateRequirement::GameJar(side) = req {
					game_jar::get(
						*side,
						self.client_meta.get(),
						&self.version_info.get().version,
						paths,
						self,
						client,
						o,
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
						let meta = fabric_quilt::get_meta(
							&self.version_info.get().version,
							mode,
							paths,
							self,
							client,
						)
						.await
						.context("Failed to download Fabric/Quilt metadata")?;
						fabric_quilt::download_files(&meta, paths, *mode, self, client, o)
							.await
							.context("Failed to download common Fabric/Quilt files")?;
						self.fq_meta.fill(meta);
					}

					fabric_quilt::download_side_specific_files(
						self.fq_meta.get(),
						paths,
						*side,
						self,
						client,
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

		if self.has_requirement(UpdateRequirement::ClientLoggingConfig) {
			log_config::get(
				self.client_meta.get(),
				&self.version_info.get().version,
				paths,
				self,
				client,
			)
			.await
			.context("Failed to get client logging config")?;
		}

		Ok(())
	}
}

/// Struct returned by updating functions, with data like changed files
#[derive(Default)]
pub struct UpdateMethodResult {
	/// The files that this function has updated
	pub files_updated: HashSet<PathBuf>,
}

impl UpdateMethodResult {
	/// Create a new UpdateMethodResult
	pub fn new() -> Self {
		Self::default()
	}

	/// Merges this result with another one
	pub fn merge(&mut self, other: Self) {
		self.files_updated.extend(other.files_updated);
	}
}
