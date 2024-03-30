/// Easy creation of config
pub mod builder;
/// Configuring instances
pub mod instance;
/// Configuring profile modifications
pub mod modifications;
/// Configuring packages
pub mod package;
/// Configuring global preferences
pub mod preferences;
/// Configuring profiles
pub mod profile;
/// Configuring users
pub mod user;

use self::instance::{read_instance_config, InstanceConfig};
use self::package::PackageConfig;
use self::preferences::PrefDeser;
use self::profile::ProfileConfig;
use self::user::UserConfig;
use anyhow::{bail, ensure, Context};
use mcvm_core::user::UserManager;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::util::is_valid_identifier;
use oauth2::ClientId;
use preferences::ConfigPreferences;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::id::ProfileID;
use super::profile::{InstanceRegistry, Profile};
use crate::io::files::paths::Paths;
use crate::pkg::reg::PkgRegistry;

use serde_json::json;

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

/// The data resulting from reading configuration.
/// Represents all of the configured data that MCVM will use
#[derive(Debug)]
pub struct Config {
	/// The user manager
	pub users: UserManager,
	/// The available instances
	pub instances: InstanceRegistry,
	/// The available profiles
	pub profiles: HashMap<ProfileID, Profile>,
	/// The registry of packages. Will include packages that are configured when created this way
	pub packages: PkgRegistry,
	/// Globally configured packages to include in every profile
	pub global_packages: Vec<PackageConfig>,
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
	profiles: HashMap<ProfileID, ProfileConfig>,
	instance_presets: HashMap<String, InstanceConfig>,
	packages: Vec<PackageConfig>,
	preferences: PrefDeser,
}

impl Config {
	/// Get the config path
	pub fn get_path(paths: &Paths) -> PathBuf {
		paths.project.config_dir().join("mcvm.json")
	}

	/// Open the config from a file
	fn open(path: &Path) -> anyhow::Result<ConfigDeser> {
		if path.exists() {
			let file = File::open(path).context("Failed to open config file")?;
			let mut file = BufReader::new(file);
			Ok(serde_json::from_reader(&mut file).context("Failed to parse config")?)
		} else {
			let doc = default_config();
			let mut file = File::create(path).context("Failed to open config file")?;
			serde_json::to_writer_pretty(&mut file, &doc)
				.context("Failed to write default configuration")?;
			Ok(serde_json::from_value(doc).context("Failed to parse default configuration")?)
		}
	}

	/// Create the Config struct from deserialized config
	fn load_from_deser(
		config: ConfigDeser,
		show_warnings: bool,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Self> {
		let mut users = UserManager::new(ClientId::new("".into()));
		let mut instances = InstanceRegistry::new();
		let mut profiles = HashMap::new();
		// Preferences
		let (prefs, repositories) =
			ConfigPreferences::read(&config.preferences).context("Failed to read preferences")?;

		let packages = PkgRegistry::new(repositories, prefs.package_caching_strategy.clone());

		// Users
		for (user_id, user_config) in config.users.iter() {
			if !is_valid_identifier(user_id) {
				bail!("Invalid string '{user_id}'");
			}
			let user = user_config.to_user(user_id);
			// Disabled until we can verify game ownership.
			// We don't want to be a cracked launcher.
			if user.is_unverified() || user.is_demo() {
				bail!("Unverified and Demo users are currently disabled");
			}
			if !user.validate_username() {
				bail!("Invalid string '{}'", user.get_name());
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
				MessageContents::Warning("Users are available but no default user is set".into()),
				MessageLevel::Important,
			);
		} else if show_warnings {
			o.display(
				MessageContents::Warning("No users are available".into()),
				MessageLevel::Important,
			);
		}

		// Validate instance presets
		for (id, preset) in &config.instance_presets {
			ensure!(
				!preset.uses_preset(),
				"Instance preset '{id}' cannot use another preset"
			);
		}

		// Profiles
		for (profile_id, profile_config) in config.profiles {
			let mut profile = profile_config.to_profile(profile_id.clone());

			if show_warnings && !profile::can_install_client_type(profile.modifications.client_type)
			{
				o.display(
					MessageContents::Warning(
						format!("{} installation on the client is currently unimplemented by mcvm. You will be expected to install it yourself for the time being", profile.modifications.client_type),
					),
					MessageLevel::Important,
				);
			}

			if show_warnings && !profile::can_install_server_type(profile.modifications.server_type)
			{
				o.display(
					MessageContents::Warning(
						format!("{} installation on the server is currently unimplemented by mcvm. You will be expected to install it yourself for the time being", profile.modifications.client_type),
					),
					MessageLevel::Important,
				);
			}

			if profile_config.instances.is_empty() && show_warnings {
				o.display(
					MessageContents::Warning(format!(
						"Profile '{profile_id}' does not have any instances"
					)),
					MessageLevel::Important,
				);
			}

			for (instance_id, instance_config) in profile_config.instances {
				if !is_valid_identifier(&instance_id) {
					bail!("Invalid string '{}'", instance_id.to_string());
				}
				if instances.contains_key(&instance_id) {
					bail!("Duplicate instance '{instance_id}'");
				}
				let instance = read_instance_config(
					instance_id.clone(),
					&instance_config,
					&profile,
					&config.packages,
					&config.instance_presets,
				)
				.with_context(|| format!("Failed to configure instance '{instance_id}'"))?;
				profile.add_instance(instance_id.clone());
				instances.insert(instance_id, instance);
			}

			profile_config.packages.validate()?;

			profiles.insert(profile_id.clone(), profile);
		}

		Ok(Self {
			users,
			instances,
			profiles,
			packages,
			global_packages: config.packages,
			prefs,
		})
	}

	/// Load the configuration from the config file
	pub fn load(path: &Path, show_warnings: bool, o: &mut impl MCVMOutput) -> anyhow::Result<Self> {
		let obj = Self::open(path)?;
		Self::load_from_deser(obj, show_warnings, o)
	}
}

/// Default program configuration
fn default_config() -> serde_json::Value {
	json!(
		{
			"users": {
				"example": {
					"type": "microsoft",
					"name": "ExampleUser441"
				}
			},
			"default_user": "example",
			"profiles": {
				"example": {
					"version": "1.19.3",
					"server_type": "paper",
					"instances": {
						"example-client": {
							"type": "client"
						},
						"example-server": "server"
					}
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
			true,
			&mut output::Simple(output::MessageLevel::Debug),
		)
		.unwrap();
	}
}
