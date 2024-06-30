/// Easy programatic creation of config
#[cfg(feature = "builder")]
pub mod builder;
/// Configuring instances
pub mod instance;
/// Configuring profile modifications
pub mod modifications;
/// Configuring packages
pub mod package;
/// Configuring plugins
pub mod plugin;
/// Configuring global preferences
pub mod preferences;
/// Configuring profiles
pub mod profile;
/// Configuring users
pub mod user;

use self::instance::{read_instance_config, InstanceConfig};
use self::plugin::PluginManager;
use self::preferences::PrefDeser;
use self::profile::ProfileConfig;
use self::user::UserConfig;
use anyhow::{bail, Context};
use mcvm_core::auth_crate::mc::ClientId;
use mcvm_core::io::{json_from_file, json_to_file_pretty};
use mcvm_core::user::UserManager;
use mcvm_shared::id::{InstanceID, ProfileID};
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::translate;
use mcvm_shared::util::is_valid_identifier;
use preferences::ConfigPreferences;
use profile::consolidate_profile_configs;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::instance::Instance;
use crate::io::paths::Paths;
use crate::pkg::reg::PkgRegistry;

use serde_json::json;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// The data resulting from reading configuration.
/// Represents all of the configured data that MCVM will use
pub struct Config {
	/// The user manager
	pub users: UserManager,
	/// Instances
	pub instances: HashMap<InstanceID, Instance>,
	/// Named groups of instances
	pub instance_groups: HashMap<Arc<str>, Vec<InstanceID>>,
	/// The registry of packages. Will include packages that are configured when created this way
	pub packages: PkgRegistry,
	/// Configured plugins
	pub plugins: PluginManager,
	/// Global user preferences
	pub prefs: ConfigPreferences,
}

/// Deserialization struct for user configuration
#[derive(Deserialize, Serialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct ConfigDeser {
	users: HashMap<String, UserConfig>,
	default_user: Option<String>,
	instances: HashMap<InstanceID, InstanceConfig>,
	instance_groups: HashMap<Arc<str>, Vec<InstanceID>>,
	profiles: HashMap<ProfileID, ProfileConfig>,
	global_profile: Option<ProfileConfig>,
	preferences: PrefDeser,
}

impl Config {
	/// Get the config path
	pub fn get_path(paths: &Paths) -> PathBuf {
		paths.project.config_dir().join("mcvm.json")
	}

	/// Open the config from a file
	pub fn open(path: &Path) -> anyhow::Result<ConfigDeser> {
		if path.exists() {
			Ok(json_from_file(path).context("Failed to open config")?)
		} else {
			let config = default_config();
			json_to_file_pretty(path, &config).context("Failed to write default configuration")?;
			Ok(serde_json::from_value(config).context("Failed to parse default configuration")?)
		}
	}

	/// Create the default config at the specified path if it does not exist
	pub fn create_default(path: &Path) -> anyhow::Result<()> {
		if !path.exists() {
			let doc = default_config();
			json_to_file_pretty(path, &doc).context("Failed to write default configuration")?;
		}
		Ok(())
	}

	/// Create the Config struct from deserialized config
	fn load_from_deser(
		config: ConfigDeser,
		plugins: PluginManager,
		show_warnings: bool,
		paths: &Paths,
		client_id: ClientId,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Self> {
		let mut users = UserManager::new(client_id);
		let mut instances = HashMap::with_capacity(config.instances.len());
		// Preferences
		let (prefs, repositories) =
			ConfigPreferences::read(&config.preferences).context("Failed to read preferences")?;

		let packages = PkgRegistry::new(repositories, prefs.package_caching_strategy.clone());

		// Users
		for (user_id, user_config) in config.users.iter() {
			if !is_valid_identifier(user_id) {
				bail!("Invalid user ID '{user_id}'");
			}
			let user = user_config.to_user(user_id);
			// Disabled until we can verify game ownership.
			// We don't want to be a cracked launcher.
			if user.is_demo() {
				bail!("Unverified and Demo users are currently disabled");
			}

			users.add_user(user);
		}

		if let Some(default_user_id) = &config.default_user {
			if users.user_exists(default_user_id) {
				users
					.choose_user(default_user_id)
					.expect("Default user should exist");
			} else {
				bail!("Provided default user '{default_user_id}' does not exist");
			}
		} else if config.users.is_empty() && show_warnings {
			o.display(
				MessageContents::Warning(translate!(o, NoDefaultUser)),
				MessageLevel::Important,
			);
		} else if show_warnings {
			o.display(
				MessageContents::Warning(translate!(o, NoUsers)),
				MessageLevel::Important,
			);
		}

		// Consolidate profiles
		let profiles = consolidate_profile_configs(config.profiles, config.global_profile.as_ref())
			.context("Failed to merge profiles")?;

		// Instances
		for (instance_id, instance_config) in config.instances {
			let instance = read_instance_config(
				instance_id.clone(),
				instance_config,
				&profiles,
				&plugins,
				paths,
				o,
			)
			.with_context(|| format!("Failed to read config for instance {instance_id}"))?;

			if show_warnings
				&& !profile::can_install_client_type(&instance.config.modifications.client_type)
			{
				o.display(
					MessageContents::Warning(translate!(
						o,
						ModificationNotSupported,
						"mod" = &format!("{}", instance.config.modifications.client_type)
					)),
					MessageLevel::Important,
				);
			}

			if show_warnings
				&& !profile::can_install_server_type(&instance.config.modifications.server_type)
			{
				o.display(
					MessageContents::Warning(translate!(
						o,
						ModificationNotSupported,
						"mod" = &format!("{}", instance.config.modifications.server_type)
					)),
					MessageLevel::Important,
				);
			}

			instances.insert(instance_id, instance);
		}

		for group in config.instance_groups.keys() {
			if !is_valid_identifier(group) {
				bail!("Invalid ID for group '{group}'");
			}
		}

		Ok(Self {
			users,
			instances,
			instance_groups: config.instance_groups,
			packages,
			plugins,
			prefs,
		})
	}

	/// Load the configuration from the config file
	pub fn load(
		path: &Path,
		plugins: PluginManager,
		show_warnings: bool,
		paths: &Paths,
		client_id: ClientId,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Self> {
		let obj = Self::open(path)?;
		Self::load_from_deser(obj, plugins, show_warnings, paths, client_id, o)
	}
}

/// Default program configuration
fn default_config() -> serde_json::Value {
	json!(
		{
			"users": {
				"example": {
					"type": "microsoft"
				}
			},
			"default_user": "example",
			"profiles": {
				"1.20": {
					"version": "1.19.3",
					"modloader": "vanilla",
					"server_type": "none"
				}
			},
			"instances": {
				"example-client": {
					"from": "1.20",
					"type": "client"
				},
				"example-server": {
					"from": "1.20",
					"type": "server"
				}
			}
		}
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	use mcvm_shared::output;

	#[test]
	fn test_default_config() {
		let deser = serde_json::from_value(default_config()).unwrap();
		Config::load_from_deser(
			deser,
			PluginManager::new(),
			true,
			&Paths::new_no_create().unwrap(),
			ClientId::new(String::new()),
			&mut output::Simple(output::MessageLevel::Debug),
		)
		.unwrap();
	}
}
