use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use mcvm_core::auth_crate::mc::ClientId;
use mcvm_core::config::BrandingProperties;
use mcvm_core::user::UserManager;
use mcvm_core::util::versions::MinecraftVersion;
use mcvm_core::version::InstalledVersion;
use mcvm_core::MCVMCore;
use mcvm_plugin::api::NoOp;
use mcvm_plugin::hooks::{AddVersions, HandleAuth, HandleAuthArg};
use mcvm_shared::later::Later;
use mcvm_shared::output::MCVMOutput;
use mcvm_shared::versions::VersionInfo;
use mcvm_shared::Side;
use reqwest::Client;

use crate::data::config::plugin::PluginManager;
use crate::io::files::paths::Paths;
use mcvm_mods::fabric_quilt::{self, FabricQuiltMeta};

/// Requirements for operations that may be shared by multiple instances in a profile
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum UpdateRequirement {
	/// Fabric and Quilt
	FabricQuilt(fabric_quilt::Mode, Side),
	/// Client logging configuration
	ClientLoggingConfig,
}

/// Settings for updating
#[derive(Debug)]
pub struct UpdateSettings {
	/// Whether to force file updates
	pub force: bool,
	/// Whether we will prioritize local files instead of remote ones
	pub allow_offline: bool,
	/// Whether to do offline authentication
	pub offline_auth: bool,
}

/// Manager for when we are updating profile files.
/// It will keep track of files we have already downloaded, manage task requirements, etc
pub struct UpdateManager {
	/// Settings for the update
	pub settings: UpdateSettings,
	/// Update requirements that are fulfilled
	requirements: HashSet<UpdateRequirement>,
	/// File paths that are added when they have been updated by other functions
	files: HashSet<PathBuf>,
	/// The Minecraft version of the manager
	mc_version: Later<MinecraftVersion>,
	/// The MS client id, if used
	ms_client_id: Option<ClientId>,
	/// The core to be fulfilled later
	pub core: Later<MCVMCore>,
	/// The version info to be fulfilled later
	pub version_info: Later<VersionInfo>,
	/// The Fabric/Quilt metadata to be fulfilled later
	pub fq_meta: Later<FabricQuiltMeta>,
}

impl UpdateManager {
	/// Create a new UpdateManager
	pub fn new(force: bool, allow_offline: bool) -> Self {
		let settings = UpdateSettings {
			force,
			allow_offline,
			offline_auth: false,
		};

		Self {
			settings,
			requirements: HashSet::new(),
			core: Later::Empty,
			ms_client_id: None,
			files: HashSet::new(),
			version_info: Later::Empty,
			fq_meta: Later::new(),
			mc_version: Later::Empty,
		}
	}

	/// Set offline authentication
	pub fn offline_auth(&mut self) {
		self.settings.offline_auth = true;
	}

	/// Set the MS client ID
	pub fn set_client_id(&mut self, id: ClientId) {
		self.ms_client_id = Some(id);
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
		if self.settings.force {
			!self.files.contains(file) || !file.exists()
		} else {
			!file.exists()
		}
	}

	/// Set the Minecraft version. Can be used with the same UpdateManager and will work fine.
	/// Just make sure to fulfill requirements again.
	pub fn set_version(&mut self, version: &MinecraftVersion) {
		self.mc_version.fill(version.clone());
		// We have to clear these now since they are out of date
		self.version_info.clear();
		self.fq_meta.clear();
	}

	/// Run all of the operations that are part of the requirements.
	pub async fn fulfill_requirements(
		&mut self,
		users: &UserManager,
		plugins: &PluginManager,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		// Setup the core
		self.setup_core(client, users, plugins, paths, o)
			.await
			.context("Failed to setup core")?;

		// If the Minecraft version is not set then we can just assume it is not being used
		if self.mc_version.is_empty() {
			return Ok(());
		}

		let version = self
			.get_core_version(o)
			.await
			.context("Failed to get version")?;
		let version_info = version.get_version_info();

		self.update_fabric_quilt(&version_info, paths, client, o)
			.await
			.context("Failed to update Fabric/Quilt")?;

		self.version_info.fill(version_info);

		Ok(())
	}

