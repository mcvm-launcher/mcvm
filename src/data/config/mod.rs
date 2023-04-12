pub mod instance;
pub mod package;
mod preferences;
pub mod profile;
pub mod user;

use self::instance::read_instance_config;
use self::package::{read_package_config, FullPackageConfig, PackageConfig};
use self::preferences::PrefDeser;
use self::profile::ProfileConfig;
use self::user::{read_user_config, UserConfig};
use anyhow::{bail, Context};
use preferences::ConfigPreferences;
use serde::Deserialize;

use super::addon::game_modifications_compatible;
use super::profile::{InstanceRegistry, Profile};
use super::user::{validate_username, Auth, AuthState};
use crate::package::reg::PkgRegistry;
use crate::util::validate_identifier;

use color_print::cprintln;
use serde_json::json;

use std::collections::HashMap;
use std::fs::File;
use std::path::{PathBuf, Path};

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
					"instances": {
						"example-client": {
							"type": "client"
						},
						"example-server": {
							"type": "server"
						}
					}
				}
			}
		}
	)
}

#[derive(Deserialize)]
pub struct ConfigDeser {
	#[serde(default)]
	users: HashMap<String, UserConfig>,
	#[serde(default)]
	default_user: Option<String>,
	#[serde(default)]
	profiles: HashMap<String, ProfileConfig>,
	#[serde(default)]
	preferences: Option<PrefDeser>,
}

#[derive(Debug)]
pub struct Config {
	pub auth: Auth,
	pub instances: InstanceRegistry,
	pub profiles: HashMap<String, Box<Profile>>,
	pub packages: PkgRegistry,
	pub prefs: ConfigPreferences,
}

impl Config {
	/// Open the config from a file
	fn open(path: &Path) -> anyhow::Result<ConfigDeser> {
		if path.exists() {
			let mut file = File::open(path).context("Failed to open config file")?;
			Ok(serde_json::from_reader(&mut file).context("Failed to parse config")?)
		} else {
			let doc = default_config();
			let mut file = File::open(path).context("Failed to open config file")?;
			serde_json::to_writer_pretty(&mut file, &doc)
				.context("Failed to write default configuration")?;
			Ok(serde_json::from_value(doc).context("Failed to parse default configuration")?)
		}
	}

	fn load_from_deser(config: ConfigDeser) -> anyhow::Result<Self> {
		let mut auth = Auth::new();
		let mut instances = InstanceRegistry::new();
		let mut profiles = HashMap::new();
		// Preferences
		let (prefs, repositories) = ConfigPreferences::read(&config.preferences)
			.context("Failed to read preferences")?;


		let mut packages = PkgRegistry::new(repositories);

		// Users
		for (user_id, user_val) in config.users.iter() {
			if !validate_identifier(user_id) {
				bail!("Invalid string '{}'", user_id.to_owned());
			}
			let user = read_user_config(user_id, user_val);
			if !validate_username(user.kind, &user.name) {
				bail!("Invalid string '{}'", user.name.to_owned());
			}

			auth.users.insert(user_id.to_string(), user);
		}

		if let Some(default_user_id) = &config.default_user {
			match auth.users.get(default_user_id) {
				Some(..) => auth.state = AuthState::Authed(default_user_id.clone()),
				None => {
					bail!("Provided default user '{default_user_id}' does not exist");
				}
			}
		} else if config.users.is_empty() {
			cprintln!("<y>Warning: Users are available but no default user is set.");
		} else {
			cprintln!("<y>Warning: No users are available.");
		}

		// Profiles
		for (profile_id, profile_config) in config.profiles {
			let mut profile = profile_config.to_profile(&profile_id);

			if !game_modifications_compatible(
				&profile_config.modloader,
				&profile_config.plugin_loader,
			) {
				bail!("Modloader and Plugin Loader are incompatible for profile {profile_id}");
			}

			if profile_config.instances.is_empty() {
				cprintln!("<y>Warning: Profile '{}' does not have any instances", profile_id);
			}

			for (instance_id, instance) in profile_config.instances {
				if !validate_identifier(&instance_id) {
					bail!("Invalid string '{}'", instance_id.to_owned());
				}
				if instances.contains_key(&instance_id) {
					bail!("Duplicate instance '{instance_id}'");
				}
				let instance = read_instance_config(&instance_id, &instance, &profile)
					.with_context(|| format!("Failed to configure instance '{instance_id}'"))?;
				profile.add_instance(&instance_id);
				instances.insert(instance_id.to_string(), instance);
			}

			for package in profile_config.packages {
				let config = read_package_config(&package)
					.with_context(|| format!("Failed to configure package '{}'", package))?;

				if !validate_identifier(&config.req.name) {
					bail!("Invalid package name '{package}'");
				}

				for cfg in profile.packages.iter() {
					if cfg.req == config.req {
						bail!("Duplicate package '{package}' in profile '{profile_id}'");
					}
				}

				for feature in config.features {
					if !validate_identifier(&feature) {
						bail!("Invalid string '{feature}'");
					}
				}

				match package {
					PackageConfig::Full(FullPackageConfig::Local {
						id: _,
						version,
						path,
						..
					}) => {
						let path = shellexpand::tilde(&path);
						packages.insert_local(
							&config.req,
							&version,
							&PathBuf::from(path.to_string()),
						);
					}
					_ => {}
				}
			}

			profiles.insert(profile_id.clone(), Box::new(profile));
		}

		Ok(Self {
			auth,
			instances,
			profiles,
			packages,
			prefs,
		})
	}

	pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
		let obj = Self::open(path)?;
		Self::load_from_deser(obj)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_default_config() {
		let deser = serde_json::from_value(default_config()).unwrap();
		Config::load_from_deser(deser).unwrap();
	}
}
