use std::collections::HashMap;
use std::sync::Arc;

use anyhow::bail;
use mcvm_core::auth_crate::mc::ClientId;
use mcvm_core::user::{User, UserManager};
use mcvm_plugin::plugin::PluginManifest;
use mcvm_shared::id::InstanceID;
use mcvm_shared::modifications::{ClientType, Modloader, ServerType};
use mcvm_shared::output::MCVMOutput;
use mcvm_shared::pkg::{PackageID, PackageStability};
use mcvm_shared::Side;

use crate::instance::Instance;
use crate::io::paths::Paths;
use crate::pkg::eval::EvalPermissions;
use crate::pkg::reg::PkgRegistry;
use crate::pkg::repo::PkgRepo;
use crate::plugin::PluginManager;

use super::instance::{read_instance_config, ClientWindowConfig, InstanceConfig, LaunchConfig};
use super::package::{FullPackageConfig, PackageConfigDeser};
use super::plugin::PluginConfig;
use super::preferences::ConfigPreferences;
use super::user::{UserConfig, UserVariant};
use super::Config;

/// Simple builder for config
pub struct ConfigBuilder {
	users: UserManager,
	instances: HashMap<InstanceID, Instance>,
	instance_groups: HashMap<Arc<str>, Vec<InstanceID>>,
	packages: PkgRegistry,
	preferences: ConfigPreferences,
	plugins: PluginManager,
	default_user: Option<String>,
}

impl ConfigBuilder {
	/// Construct a new ConfigBuilder
	pub fn new(prefs: ConfigPreferences, repos: Vec<PkgRepo>) -> Self {
		let packages = PkgRegistry::new(repos, prefs.package_caching_strategy.clone());
		Self {
			users: UserManager::new(ClientId::new("".into())),
			instances: HashMap::new(),
			instance_groups: HashMap::new(),
			packages,
			preferences: prefs,
			plugins: PluginManager::new(),
			default_user: None,
		}
	}

	/// Create a UserBuilder
	pub fn user(&mut self, id: String, kind: UserBuilderKind) -> UserBuilder {
		UserBuilder::with_parent(id, kind, Some(self))
	}

	/// Finish a UserBuilder
	fn build_user(&mut self, user: User) {
		self.users.add_user(user);
	}

	/// Set the default user
	pub fn default_user(&mut self, user_id: String) -> &mut Self {
		self.default_user = Some(user_id);

		self
	}

	/// Add an instance group
	pub fn instance_group(&mut self, id: Arc<str>, contents: Vec<InstanceID>) -> &mut Self {
		self.instance_groups.insert(id, contents);

		self
	}

	/// Add a plugin configuration
	pub fn add_plugin(
		&mut self,
		plugin: PluginConfig,
		manifest: PluginManifest,
		paths: &Paths,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		self.plugins.add_plugin(plugin, manifest, paths, None, o)
	}

	/// Finishes the builder
	pub fn build(mut self) -> anyhow::Result<Config> {
		if let Some(default_user_id) = &self.default_user {
			if self.users.user_exists(default_user_id) {
				self.users
					.choose_user(default_user_id)
					.expect("Default user should exist");
			} else {
				bail!("Provided default user '{default_user_id}' does not exist");
			}
		}

		Ok(Config {
			users: self.users,
			instances: self.instances,
			instance_groups: self.instance_groups,
			packages: self.packages,
			plugins: self.plugins,
			prefs: self.preferences,
		})
	}
}

/// Builder for a User
pub struct UserBuilder<'parent> {
	id: String,
	config: UserConfig,
	parent: Option<&'parent mut ConfigBuilder>,
}

impl<'parent> UserBuilder<'parent> {
	/// Construct a new UserBuilder
	pub fn new(id: String, kind: UserBuilderKind) -> Self {
		Self::with_parent(id, kind, None)
	}

	/// Construct with a parent
	fn with_parent(
		id: String,
		kind: UserBuilderKind,
		parent: Option<&'parent mut ConfigBuilder>,
	) -> Self {
		let variant = match kind {
			UserBuilderKind::Microsoft => UserVariant::Microsoft {},
			UserBuilderKind::Demo => UserVariant::Demo {},
		};
		Self {
			id,
			config: UserConfig { variant },
			parent,
		}
	}

