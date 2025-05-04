use std::collections::HashMap;

use anyhow::bail;
use mcvm_shared::id::ProfileID;
use mcvm_shared::Side;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use mcvm_shared::modifications::{ClientType, Modloader, Proxy, ServerType};

use super::instance::{merge_instance_configs, InstanceConfig};
use super::package::PackageConfigDeser;

/// Configuration for a profile
#[derive(Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct ProfileConfig {
	/// The configuration for the instance
	#[serde(flatten)]
	pub instance: InstanceConfig,
	/// Package configuration
	#[serde(default)]
	pub packages: ProfilePackageConfiguration,
}

impl ProfileConfig {
	/// Merge this profile with another one
	pub fn merge(&mut self, other: Self) {
		self.instance = merge_instance_configs(&self.instance, other.instance);
	}
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

/// Consolidates profile configs into the full profiles
pub fn consolidate_profile_configs(
	profiles: HashMap<ProfileID, ProfileConfig>,
	global_profile: Option<&ProfileConfig>,
) -> anyhow::Result<HashMap<ProfileID, ProfileConfig>> {
	let mut out: HashMap<_, ProfileConfig> = HashMap::with_capacity(profiles.len());

	let max_iterations = 10000;

	// We do this by repeatedly finding a profile with an already resolved ancenstor
	let mut i = 0;
	while out.len() != profiles.len() {
		for (id, profile) in &profiles {
			// Don't redo profiles that are already done
			if out.contains_key(id) {
				continue;
			}

			if profile.instance.common.from.is_empty() {
				// Profiles with no ancestor can just be added directly to the output, after deriving from the global profile
				let mut profile = profile.clone();
				if let Some(global_profile) = global_profile {
					let overlay = profile;
					profile = global_profile.clone();
					profile.merge(overlay);
				}
				out.insert(id.clone(), profile);
			} else {
				for parent in profile.instance.common.from.iter() {
					// If the parent is already in the map (already consolidated) then we can derive from it and add to the map
					if let Some(parent) = out.get(&ProfileID::from(parent.clone())) {
						let mut new = parent.clone();
						new.merge(profile.clone());
						out.insert(id.clone(), new);
					} else {
						bail!("Parent profile '{parent}' does not exist");
					}
				}
			}
		}

		i += 1;
		if i > max_iterations {
			panic!("Max iterations exceeded while resolving profiles. This is a bug in MCVM.");
		}
	}

	Ok(out)
}

/// Game modifications
#[derive(Clone, Debug)]
pub struct GameModifications {
	modloader: Modloader,
	/// Type of the client
	client_type: ClientType,
	/// Type of the server
	server_type: ServerType,
}

impl GameModifications {
	/// Create a new GameModifications
	pub fn new(modloader: Modloader, client_type: ClientType, server_type: ServerType) -> Self {
		Self {
			modloader,
			client_type,
			server_type,
		}
	}

	/// Gets the client type
	pub fn client_type(&self) -> ClientType {
		if let ClientType::None = self.client_type {
			match &self.modloader {
				Modloader::Vanilla => ClientType::Vanilla,
				Modloader::Forge => ClientType::Forge,
				Modloader::NeoForged => ClientType::NeoForged,
				Modloader::Fabric => ClientType::Fabric,
				Modloader::Quilt => ClientType::Quilt,
				Modloader::LiteLoader => ClientType::LiteLoader,
				Modloader::Risugamis => ClientType::Risugamis,
				Modloader::Rift => ClientType::Rift,
				Modloader::Unknown(modloader) => ClientType::Unknown(modloader.clone()),
			}
		} else {
			self.client_type.clone()
		}
	}

	/// Gets the server type
	pub fn server_type(&self) -> ServerType {
		if let ServerType::None = self.server_type {
			match &self.modloader {
				Modloader::Vanilla => ServerType::Vanilla,
				Modloader::Forge => ServerType::Forge,
				Modloader::NeoForged => ServerType::NeoForged,
				Modloader::Fabric => ServerType::Fabric,
				Modloader::Quilt => ServerType::Quilt,
				Modloader::LiteLoader => ServerType::Unknown("liteloader".into()),
				Modloader::Risugamis => ServerType::Risugamis,
				Modloader::Rift => ServerType::Rift,
				Modloader::Unknown(modloader) => ServerType::Unknown(modloader.clone()),
			}
		} else {
			self.server_type.clone()
		}
	}

	/// Gets the modloader of a side
	pub fn get_modloader(&self, side: Side) -> Modloader {
		match side {
			Side::Client => match self.client_type {
				ClientType::None => self.modloader.clone(),
				ClientType::Vanilla => Modloader::Vanilla,
				ClientType::Forge => Modloader::Forge,
				ClientType::NeoForged => Modloader::NeoForged,
				ClientType::Fabric => Modloader::Fabric,
				ClientType::Quilt => Modloader::Quilt,
				ClientType::LiteLoader => Modloader::LiteLoader,
				ClientType::Risugamis => Modloader::Risugamis,
				ClientType::Rift => Modloader::Rift,
				_ => Modloader::Vanilla,
			},
			Side::Server => match self.server_type {
				ServerType::None => self.modloader.clone(),
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
			(&self.client_type, &self.server_type),
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
pub fn can_install_client_type(client_type: &ClientType) -> bool {
	matches!(client_type, ClientType::None | ClientType::Vanilla)
}

/// Check if a server type can be installed by MCVM
pub fn can_install_server_type(server_type: &ServerType) -> bool {
	matches!(server_type, ServerType::None | ServerType::Vanilla)
}

/// Check if a proxy can be installed by MCVM
pub fn can_install_proxy(proxy: Proxy) -> bool {
	// TODO: Support Velocity
	matches!(proxy, Proxy::None)
}
