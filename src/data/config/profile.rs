use std::collections::HashMap;

use mcvm_core::util::versions::MinecraftVersionDeser;
use mcvm_shared::Side;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::data::id::{InstanceID, ProfileID};
use crate::data::profile::Profile;

use mcvm_shared::modifications::{ClientType, Modloader, Proxy, ServerType};
use mcvm_shared::pkg::PackageStability;

use super::instance::InstanceConfig;
use super::package::PackageConfigDeser;

/// Configuration for a profile
#[derive(Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct ProfileConfig {
	/// The Minecraft version of this profile
	pub version: MinecraftVersionDeser,
	/// Configured modloader
	#[serde(default)]
	pub modloader: Modloader,
	/// Configured client type
	#[serde(default)]
	pub client_type: ClientType,
	/// Configured server type
	#[serde(default)]
	pub server_type: ServerType,
	/// Configured proxy
	#[serde(default)]
	pub proxy: Proxy,
	/// Configured list of instances in this profile
	#[serde(default)]
	pub instances: HashMap<InstanceID, InstanceConfig>,
	/// Packages on this profile
	#[serde(default)]
	pub packages: ProfilePackageConfiguration,
	/// Default stability setting of packages on this profile
	#[serde(default)]
	pub package_stability: PackageStability,
}

/// Different representations of package configuration on a profile
#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum ProfilePackageConfiguration {
	/// Is just a list of packages for every instance
	Simple(Vec<PackageConfigDeser>),
	/// Full configuration
	Full {
		/// Packages to apply to every instance
		#[serde(default)]
		global: Vec<PackageConfigDeser>,
		/// Packages to apply to only clients
		#[serde(default)]
		client: Vec<PackageConfigDeser>,
		/// Packages to apply to only servers
		#[serde(default)]
		server: Vec<PackageConfigDeser>,
	},
}

impl Default for ProfilePackageConfiguration {
	fn default() -> Self {
		Self::Simple(Vec::new())
	}
}

impl ProfilePackageConfiguration {
	/// Validate all the configured packages
	pub fn validate(&self) -> anyhow::Result<()> {
		match &self {
			Self::Simple(global) => {
				for pkg in global {
					pkg.validate()?;
				}
			}
			Self::Full {
				global,
				client,
				server,
			} => {
				for pkg in global.iter().chain(client.iter()).chain(server.iter()) {
					pkg.validate()?;
				}
			}
		}

		Ok(())
	}

	/// Iterate over all of the packages
	pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a PackageConfigDeser> + 'a> {
		match &self {
			Self::Simple(global) => Box::new(global.iter()),
			Self::Full {
				global,
				client,
				server,
			} => Box::new(global.iter().chain(client.iter()).chain(server.iter())),
		}
	}

	/// Iterate over the global package list
	pub fn iter_global(&self) -> impl Iterator<Item = &PackageConfigDeser> {
		match &self {
			Self::Simple(global) => global,
			Self::Full { global, .. } => global,
		}
		.iter()
	}

	/// Iterate over the package list for a specific side
	pub fn iter_side(&self, side: Side) -> impl Iterator<Item = &PackageConfigDeser> {
		match &self {
			Self::Simple(..) => [].iter(),
			Self::Full { client, server, .. } => match side {
				Side::Client => client.iter(),
				Side::Server => server.iter(),
			},
		}
	}

	/// Adds a package to the global list
	pub fn add_global_package(&mut self, pkg: PackageConfigDeser) {
		match self {
			Self::Simple(global) => global.push(pkg),
			Self::Full { global, .. } => global.push(pkg),
		}
	}

	/// Adds a package to the client list
	pub fn add_client_package(&mut self, pkg: PackageConfigDeser) {
		match self {
			Self::Simple(global) => {
				*self = Self::Full {
					global: global.clone(),
					client: vec![pkg],
					server: Vec::new(),
				}
			}
			Self::Full { client, .. } => client.push(pkg),
		}
	}

	/// Adds a package to the server list
	pub fn add_server_package(&mut self, pkg: PackageConfigDeser) {
		match self {
			Self::Simple(global) => {
				*self = Self::Full {
					global: global.clone(),
					client: Vec::new(),
					server: vec![pkg],
				}
			}
			Self::Full { server, .. } => server.push(pkg),
		}
	}
}

impl ProfileConfig {
	/// Creates a profile from this profile configuration
	pub fn to_profile(&self, profile_id: ProfileID) -> Profile {
		Profile::new(
			profile_id,
			self.version.to_mc_version(),
			GameModifications::new(
				self.modloader,
				self.client_type,
				self.server_type,
				self.proxy,
			),
			self.packages.clone(),
			self.package_stability,
		)
	}
}

/// Game modifications
#[derive(Clone, Debug)]
pub struct GameModifications {
	modloader: Modloader,
	/// Type of the client
	pub client_type: ClientType,
	/// Type of the server
	pub server_type: ServerType,
	/// Proxy
	pub proxy: Proxy,
}

impl GameModifications {
	/// Create a new GameModifications
	pub fn new(
		modloader: Modloader,
		client_type: ClientType,
		server_type: ServerType,
		proxy: Proxy,
	) -> Self {
		Self {
			modloader,
			client_type,
			server_type,
			proxy,
		}
	}

	/// Gets the modloader of a side
	pub fn get_modloader(&self, side: Side) -> Modloader {
		match side {
			Side::Client => match self.client_type {
				ClientType::None => self.modloader,
				ClientType::Vanilla => Modloader::Vanilla,
				ClientType::Forge => Modloader::Forge,
				ClientType::NeoForged => Modloader::NeoForged,
				ClientType::Fabric => Modloader::Fabric,
				ClientType::Quilt => Modloader::Quilt,
				ClientType::LiteLoader => Modloader::LiteLoader,
				ClientType::Risugamis => Modloader::Risugamis,
				ClientType::Rift => Modloader::Rift,
			},
			Side::Server => match self.server_type {
				ServerType::None => self.modloader,
				ServerType::Forge | ServerType::SpongeForge => Modloader::Forge,
				ServerType::NeoForged => Modloader::NeoForged,
				ServerType::Fabric => Modloader::Fabric,
				ServerType::Quilt => Modloader::Quilt,
				ServerType::Risugamis => Modloader::Risugamis,
				ServerType::Rift => Modloader::Rift,
				_ => Modloader::Vanilla,
			},
		}
	}

	/// Gets whether both client and server have the same modloader
	pub fn common_modloader(&self) -> bool {
		matches!(
			(self.client_type, self.server_type),
			(ClientType::None, ServerType::None)
				| (ClientType::Vanilla, ServerType::Vanilla)
				| (ClientType::Forge, ServerType::Forge)
				| (ClientType::NeoForged, ServerType::NeoForged)
				| (ClientType::Fabric, ServerType::Fabric)
				| (ClientType::Quilt, ServerType::Quilt)
				| (ClientType::Risugamis, ServerType::Risugamis)
				| (ClientType::Rift, ServerType::Rift)
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
			| ServerType::Folia
			| ServerType::Sponge
			| ServerType::Fabric
			| ServerType::Quilt
	)
}

/// Check if a proxy can be installed by MCVM
pub fn can_install_proxy(proxy: Proxy) -> bool {
	matches!(proxy, Proxy::None)
}
