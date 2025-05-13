use std::collections::HashMap;

use anyhow::bail;
use mcvm_shared::id::ProfileID;
use mcvm_shared::Side;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::instance::{merge_instance_configs, InstanceConfig};
use super::package::PackageConfigDeser;

/// Configuration for a profile
#[derive(Deserialize, Serialize, Clone, Default)]
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
