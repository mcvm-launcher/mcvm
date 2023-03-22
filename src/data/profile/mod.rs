pub mod update;

use crate::data::instance::Instance;
use crate::package::PkgConfig;
use crate::Paths;
use crate::util::print::PrintOptions;

use self::update::UpdateManager;

use super::addon::Modloader;
use super::addon::PluginLoader;
use super::instance::create::CreateError;

pub type InstanceRegistry = std::collections::HashMap<String, Instance>;

#[derive(Debug)]
pub struct Profile {
	pub name: String,
	pub version: String,
	pub instances: Vec<String>,
	pub packages: Vec<PkgConfig>,
	pub modloader: Modloader,
	pub plugin_loader: PluginLoader,
}

impl Profile {
	pub fn new(
		name: &str,
		version: &str,
		modloader: Modloader,
		plugin_loader: PluginLoader,
	) -> Self {
		Profile {
			name: name.to_owned(),
			version: version.to_owned(),
			instances: Vec::new(),
			packages: Vec::new(),
			modloader,
			plugin_loader,
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
		verbose: bool,
		force: bool,
	) -> Result<Vec<String>, CreateError> {
		let options = PrintOptions::new(verbose, 0);
		let mut manager = UpdateManager::new(options, force);
		for id in self.instances.iter_mut() {
			let instance = reg.get(id).expect("Profile has unknown instance");
			manager.add_requirements(instance.get_requirements());
		}
		let version_list = manager.fulfill_requirements(paths, &self.version).await?;
		for id in self.instances.iter_mut() {
			let instance = reg.get_mut(id).expect("Profile has unknown instance");
			let files = instance
				.create(&manager, paths)
				.await?;
			manager.add_files(files);
		}
		Ok(version_list)
	}
}
