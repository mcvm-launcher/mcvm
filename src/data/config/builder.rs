use std::collections::HashMap;

use anyhow::bail;
use mcvm_core::user::{User, UserManager};
use mcvm_core::util::versions::MinecraftVersionDeser;
use mcvm_pkg::PackageContentType;
use mcvm_shared::modifications::{ClientType, Modloader, ServerType};
use mcvm_shared::pkg::{PackageID, PackageStability};
use mcvm_shared::Side;
use oauth2::ClientId;

use crate::data::id::{InstanceID, ProfileID};
use crate::data::instance::Instance;
use crate::data::profile::{InstanceRegistry, Profile};
use crate::io::snapshot;
use crate::package::eval::EvalPermissions;
use crate::package::reg::PkgRegistry;
use crate::package::repo::PkgRepo;

use super::instance::{
	read_instance_config, ClientWindowConfig, FullInstanceConfig, InstanceConfig, LaunchConfig,
};
use super::package::{FullPackageConfig, PackageConfig, PackageType};
use super::preferences::ConfigPreferences;
use super::profile::{ProfileConfig, ProfilePackageConfiguration};
use super::user::{UserConfig, UserVariant};
use super::Config;

/// Simple builder for config
pub struct ConfigBuilder {
	users: UserManager,
	instances: InstanceRegistry,
	profiles: HashMap<ProfileID, Profile>,
	packages: PkgRegistry,
	preferences: ConfigPreferences,
	global_packages: Vec<PackageConfig>,
	default_user: Option<String>,
}

impl ConfigBuilder {
	/// Construct a new ConfigBuilder
	pub fn new(prefs: ConfigPreferences, repos: Vec<PkgRepo>) -> Self {
		let packages = PkgRegistry::new(repos, prefs.package_caching_strategy.clone());
		Self {
			users: UserManager::new(ClientId::new("".into())),
			instances: InstanceRegistry::new(),
			profiles: HashMap::new(),
			packages,
			preferences: prefs,
			global_packages: Vec::new(),
			default_user: None,
		}
	}

	/// Create a UserBuilder
	pub fn user(&mut self, id: String, name: String, kind: UserBuilderKind) -> UserBuilder {
		UserBuilder::with_parent(id, name, kind, Some(self))
	}

	/// Finish a UserBuilder
	fn build_user(&mut self, user: User) {
		self.users.add_user(user);
	}

	/// Create a ProfileBuilder
	pub fn profile(&mut self, id: ProfileID, version: MinecraftVersionDeser) -> ProfileBuilder {
		ProfileBuilder::with_parent(id, version, Some(self))
	}

	/// Finish a ProfileBuilder
	fn build_profile(&mut self, id: ProfileID, profile: Profile, instances: InstanceRegistry) {
		self.instances.extend(instances);
		self.profiles.insert(id, profile);
	}