	/// Sets up the core
	async fn setup_core(
		&mut self,
		client: &Client,
		users: &UserManager,
		plugins: &PluginManager,
		paths: &Paths,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		if self.core.is_full() {
			return Ok(());
		}

		// Setup the core
		let mut core_config = mcvm_core::ConfigBuilder::new()
			.allow_offline(self.settings.allow_offline)
			.force_reinstall(self.settings.force)
			.branding(BrandingProperties::new(
				"mcvm".into(),
				crate::VERSION.into(),
			));
		if let Some(client_id) = &self.ms_client_id {
			core_config = core_config.ms_client_id(client_id.clone());
		}
		let core_config = core_config.build();
		let mut core = MCVMCore::with_config(core_config).context("Failed to initialize core")?;

		// Set up user manager along with custom auth function that handles using plugins
		core.get_users().steal_users(users);
		core.get_users().set_offline(self.settings.offline_auth);
		{
			let plugins = plugins.clone();
			let paths = paths.clone();

			core.get_users()
				.set_custom_auth_function(Arc::new(move |user_id, user_type| {
					let arg = HandleAuthArg {
						user_id: user_id.to_string(),
						user_type: user_type.to_string(),
					};
					let results = plugins
						.call_hook(HandleAuth, &arg, &paths, &mut NoOp)
						.context("Failed to call handle auth hook")?;
					for result in results {
						let result = result.result(&mut NoOp)?;
						if result.handled {
							return Ok(result.profile);
						}
					}

					Ok(None)
				}));
		}

		core.set_client(client.clone());

		// Add extra versions to manifest from plugins
		let results = plugins
			.call_hook(AddVersions, &(), paths, o)
			.context("Failed to call add_versions hook")?;
		for result in results {
			let result = result.result(o)?;
			core.add_additional_versions(result);
		}

		self.core.fill(core);

		Ok(())
	}

	/// Get the version from the core
	pub async fn get_core_version(
		&mut self,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<InstalledVersion> {
		let version = self
			.core
			.get_mut()
			.get_version(self.mc_version.get_mut(), o)
			.await
			.context("Failed to get core version")?;

		Ok(version)
	}

	/// Update Fabric or Quilt if it is required
	async fn update_fabric_quilt(
		&mut self,
		version_info: &VersionInfo,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		if self.fq_meta.is_full() {
			return Ok(());
		}

		let core = self.core.get();

		// Check if we need to update
		let required = matches!(
			self.requirements
				.iter()
				.find(|x| matches!(x, UpdateRequirement::FabricQuilt(..))),
			Some(..)
		);

		// Update Fabric / Quilt
		if required {
			for req in self.requirements.iter() {
				if let UpdateRequirement::FabricQuilt(mode, side) = req {
					if self.fq_meta.is_empty() {
						let meta = fabric_quilt::get_meta(
							&version_info.version,
							mode,
							&paths.core,
							core.get_update_manager(),
							client,
						)
						.await
						.context("Failed to download Fabric/Quilt metadata")?;
						fabric_quilt::download_files(
							&meta,
							&paths.core,
							*mode,
							core.get_update_manager(),
							client,
							o,
						)
						.await
						.context("Failed to download common Fabric/Quilt files")?;
						self.fq_meta.fill(meta);
					}

					fabric_quilt::download_side_specific_files(
						self.fq_meta.get(),
						&paths.core,
						*side,
						core.get_update_manager(),
						client,
					)
					.await
					.context("Failed to download {mode} files for {side}")?;
				}
			}
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

	/// Create a new UpdateMethodResult from one path
	pub fn from_path(path: PathBuf) -> Self {
		let mut out = Self::new();
		out.files_updated.insert(path);
		out
	}

	/// Merges this result with another one
	pub fn merge(&mut self, other: Self) {
		self.files_updated.extend(other.files_updated);
	}
}