	/// Finish the builder and go to the parent
	pub fn build(self) {
		let (user, parent) = self.build_self();
		if let Some(parent) = parent {
			parent.build_user(user);
		}
	}

	/// Finish the builder and return the self
	pub fn build_self(self) -> (User, Option<&'parent mut ConfigBuilder>) {
		let built = self.config.to_user(&self.id);
		(built, self.parent)
	}
}

/// User kind for a UserBuilder
#[derive(Copy, Clone)]
pub enum UserBuilderKind {
	/// A Microsoft user
	Microsoft,
	/// A demo user
	Demo,
}

/// Builder for an instance
pub struct InstanceBuilder<'parent> {
	id: InstanceID,
	config: InstanceConfig,
	parent: Option<&'parent mut ConfigBuilder>,
}

impl<'parent> InstanceBuilder<'parent> {
	/// Construct a new InstanceBuilder
	pub fn new(id: InstanceID, side: Side) -> Self {
		Self::with_parent(id, side, None)
	}

	/// Construct with a parent
	fn with_parent(id: InstanceID, side: Side, parent: Option<&'parent mut ConfigBuilder>) -> Self {
		let config = InstanceConfig {
			side: Some(side),
			name: None,
			common: Default::default(),
			window: Default::default(),
		};

		Self { id, config, parent }
	}

	/// Set the name of the instance
	pub fn name(&mut self, name: String) -> &mut Self {
		self.config.name = Some(name);
		self
	}

	/// Set the modloader of the instance
	pub fn modloader(&mut self, modloader: Modloader) -> &mut Self {
		self.config.common.modloader = Some(modloader);
		self
	}

	/// Set the client type of the instance
	pub fn client_type(&mut self, client_type: ClientType) -> &mut Self {
		self.config.common.client_type = Some(client_type);
		self
	}

	/// Set the server type of the instance
	pub fn server_type(&mut self, server_type: ServerType) -> &mut Self {
		self.config.common.server_type = Some(server_type);
		self
	}

	/// Set the default package stability of the instance
	pub fn package_stability(&mut self, package_stability: PackageStability) -> &mut Self {
		self.config.common.package_stability = Some(package_stability);
		self
	}

	/// Create a PackageBuilder
	pub fn package<'this>(
		&'this mut self,
		data: InitialPackageData,
	) -> PackageBuilder<PackageBuilderInstanceParent<'this, 'parent>> {
		let parent = PackageBuilderInstanceParent(self);
		PackageBuilder::with_parent(data, parent)
	}

	/// Finish a PackageBuilder
	fn build_package(&mut self, package: FullPackageConfig) {
		let config = PackageConfigDeser::Full(package);
		self.config.common.packages.push(config);
	}

	/// Set the launch options of the instance
	pub fn launch_options(&mut self, launch_options: LaunchConfig) -> &mut Self {
		self.config.common.launch = launch_options;

		self
	}

	/// Set the client window config of the instance
	pub fn window_config(&mut self, window_config: ClientWindowConfig) -> &mut Self {
		self.config.window = window_config;

		self
	}

	/// Set the datapack folder of the instance
	pub fn datapack_folder(&mut self, folder: String) -> &mut Self {
		self.config.common.datapack_folder = Some(folder);

		self
	}

	/// Finish the builder and go to the parent
	pub fn build(self, paths: &Paths, o: &mut impl MCVMOutput) -> anyhow::Result<()> {
		let (id, instance, parent) = self.build_self(paths, o)?;
		if let Some(parent) = parent {
			parent.instances.insert(id, instance);
		}

		Ok(())
	}

	/// Finish the builder and return the self
	pub fn build_self(
		self,
		paths: &Paths,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<(InstanceID, Instance, Option<&'parent mut ConfigBuilder>)> {
		let default_plugins = PluginManager::new();
		let plugins = if let Some(ref parent) = self.parent {
			&parent.plugins
		} else {
			&default_plugins
		};
		let built = read_instance_config(
			self.id.clone(),
			self.config,
			&HashMap::new(),
			plugins,
			paths,
			o,
		)?;

		Ok((self.id, built, self.parent))
	}
}

/// Builder for a package
pub struct PackageBuilder<Parent: PackageBuilderParent> {
	config: FullPackageConfig,
	parent: Parent,
}

