/// Addon-related functions for instances
mod addons;
/// Creation of instance contents
pub mod create;
/// Launching an instance
pub mod launch;
/// Managing and installing packages on an instance
pub mod packages;

use anyhow::Context;
use mcvm_options::client::ClientOptions;
use mcvm_options::server::ServerOptions;
use mcvm_shared::later::Later;
use mcvm_shared::Side;

use crate::io::files::paths::Paths;
use crate::io::snapshot;

use self::create::{InstanceDirs, ModificationData};
use self::launch::LaunchOptions;

use super::config::instance::ClientWindowConfig;
use super::config::package::PackageConfig;
use super::config::profile::GameModifications;
use super::id::{InstanceID, ProfileID};

use std::path::PathBuf;

/// An instance of the game on a profile
#[derive(Debug)]
pub struct Instance {
	/// What type of instance this is
	pub(crate) kind: InstKind,
	/// The ID of this instance
	pub(crate) id: InstanceID,
	/// The ID of the parent profile for this instance
	pub(crate) profile_id: ProfileID,
	/// Directories of the instance
	pub(crate) dirs: Later<InstanceDirs>,
	/// Configuration for the instance
	config: InstanceStoredConfig,
	/// Modification data
	modification_data: ModificationData,
}

/// Different kinds of instances and their associated data
#[derive(Debug, Clone)]
pub enum InstKind {
	/// A client instance
	Client {
		/// Options for the client
		options: Option<Box<ClientOptions>>,
		/// Configuration for the client window
		window: ClientWindowConfig,
	},
	/// A server instance
	Server {
		/// Options for the server
		options: Option<Box<ServerOptions>>,
		/// The new world name if it is changed by the options
		world_name: Option<String>,
	},
}

impl InstKind {
	/// Create a new client InstKind
	pub fn client(options: Option<Box<ClientOptions>>, window: ClientWindowConfig) -> Self {
		Self::Client { options, window }
	}

	/// Create a new server InstKind
	pub fn server(options: Option<Box<ServerOptions>>) -> Self {
		Self::Server {
			options,
			world_name: None,
		}
	}

	/// Convert to the Side enum
	pub fn to_side(&self) -> Side {
		match self {
			Self::Client { .. } => Side::Client,
			Self::Server { .. } => Side::Server,
		}
	}
}

/// The stored configuration on an instance
#[derive(Debug)]
pub struct InstanceStoredConfig {
	/// Modifications to the instance
	pub modifications: GameModifications,
	/// Launch options for the instance
	pub launch: LaunchOptions,
	/// The instance's global datapack folder
	pub datapack_folder: Option<String>,
	/// The instance's snapshot configuration
	pub snapshot_config: snapshot::Config,
	/// The packages on the instance, consolidated from all parent sources
	pub packages: Vec<PackageConfig>,
}

impl Instance {
	/// Create a new instance
	pub fn new(
		kind: InstKind,
		id: InstanceID,
		profile_id: ProfileID,
		config: InstanceStoredConfig,
	) -> Self {
		Self {
			kind,
			id,
			profile_id,
			config,
			dirs: Later::Empty,
			modification_data: ModificationData::new(),
		}
	}

	/// Get the kind of the instance
	pub fn get_kind(&self) -> &InstKind {
		&self.kind
	}

	/// Get the side of the instance
	pub fn get_side(&self) -> Side {
		self.kind.to_side()
	}

	/// Get the ID of the instance
	pub fn get_id(&self) -> &InstanceID {
		&self.id
	}

	/// Get the ID of the instance's parent profile
	pub fn get_profile_id(&self) -> &ProfileID {
		&self.profile_id
	}

	/// Get the instance's directories
	pub fn get_dirs(&self) -> &Later<InstanceDirs> {
		&self.dirs
	}
}

/// Snapshot-related functions
impl Instance {
	/// Starts snapshot interactions by generating the path and opening the index
	pub fn open_snapshot_index(&self, paths: &Paths) -> anyhow::Result<(PathBuf, snapshot::Index)> {
		let snapshot_dir = snapshot::get_snapshot_directory(&self.id, paths);
		let index =
			snapshot::Index::open(&snapshot_dir).context("Failed to open snapshot index")?;
		Ok((snapshot_dir, index))
	}

	/// Creates a new snapshot for this instance
	pub fn create_snapshot(
		&mut self,
		id: String,
		kind: snapshot::SnapshotKind,
		paths: &Paths,
	) -> anyhow::Result<()> {
		self.ensure_dirs(paths)?;
		let (snapshot_dir, mut index) = self.open_snapshot_index(paths)?;

		index.create_snapshot(
			kind,
			id,
			&self.config.snapshot_config,
			&self.id,
			&self.dirs.get().game_dir,
			paths,
		)?;

		index.finish(&snapshot_dir)?;
		Ok(())
	}

	/// Removes a snapshot from this instance
	pub fn remove_snapshot(&self, id: &str, paths: &Paths) -> anyhow::Result<()> {
		let (snapshot_dir, mut index) = self.open_snapshot_index(paths)?;

		index.remove_snapshot(id, &self.id, paths)?;

		index.finish(&snapshot_dir)?;
		Ok(())
	}

	/// Restores a snapshot for this instance
	pub async fn restore_snapshot(&mut self, id: &str, paths: &Paths) -> anyhow::Result<()> {
		self.ensure_dirs(paths)?;
		let (snapshot_dir, index) = self.open_snapshot_index(paths)?;

		index
			.restore_snapshot(id, &self.id, &self.dirs.get().game_dir, paths)
			.await?;

		index.finish(&snapshot_dir)?;
		Ok(())
	}
}
