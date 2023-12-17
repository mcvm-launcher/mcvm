#![warn(missing_docs)]

//! This library is used by MCVM to install and launch Minecraft. It aims to be the most powerful, fast,
//! and correct implementation available, without being bloated with extra features. Implementations
//! for installing certain modifications, like modloaders and alternative server runtimes, will be
//! provided in extension plugins

pub use mcvm_auth as auth_crate;

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
use io::{persistent::PersistentData, update::UpdateManager};
use mcvm_shared::later::Later;
use mcvm_shared::output::{self, MCVMOutput};
use mcvm_shared::util::print::PrintOptions;
use mcvm_shared::versions::VersionInfo;
use net::game_files::version_manifest::{
	self, make_version_list, VersionManifest, VersionManifestAndList,
};
use user::UserManager;
use util::versions::MinecraftVersion;
use version::{InstalledVersion, LoadVersionParameters, VersionParameters, VersionRegistry};

pub use config::{ConfigBuilder, Configuration};
pub use instance::{ClientWindowConfig, Instance, InstanceConfiguration, InstanceKind};
pub use io::files::paths::Paths;
pub use launch::{InstanceHandle, QuickPlayType, WrapperCommand};

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
			update_manager: UpdateManager::new(
				PrintOptions::new(true, 0),
				config.force_reinstall,
				config.allow_offline,
			),
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

	/// Get the UpdateManager in order to help with custom installation
	/// routines
	pub fn get_update_manager(&self) -> &UpdateManager {
		&self.update_manager
	}

	/// Get the UpdateManager mutably in order to help with custom installation
	/// routines. Don't modify this unless you know what you are doing!
	pub fn get_update_manager_mut(&mut self) -> &mut UpdateManager {
		&mut self.update_manager
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
			censor_secrets: self.config.censor_secrets,
		};
		Ok(InstalledVersion { inner, params })
	}

	/// Get just the VersionInfo for a version, without creating the version.
	/// This is useful for doing your own installation of things. This will download
	/// the version manifest if it is not downloaded already
	pub async fn get_version_info(&mut self, version: String) -> anyhow::Result<VersionInfo> {
		let mut o = output::NoOp;
		let manifest = self
			.get_version_manifest(&mut o)
			.await
			.context("Failed to get version manifest")?;
		let list = make_version_list(manifest).context("Failed to create version list")?;
		Ok(VersionInfo {
			version,
			versions: list,
		})
	}
}
