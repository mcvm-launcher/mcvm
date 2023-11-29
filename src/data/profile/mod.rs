/// Functions for updating profiles
pub mod update;

use anyhow::Context;
use mcvm_core::MCVMCore;
use mcvm_shared::output::MCVMOutput;
use mcvm_shared::pkg::PackageStability;
use reqwest::Client;

use crate::data::instance::Instance;
use crate::io::files::paths::Paths;
use mcvm_core::util::versions::MinecraftVersion;

use self::update::manager::UpdateManager;

use super::config::profile::GameModifications;
use super::config::profile::ProfilePackageConfiguration;
use super::id::{InstanceID, ProfileID};
use mcvm_core::user::UserManager;

/// A hashmap of InstanceIDs to Instances
pub type InstanceRegistry = std::collections::HashMap<InstanceID, Instance>;

/// A user profile which applies many settings to contained instances
#[derive(Debug)]
pub struct Profile {
	/// The ID of this profile
	pub id: ProfileID,
	/// The Minecraft version of this profile
	pub version: MinecraftVersion,
	/// The instances that are contained in this profile
	pub instances: Vec<InstanceID>,
	/// The packages that are selected for this profile
	pub packages: ProfilePackageConfiguration,
	/// Modifications applied to instances in this profile
	pub modifications: GameModifications,
	/// The default stability for packages in this profile
	pub default_stability: PackageStability,
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
			instances: Vec::new(),
			packages,
			modifications,
			default_stability,
		}
	}

	/// Add a new instance to this profile
	pub fn add_instance(&mut self, instance: InstanceID) {
		self.instances.push(instance);
	}

	/// Create all the instances in this profile. Returns the version list.
	pub async fn create_instances(
		&mut self,
		reg: &mut InstanceRegistry,
		paths: &Paths,
		mut manager: UpdateManager,
		users: &UserManager,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Vec<String>> {
		for id in self.instances.iter_mut() {
			let instance = reg.get(id).expect("Profile has unknown instance");
			manager.add_requirements(instance.get_requirements());
		}
		let client = Client::new();
		manager.fulfill_requirements(paths, &client, o).await?;
		for id in self.instances.iter_mut() {
			// FIXME: This sucks
			let mut core = MCVMCore::new().context("Failed to initialize core")?;
			core.get_users().steal_users(users);
			let mut installed_version = core
				.get_version(&self.version, o)
				.await
				.context("Failed to get version")?;
			let instance = reg.get_mut(id).expect("Profile has unknown instance");
			{
				instance
					.create(&mut installed_version, &manager, paths, users, &client, o)
					.await
					.with_context(|| format!("Failed to create instance {id}"))?;
			}
		}
		Ok(manager.version_info.get_val().versions)
	}
}
