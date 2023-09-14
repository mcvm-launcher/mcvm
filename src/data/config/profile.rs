use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::data::id::{InstanceID, ProfileID};
use crate::data::profile::Profile;
use crate::util::versions::MinecraftVersionDeser;

use mcvm_shared::modifications::{ClientType, Modloader, ServerType};
use mcvm_shared::pkg::PackageStability;

use super::modifications::GameModifications;
use super::{instance::InstanceConfig, package::PackageConfig};

/// Configuration for a profile
#[derive(Deserialize, Serialize)]
pub struct ProfileConfig {
	version: MinecraftVersionDeser,
	/// Configured modloader
	#[serde(default)]
	pub modloader: Modloader,
	/// Configured client type
	#[serde(default)]
	pub client_type: ClientType,
	/// Configured server type
	#[serde(default)]
	pub server_type: ServerType,
	/// Configured list of instances in this profile
	pub instances: HashMap<InstanceID, InstanceConfig>,
	/// Packages on this profile
	#[serde(default)]
	pub packages: Vec<PackageConfig>,
	/// Default stability setting of packages on this profile
	#[serde(default)]
	pub package_stability: PackageStability,
}

impl ProfileConfig {
	/// Creates a profile from this profile configuration
	pub fn to_profile(&self, profile_id: ProfileID) -> Profile {
		Profile::new(
			profile_id,
			self.version.to_mc_version(),
			GameModifications::new(self.modloader, self.client_type, self.server_type),
		)
	}
}

/// Check if a client type can be installed by MCVM
pub fn can_install_client_type(client_type: ClientType) -> bool {
	matches!(
		client_type,
		ClientType::None | ClientType::Vanilla | ClientType::Fabric | ClientType::Quilt
	)
}

/// Check if a server type can be installed by MCVM
pub fn can_install_server_type(server_type: ServerType) -> bool {
	matches!(
		server_type,
		ServerType::None
			| ServerType::Vanilla
			| ServerType::Paper
			| ServerType::Fabric
			| ServerType::Quilt
	)
}
