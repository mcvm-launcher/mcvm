pub mod update;

use anyhow::Context;

use crate::data::instance::Instance;
use crate::io::files::paths::Paths;
use crate::io::lock::Lockfile;
use crate::package::PkgProfileConfig;
use crate::util::versions::MinecraftVersion;

use self::update::UpdateManager;

use super::config::profile::GameModifications;

pub type InstanceRegistry = std::collections::HashMap<String, Instance>;

#[derive(Debug)]
pub struct Profile {
	pub name: String,
	pub version: MinecraftVersion,
	pub instances: Vec<String>,
	pub packages: Vec<PkgProfileConfig>,
	pub modifications: GameModifications,
}

impl Profile {
	pub fn new(name: &str, version: MinecraftVersion, modifications: GameModifications) -> Self {
		Profile {
			name: name.to_owned(),
			version,
			instances: Vec::new(),
			packages: Vec::new(),
			modifications,
		}
	}

	pub fn add_instance(&mut self, instance: &str) {
		self.instances.push(instance.to_owned());
	}

	/// Create all the instances in this profile. Returns the version list.
	pub async fn create_instances(
		&mut self,
		reg: &mut InstanceRegistry,
		paths: &Paths,
		mut manager: UpdateManager,
		lock: &mut Lockfile,
	) -> anyhow::Result<Vec<String>> {
		for id in self.instances.iter_mut() {
			let instance = reg.get(id).expect("Profile has unknown instance");
			manager.add_requirements(instance.get_requirements());
		}
		manager.fulfill_requirements(paths, lock).await?;
		for id in self.instances.iter_mut() {
			let instance = reg.get_mut(id).expect("Profile has unknown instance");
			let files = instance
				.create(&manager, paths)
				.await
				.with_context(|| format!("Failed to create instance {id}"))?;
			manager.add_files(files);
		}
		Ok(manager.version_list.get_val())
	}
}
