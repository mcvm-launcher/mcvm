#![warn(missing_docs)]

//! This library is used by MCVM to install and launch Minecraft. It aims to be the most powerful, fast,
//! and correct implementation available, without being bloated with extra features. Implementations
//! for installing certain modifications, like modloaders and alternative server runtimes, will be
//! provided in extension plugins

/// Configuration for library functionality
pub mod config;
/// Instances of versions that can be launched
pub mod instance;
/// Input / output with data formats and the system
pub mod io;
/// Code for launching the game
pub mod launch;
/// Networking interfaces
pub mod net;
/// Different types of users and authentication
pub mod user;
/// Common utilities
pub mod util;
/// Installable versions of the game
pub mod version;

use anyhow::Context;
pub use config::Configuration;
use io::{files::paths::Paths, persistent::PersistentData, update::UpdateManager};
use mcvm_shared::{later::Later, output::MCVMOutput, util::print::PrintOptions};
use net::game_files::version_manifest::{self, VersionManifest, VersionManifestAndList};
use user::UserManager;
use util::versions::MinecraftVersion;
use version::{InstalledVersion, LoadVersionParameters, VersionParameters, VersionRegistry};

/// Wrapper around all usage of `mcvm_core`
pub struct MCVMCore {
	config: Configuration,
	paths: Paths,
	req_client: reqwest::Client,
	persistent: PersistentData,
	update_manager: UpdateManager,
	versions: VersionRegistry,
	version_manifest: Later<VersionManifestAndList>,
	users: UserManager,
}

impl MCVMCore {
	/// Construct a new core with default settings
	pub fn new() -> anyhow::Result<Self> {
		Self::with_config(Configuration::new())
	}

	/// Construct a new core with set configuration
	pub fn with_config(config: Configuration) -> anyhow::Result<Self> {
		Self::with_config_and_paths(config, Paths::new().context("Failed to create core paths")?)
	}

	/// Construct a new core with set configuration and paths
	pub fn with_config_and_paths(config: Configuration, paths: Paths) -> anyhow::Result<Self> {
		let persistent =
			PersistentData::open(&paths).context("Failed to open persistent data file")?;
		let out = Self {
			paths,
			req_client: reqwest::Client::new(),
			persistent,
			update_manager: UpdateManager::new(PrintOptions::new(true, 0), false, false),
			versions: VersionRegistry::new(),
			version_manifest: Later::Empty,
			users: UserManager::new(config.ms_client_id.clone()),
			config,
		};
		Ok(out)
	}

	/// Get the configuration that the core uses
	pub fn get_config(&self) -> &Configuration {
		&self.config
	}

	/// Set the reqwest client to be used if you already have one
	pub fn set_client(&mut self, req_client: reqwest::Client) {
		self.req_client = req_client;
	}

	/// Get the reqwest client that the core uses
	pub fn get_client(&self) -> &reqwest::Client {
		&self.req_client
	}

	/// Get the paths that the core uses
	pub fn get_paths(&self) -> &Paths {
		&self.paths
	}

	/// Get the UserManager in order to add, remove, and auth users
	pub fn get_users(&mut self) -> &mut UserManager {
		&mut self.users
	}

	/// Get the version manifest
	pub async fn get_version_manifest(
		&mut self,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<&VersionManifest> {
		if self.version_manifest.is_empty() {
			let manifest = version_manifest::get_with_output(
				&self.paths,
				&self.update_manager,
				&self.req_client,
				o,
			)
			.await
			.context("Failed to get version manifest")?;

			let combo = VersionManifestAndList::new(manifest)?;

			self.version_manifest.fill(combo);
		}
		Ok(&self.version_manifest.get().manifest)
	}

	/// Load or install a version of the game
	pub async fn get_version(
		&mut self,
		version: &MinecraftVersion,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<InstalledVersion> {
		self.get_version_manifest(o)
			.await
			.context("Failed to ensure version manifest exists")?;
		let version = version
			.get_version(&self.version_manifest.get().manifest)
			.context("Version does not exist")?;
		let params = LoadVersionParameters {
			paths: &self.paths,
			req_client: &self.req_client,
			version_manifest: self.version_manifest.get(),
			update_manager: &self.update_manager,
		};
		let inner = self
			.versions
			.get_version(&version, params, o)
			.await
			.context("Failed to get or install version")?;
		let params = VersionParameters {
			paths: &self.paths,
			req_client: &self.req_client,
			persistent: &mut self.persistent,
			version_manifest: self.version_manifest.get(),
			update_manager: &mut self.update_manager,
			users: &mut self.users,
		};
		Ok(InstalledVersion { inner, params })
	}
}
