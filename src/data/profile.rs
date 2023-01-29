use crate::data::instance::Instance;
use crate::data::instance::CreateError;
use crate::package::Package;
use crate::lib::versions::MinecraftVersion;
use crate::Paths;

pub type InstanceRegistry = std::collections::HashMap<String, Instance>;

#[derive(Debug)]
pub struct Profile {
	pub name: String,
	pub version: MinecraftVersion,
	pub instances: Vec<String>,
	pub packages: Vec<String>
}

impl Profile {
	pub fn new(name: &str, version: &MinecraftVersion) -> Self {
		Profile {
			name: name.to_owned(),
			version: version.to_owned(),
			instances: Vec::new(),
			packages: Vec::new()
		}
	}

	pub fn add_package(&mut self, pkg: &Package) {
		self.packages.push(pkg.name.to_owned());
	}

	pub fn add_instance(&mut self, instance: &str) {
		self.instances.push(instance.to_owned());
	}

	pub fn create_instances(
		&mut self,
		reg: &mut InstanceRegistry,
		paths: &Paths,
		verbose: bool,
		force: bool
	)
	-> Result<(), CreateError> {
		for id in self.instances.iter_mut() {
			let instance = reg.get_mut(id).expect("Profile has unknown instance");
			instance.create(paths, verbose, force)?;
		}
		Ok(())
	}
}