impl<Parent> PackageBuilder<Parent>
where
	Parent: PackageBuilderParent,
{
	/// Construct with a parent
	fn with_parent(data: InitialPackageData, parent: Parent) -> Self {
		let config = FullPackageConfig {
			id: data.id,
			features: Default::default(),
			use_default_features: true,
			permissions: Default::default(),
			stability: Default::default(),
			worlds: Default::default(),
			content_version: Default::default(),
		};

		Self { config, parent }
	}

	/// Add to the package's features
	pub fn features(&mut self, features: Vec<String>) -> &mut Self {
		self.config.features.extend(features);
		self
	}

	/// Set the use_default_features setting of the package
	pub fn use_default_features(&mut self, value: bool) -> &mut Self {
		self.config.use_default_features = value;
		self
	}

	/// Set the permissions of the package
	pub fn permissions(&mut self, permissions: EvalPermissions) -> &mut Self {
		self.config.permissions = permissions;
		self
	}

	/// Set the configured stability of the package
	pub fn stability(&mut self, stability: PackageStability) -> &mut Self {
		self.config.stability = Some(stability);
		self
	}

	/// Set the configured worlds of the package
	pub fn worlds(&mut self, worlds: Vec<String>) -> &mut Self {
		self.config.worlds = worlds;
		self
	}

	/// Set the configured content version of the package
	pub fn content_version(&mut self, version: String) -> &mut Self {
		self.config.content_version = Some(version);
		self
	}

	/// Finish the builder and go to the parent
	pub fn build(self) {
		self.parent.build_package(self.config);
	}
}

impl PackageBuilder<PackageBuilderOrphan> {
	/// Construct a new PackageBuilder
	pub fn new(data: InitialPackageData) -> Self {
		Self::with_parent(data, PackageBuilderOrphan)
	}
}

/// Initial data for a PackageBuilder
pub struct InitialPackageData {
	id: PackageID,
}

/// Trait for a parent builder that can have a PackageBuilder added
pub trait PackageBuilderParent {
	/// Add the package to the parent
	fn build_package(self, package: FullPackageConfig);
}

/// Data for a PackageBuilder with no parent
pub struct PackageBuilderOrphan;

impl PackageBuilderParent for PackageBuilderOrphan {
	fn build_package(self, _package: FullPackageConfig) {}
}

/// Data for a PackageBuilder that returns to an InstanceBuilder
pub struct PackageBuilderInstanceParent<'instance, 'parent>(
	&'instance mut InstanceBuilder<'parent>,
);

impl<'instance, 'parent> PackageBuilderParent for PackageBuilderInstanceParent<'instance, 'parent> {
	fn build_package(self, package: FullPackageConfig) {
		self.0.build_package(package)
	}
}

#[cfg(test)]
mod tests {
	// use mcvm_plugin::api::NoOp;
	// use mcvm_shared::lang::Language;

	// use crate::data::config::preferences::{PrefDeser, RepositoriesDeser};
	// use crate::pkg::reg::CachingStrategy;

	// use super::*;

	// #[test]
	// fn test_config_building() {
	// 	let (prefs, repos) = get_prefs().expect("Failed to get preferences");
	// 	let mut config = ConfigBuilder::new(prefs, repos);
	// 	let mut profile = config.profile(
	// 		"profile".into(),
	// 		MinecraftVersionDeser::Version("1.19.3".into()),
	// 	);
	// 	modify_profile(&mut profile);
	// 	config
	// 		.package(InitialPackageData {
	// 			id: "global-package".into(),
	// 		})
	// 		.build();
	// 	config
	// 		.user("user".into(), UserBuilderKind::Microsoft)
	// 		.build();
	// 	config.default_user("user".into());
	// 	let config = config.build().expect("Failed to build config");
	// 	assert!(config.users.user_exists("user"));
	// 	assert_eq!(
	// 		config.users.get_chosen_user().map(|x| x.get_id().clone()),
	// 		Some("user".into())
	// 	);
	// }

	// fn get_prefs() -> anyhow::Result<(ConfigPreferences, Vec<PkgRepo>)> {
	// 	let deser = PrefDeser {
	// 		repositories: RepositoriesDeser::default(),
	// 		package_caching_strategy: CachingStrategy::default(),
	// 		language: Language::default(),
	// 	};
	// 	ConfigPreferences::read(&deser)
	// }
}
