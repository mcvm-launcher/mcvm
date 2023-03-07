use crate::data::instance::Instance;
use crate::package::PkgConfig;
use crate::util::json;
use crate::Paths;

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
	pub plugin_loader: PluginLoader
}

impl Profile {
	pub fn new(name: &str, version: &str, modloader: Modloader, plugin_loader: PluginLoader) -> Self {
		Profile {
			name: name.to_owned(),
			version: version.to_owned(),
			instances: Vec::new(),
			packages: Vec::new(),
			modloader,
			plugin_loader
		}
	}

	pub fn add_instance(&mut self, instance: &str) {
		self.instances.push(instance.to_owned());
	}

	pub async fn create_instances(
		&mut self,
		reg: &mut InstanceRegistry,
		version_manifest: &json::JsonObject,
		paths: &Paths,
		verbose: bool,
		force: bool
	) -> Result<(), CreateError> {
		for id in self.instances.iter_mut() {
			let instance = reg.get_mut(id).expect("Profile has unknown instance");
			instance.create(version_manifest, paths, verbose, force).await?;
		}
		Ok(())
	}
}