	/// Create a PackageBuilder
	pub fn package(
		&mut self,
		data: InitialPackageData,
	) -> PackageBuilder<PackageBuilderConfigParent<'_>> {
		let parent = PackageBuilderConfigParent(self);
		PackageBuilder::with_parent(data, parent)
	}

	/// Finish a PackageBuilder
	fn build_package(&mut self, package: FullPackageConfig) {
		let config = PackageConfig::Full(package);
		self.global_packages.push(config);
	}

	/// Set the default user
	pub fn default_user(&mut self, user_id: String) -> &mut Self {
		self.default_user = Some(user_id);

		self
	}

	/// Finishes the builder
	pub fn build(mut self) -> anyhow::Result<Config> {
		if let Some(default_user_id) = &self.default_user {
			if self.users.user_exists(&default_user_id) {
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
			profiles: self.profiles,
			packages: self.packages,
			global_packages: self.global_packages,
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
	pub fn new(id: String, name: String, kind: UserBuilderKind) -> Self {
		Self::with_parent(id, name, kind, None)
	}

	/// Construct with a parent
	fn with_parent(
		id: String,
		name: String,
		kind: UserBuilderKind,
		parent: Option<&'parent mut ConfigBuilder>,
	) -> Self {
		let variant = match kind {
			UserBuilderKind::Microsoft => UserVariant::Microsoft { uuid: None },
			UserBuilderKind::Demo => UserVariant::Demo { uuid: None },
			UserBuilderKind::Unverified => UserVariant::Unverified {},
		};
		Self {
			id,
			config: UserConfig { name, variant },
			parent,
		}
	}

	/// Fill the UUID of the user if it supports it
	pub fn uuid(&mut self, uuid: String) -> &mut Self {
		match &mut self.config.variant {
			UserVariant::Microsoft { uuid: uuid_to_set }
			| UserVariant::Demo { uuid: uuid_to_set } => *uuid_to_set = Some(uuid),
			_ => {}
		}
		self
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
	/// An unverified user
	Unverified,
}

/// Builder for a profile
pub struct ProfileBuilder<'parent> {
	id: ProfileID,
	config: ProfileConfig,
	instances: HashMap<InstanceID, InstanceConfig>,
	parent: Option<&'parent mut ConfigBuilder>,
}

impl<'parent> ProfileBuilder<'parent> {
	/// Construct a new ProfileBuilder
	pub fn new(id: ProfileID, version: MinecraftVersionDeser) -> Self {
		Self::with_parent(id, version, None)
	}

	/// Construct with a parent
	fn with_parent(
		id: ProfileID,
		version: MinecraftVersionDeser,
		parent: Option<&'parent mut ConfigBuilder>,
	) -> Self {
		let config = ProfileConfig {
			version,
			modloader: Modloader::Vanilla,
			client_type: ClientType::None,
			server_type: ServerType::None,
			instances: HashMap::new(),
			packages: ProfilePackageConfiguration::Full {
				global: Vec::new(),
				client: Vec::new(),
				server: Vec::new(),
			},
			package_stability: PackageStability::default(),
		};

		Self {
			id,
			config,
			instances: HashMap::new(),
			parent,
		}
	}

	/// Create an InstanceBuilder
	pub fn instance<'this>(
		&'this mut self,
		id: InstanceID,
		side: Side,
	) -> InstanceBuilder<'this, 'parent> {
		InstanceBuilder::with_parent(id, side, Some(self))
	}

	/// Finish an InstanceBuilder
	fn build_instance(&mut self, id: InstanceID, instance: FullInstanceConfig) {
		self.instances.insert(id, InstanceConfig::Full(instance));
	}

	/// Create a PackageBuilder
	pub fn package<'this>(
		&'this mut self,
		group: ProfilePackageGroup,
		data: InitialPackageData,
	) -> PackageBuilder<PackageBuilderProfileParent<'this, 'parent>> {
		let parent = PackageBuilderProfileParent(group, self);
		PackageBuilder::with_parent(data, parent)
	}

	/// Finish a PackageBuilder
	fn build_package(&mut self, group: ProfilePackageGroup, package: FullPackageConfig) {
		let config = PackageConfig::Full(package);
		match group {
			ProfilePackageGroup::Global => self.config.packages.add_global_package(config),
			ProfilePackageGroup::Client => self.config.packages.add_client_package(config),
			ProfilePackageGroup::Server => self.config.packages.add_server_package(config),
		}
	}

	/// Set the modloader of the profile
	pub fn modloader(&mut self, modloader: Modloader) -> &mut Self {
		self.config.modloader = modloader;
		self
	}

	/// Set the client type of the profile
	pub fn client_type(&mut self, client_type: ClientType) -> &mut Self {
		self.config.client_type = client_type;
		self
	}

	/// Set the server type of the profile
	pub fn server_type(&mut self, server_type: ServerType) -> &mut Self {
		self.config.server_type = server_type;
		self
	}

	/// Set the default package stability of the profile
	pub fn package_stability(&mut self, package_stability: PackageStability) -> &mut Self {
		self.config.package_stability = package_stability;
		self
	}

	/// Finish the builder and go to the parent
	pub fn build(self) -> anyhow::Result<()> {
		let (id, profile, instances, parent) = self.build_self()?;
		if let Some(parent) = parent {
			parent.build_profile(id, profile, instances);
		}

		Ok(())
	}

	/// Finish the builder and return the self
	pub fn build_self(
		self,
	) -> anyhow::Result<(
		ProfileID,
		Profile,
		InstanceRegistry,
		Option<&'parent mut ConfigBuilder>,
	)> {
		let mut built = self.config.to_profile(self.id.clone());
		let mut new_map = HashMap::new();
		for (id, instance) in self.instances {
			built.instances.push(id.clone());
			let instance =
				read_instance_config(self.id.clone(), &instance, &built, &HashMap::new())?;
			new_map.insert(id, instance);
		}

		Ok((self.id, built, new_map, self.parent))
	}
}

/// Builder for an instance
pub struct InstanceBuilder<'parent, 'grandparent> {
	id: InstanceID,
	config: FullInstanceConfig,
	parent: Option<&'parent mut ProfileBuilder<'grandparent>>,
}

impl<'parent, 'grandparent> InstanceBuilder<'parent, 'grandparent> {
	/// Construct a new InstanceBuilder
	pub fn new(id: InstanceID, side: Side) -> Self {
		Self::with_parent(id, side, None)
	}

