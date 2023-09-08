/// Functions for updating profiles
pub mod update;

use anyhow::Context;
use mcvm_shared::output::MCVMOutput;
use reqwest::Client;

use crate::data::instance::Instance;
use crate::io::files::paths::Paths;
use crate::io::lock::Lockfile;
use crate::package::PkgProfileConfig;
use crate::util::versions::MinecraftVersion;

use self::update::manager::UpdateManager;

use super::config::profile::GameModifications;
use super::id::{InstanceID, ProfileID};
use super::user::UserManager;

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
	pub packages: Vec<PkgProfileConfig>,
	/// Modifications applied to instances in this profile
	pub modifications: GameModifications,
}

impl Profile {
	/// Create a new profile
	pub fn new(id: ProfileID, version: MinecraftVersion, modifications: GameModifications) -> Self {
		Profile {
			id,
			version,
			instances: Vec::new(),
			packages: Vec::new(),
			modifications,
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
		lock: &mut Lockfile,
		users: &UserManager,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Vec<String>> {
		for id in self.instances.iter_mut() {
			let instance = reg.get(id).expect("Profile has unknown instance");
			manager.add_requirements(instance.get_requirements());
		}
		let client = Client::new();
		manager
			.fulfill_requirements(paths, lock, &client, o)
			.await?;
		for id in self.instances.iter_mut() {
			let instance = reg.get_mut(id).expect("Profile has unknown instance");
			let result = instance
				.create(&manager, paths, users, &client, o)
				.await
				.with_context(|| format!("Failed to create instance {id}"))?;
			manager.add_result(result);
		}
		Ok(manager.version_info.get_val().versions)
	}
}
