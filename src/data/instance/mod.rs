/// Addon-related functions for instances
mod addons;
/// Creation of instance contents
pub mod create;
/// Launching an instance
pub mod launch;
/// Managing and installing packages on an instance
pub mod packages;

use mcvm_shared::later::Later;
use mcvm_shared::Side;

use self::create::{InstanceDirs, ModificationData};
use self::launch::LaunchOptions;

use super::config::instance::ClientWindowConfig;
use super::config::package::PackageConfig;
use super::config::profile::GameModifications;
use mcvm_shared::id::{InstanceID, InstanceRef, ProfileID};

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
		/// Configuration for the client window
		window: ClientWindowConfig,
	},
	/// A server instance
	Server {
		/// The new world name if it is changed by the options
		world_name: Option<String>,
	},
}

impl InstKind {
	/// Create a new client InstKind
	pub fn client(window: ClientWindowConfig) -> Self {
		Self::Client { window }
	}

	/// Create a new server InstKind
	pub fn server() -> Self {
		Self::Server { world_name: None }
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
	/// The packages on the instance, consolidated from all parent sources
	pub packages: Vec<PackageConfig>,
	/// Custom plugin config
	pub plugin_config: serde_json::Map<String, serde_json::Value>,
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

	/// Get the instance ref for this instance
	pub fn get_inst_ref(&self) -> InstanceRef {
		InstanceRef::new(self.profile_id.clone(), self.id.clone())
	}
}