	/// Construct with a parent
	fn with_parent(
		id: InstanceID,
		side: Side,
		parent: Option<&'parent mut ProfileBuilder<'grandparent>>,
	) -> Self {
		let config = match side {
			Side::Client => FullInstanceConfig::Client {
				launch: Default::default(),
				options: Default::default(),
				window: Default::default(),
				preset: Default::default(),
				datapack_folder: Default::default(),
				snapshots: Default::default(),
				packages: Default::default(),
			},
			Side::Server => FullInstanceConfig::Server {
				launch: Default::default(),
				options: Default::default(),
				preset: Default::default(),
				datapack_folder: Default::default(),
				snapshots: Default::default(),
				packages: Default::default(),
			},
		};

		Self { id, config, parent }
	}

	/// Create a PackageBuilder
	pub fn package<'this>(
		&'this mut self,
		data: InitialPackageData,
	) -> PackageBuilder<PackageBuilderInstanceParent<'this, 'parent, 'grandparent>> {
		let parent = PackageBuilderInstanceParent(self);
		PackageBuilder::with_parent(data, parent)
	}

	/// Finish a PackageBuilder
	fn build_package(&mut self, package: FullPackageConfig) {
		let config = PackageConfig::Full(package);
		match &mut self.config {
			FullInstanceConfig::Client { packages, .. } => packages.push(config),
			FullInstanceConfig::Server { packages, .. } => packages.push(config),
		};
	}

	/// Set the launch options of the instance
	pub fn launch_options(&mut self, launch_options: LaunchConfig) -> &mut Self {
		match &mut self.config {
			FullInstanceConfig::Client { launch, .. } => *launch = launch_options,
			FullInstanceConfig::Server { launch, .. } => *launch = launch_options,
		};

		self
	}

	/// Set the client window config of the instance
	pub fn window_config(&mut self, window_config: ClientWindowConfig) -> &mut Self {
		match &mut self.config {
			FullInstanceConfig::Client { window, .. } => *window = window_config,
			FullInstanceConfig::Server { .. } => {}
		};

		self
	}

	/// Set the datapack folder of the instance
	pub fn datapack_folder(&mut self, folder: String) -> &mut Self {
		match &mut self.config {
			FullInstanceConfig::Client {
				datapack_folder, ..
			} => *datapack_folder = Some(folder),
			FullInstanceConfig::Server {
				datapack_folder, ..
			} => *datapack_folder = Some(folder),
		};

		self
	}

	/// Set the snapshot config of the instance
	pub fn snapshot_config(&mut self, snapshot_config: snapshot::Config) -> &mut Self {
		match &mut self.config {
			FullInstanceConfig::Client { snapshots, .. } => *snapshots = Some(snapshot_config),
			FullInstanceConfig::Server { snapshots, .. } => *snapshots = Some(snapshot_config),
		};

		self
	}

	/// Finish the builder and go to the parent
	pub fn build(self) {
		if let Some(parent) = self.parent {
			parent.build_instance(self.id, self.config);
		}
	}

	/// Finish the builder and return the self
	pub fn build_self(
		self,
		profile: &Profile,
	) -> anyhow::Result<(
		InstanceID,
		Instance,
		Option<&'parent mut ProfileBuilder<'grandparent>>,
	)> {
		let built = read_instance_config(
			self.id.clone(),
			&InstanceConfig::Full(self.config),
			profile,
			&HashMap::new(),
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
		let config = match data {
			InitialPackageData::Local {
				id,
				path,
				content_type,
			} => FullPackageConfig::Local {
				r#type: PackageType::Local,
				id,
				content_type,
				path,
				features: Default::default(),
				use_default_features: true,
				permissions: Default::default(),
				stability: Default::default(),
			},
			InitialPackageData::Repository { id } => FullPackageConfig::Repository {
				id,
				features: Default::default(),
				use_default_features: true,
				permissions: Default::default(),
				stability: Default::default(),
			},
		};

		Self { config, parent }
	}

	/// Add to the package's features
	pub fn features(&mut self, features: Vec<String>) -> &mut Self {
		let other_features = features;
		match &mut self.config {
			FullPackageConfig::Local { features, .. } => features.extend(other_features),
			FullPackageConfig::Repository { features, .. } => features.extend(other_features),
		}
		self
	}

	/// Set the use_default_features setting of the package
	pub fn use_default_features(&mut self, value: bool) -> &mut Self {
		match &mut self.config {
			FullPackageConfig::Local {
				use_default_features,
				..
			} => *use_default_features = value,
			FullPackageConfig::Repository {
				use_default_features,
				..
			} => *use_default_features = value,
		}
		self
	}

	/// Set the permissions of the package
	pub fn permissions(&mut self, permissions: EvalPermissions) -> &mut Self {
		let other_permissions = permissions;
		match &mut self.config {
			FullPackageConfig::Local { permissions, .. } => *permissions = other_permissions,
			FullPackageConfig::Repository { permissions, .. } => *permissions = other_permissions,
		}
		self
	}

	/// Set the configured stability of the package
	pub fn stability(&mut self, stability: PackageStability) -> &mut Self {
		let other_stability = stability;
		match &mut self.config {
			FullPackageConfig::Local { stability, .. } => *stability = Some(other_stability),
			FullPackageConfig::Repository { stability, .. } => *stability = Some(other_stability),
		}
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
pub enum InitialPackageData {
	/// A local package
	Local {
		/// The ID of the pcakage
		id: PackageID,
		/// The path to the local package
		path: String,
		/// The content type of the package
		content_type: PackageContentType,
	},
	/// A repository package
	Repository {
		/// The ID of the package
		id: PackageID,
	},
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

/// Data for a PackageBuilder that returns to a ProfileBuilder
pub struct PackageBuilderProfileParent<'profile, 'parent>(
	ProfilePackageGroup,
	&'profile mut ProfileBuilder<'parent>,
);

/// The different package groups that a PackageBuilder can return to
pub enum ProfilePackageGroup {
	/// Global
	Global,
	/// Client sided
	Client,
	/// Server sided
	Server,
}

impl<'profile, 'parent> PackageBuilderParent for PackageBuilderProfileParent<'profile, 'parent> {
	fn build_package(self, package: FullPackageConfig) {
		self.1.build_package(self.0, package)
	}
}

/// Data for a PackageBuilder that returns to a InstanceBuilder
pub struct PackageBuilderInstanceParent<'instance, 'parent, 'grandparent>(
	&'instance mut InstanceBuilder<'parent, 'grandparent>,
);

impl<'instance, 'parent, 'grandparent> PackageBuilderParent
	for PackageBuilderInstanceParent<'instance, 'parent, 'grandparent>
{
	fn build_package(self, package: FullPackageConfig) {
		self.0.build_package(package)
	}
}

/// Data for a PackageBuilder that returns to a ConfigBuilder
pub struct PackageBuilderConfigParent<'config>(&'config mut ConfigBuilder);

impl<'config> PackageBuilderParent for PackageBuilderConfigParent<'config> {
	fn build_package(self, package: FullPackageConfig) {
		self.0.build_package(package)
	}
}

#[cfg(test)]
mod tests {
	use mcvm_shared::lang::Language;

	use crate::data::config::preferences::{PrefDeser, RepositoriesDeser};
	use crate::package::reg::CachingStrategy;

	use super::*;

	#[test]
	fn test_config_building() {
		let (prefs, repos) = get_prefs().expect("Failed to get preferences");
		let mut config = ConfigBuilder::new(prefs, repos);
		let mut profile = config.profile(
			"profile".into(),
			MinecraftVersionDeser::Version("1.19.3".into()),
		);
		modify_profile(&mut profile);
		config
			.package(InitialPackageData::Repository {
				id: "global-package".into(),
			})
			.build();
		config
			.user("user".into(), "User".into(), UserBuilderKind::Microsoft)
			.build();
		config.default_user("user".into());
		let config = config.build().expect("Failed to build config");
		assert!(config.users.user_exists("user"));
		assert_eq!(
			config.users.get_chosen_user().map(|x| x.get_id().clone()),
			Some("user".into())
		);
	}

	#[test]
	fn test_profile_building() {
		let mut profile = ProfileBuilder::new(
			"profile".into(),
			MinecraftVersionDeser::Version("1.19.3".into()),
		);
		modify_profile(&mut profile);

		let (profile_id, profile, instances, ..) =
			profile.build_self().expect("Failed to build profile");
		assert_eq!(profile_id, "profile".into());
		assert!(instances.contains_key("instance"));
		assert_eq!(profile.instances, vec!["instance".into()]);
		assert_eq!(profile.modifications.client_type, ClientType::Fabric);
	}

	fn modify_profile(profile: &mut ProfileBuilder<'_>) {
		let mut instance = profile.instance("instance".into(), Side::Client);
		let package = instance.package(InitialPackageData::Repository {
			id: "instance-package".into(),
		});
		package.build();
		instance.launch_options(LaunchConfig::default());
		instance.build();
		profile.client_type(ClientType::Fabric);
		let mut package = profile.package(
			ProfilePackageGroup::Global,
			InitialPackageData::Repository {
				id: "profile-package".into(),
			},
		);
		package.features(vec!["hello".into(), "goodbye".into()]);
		package.build();
	}

	fn get_prefs() -> anyhow::Result<(ConfigPreferences, Vec<PkgRepo>)> {
		let deser = PrefDeser {
			repositories: RepositoriesDeser::default(),
			package_caching_strategy: CachingStrategy::default(),
			language: Language::default(),
		};
		ConfigPreferences::read(&deser)
	}
}
