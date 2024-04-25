/// Installing and launching proxies on profiles
pub mod proxy;
/// Functions for updating profiles
pub mod update;

use std::collections::HashMap;

use anyhow::Context;
use mcvm_shared::later::Later;
use mcvm_shared::output::MCVMOutput;
use mcvm_shared::pkg::PackageStability;
use reqwest::Client;

use crate::data::instance::Instance;
use crate::io::files::paths::Paths;
use mcvm_core::util::versions::MinecraftVersion;

use self::proxy::ProxyProperties;
use self::update::manager::UpdateManager;

use super::config::plugin::PluginManager;
use super::config::profile::GameModifications;
use super::config::profile::ProfilePackageConfiguration;
use mcvm_core::user::UserManager;
use mcvm_shared::id::InstanceRef;
use mcvm_shared::id::{InstanceID, ProfileID};

/// A hashmap of InstanceIDs to Instances
pub type InstanceRegistry = std::collections::HashMap<InstanceRef, Instance>;

/// A user profile which applies many settings to contained instances
#[derive(Debug)]
pub struct Profile {
	/// The ID of this profile
	pub id: ProfileID,
	/// The Minecraft version of this profile
	pub version: MinecraftVersion,
	/// The instances that are contained in this profile
	pub instances: HashMap<InstanceID, Instance>,
	/// The packages that are selected for this profile
	pub packages: ProfilePackageConfiguration,
	/// Modifications applied to instances in this profile
	pub modifications: GameModifications,
	/// The default stability for packages in this profile
	pub default_stability: PackageStability,
	/// The profile's proxy properties, fulfilled when creating
	proxy_props: Later<ProxyProperties>,
}

impl Profile {
	/// Create a new profile
	pub fn new(
		id: ProfileID,
		version: MinecraftVersion,
		modifications: GameModifications,
		packages: ProfilePackageConfiguration,
		default_stability: PackageStability,
	) -> Self {
		Profile {
			id,
			version,
			instances: HashMap::new(),
			packages,
			modifications,
			default_stability,
			proxy_props: Later::Empty,
		}
	}

	/// Add a new instance to this profile
	pub fn add_instance(&mut self, instance: Instance) {
		self.instances.insert(instance.id.clone(), instance);
	}

	/// Get the InstanceRef of an instance on this profile
	pub fn get_inst_ref(&self, instance: &InstanceID) -> InstanceRef {
		InstanceRef::new(self.id.clone(), instance.clone())
	}

	/// Create this profile and all of it's instances
	pub async fn create(
		&mut self,
		paths: &Paths,
		manager: &mut UpdateManager,
		users: &UserManager,
		plugins: &PluginManager,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		for instance in self.instances.values_mut() {
			manager.add_requirements(instance.get_requirements());
		}
		manager
			.fulfill_requirements(users, plugins, paths, client, o)
			.await?;

		for instance in self.instances.values_mut() {
			let result = instance
				.create(manager, plugins, paths, users, client, o)
				.await
				.with_context(|| format!("Failed to create instance {}", instance.id))?;
			manager.add_result(result);
		}

		// Update the proxy
		self.create_proxy(manager, paths, client, o)
			.await
			.context("Failed to create proxy")?;

		Ok(())
	}
}
