pub mod instance;
pub mod modifications;
pub mod package;
pub mod preferences;
pub mod profile;
pub mod user;

use self::instance::{read_instance_config, InstanceConfig};
use self::package::{FullPackageConfig, PackageConfig};
use self::preferences::PrefDeser;
use self::profile::ProfileConfig;
use self::user::UserConfig;
use anyhow::{bail, ensure, Context};
use mcvm_shared::modifications::Modloader;
use preferences::ConfigPreferences;
use serde::{Deserialize, Serialize};

use super::profile::{InstanceRegistry, Profile};
use super::user::{validate_username, Auth, AuthState};
use crate::io::files::paths::Paths;
use crate::package::reg::PkgRegistry;
use crate::util::validate_identifier;

use color_print::cprintln;
use serde_json::json;

use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

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

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct ConfigDeser {
	users: HashMap<String, UserConfig>,
	default_user: Option<String>,
	profiles: HashMap<String, ProfileConfig>,
	instance_presets: HashMap<String, InstanceConfig>,
	preferences: PrefDeser,
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
	/// Get the config path
	pub fn get_path(paths: &Paths) -> PathBuf {
		paths.project.config_dir().join("mcvm.json")
	}

	/// Open the config from a file
	fn open(path: &Path) -> anyhow::Result<ConfigDeser> {
		if path.exists() {
			let mut file = File::open(path).context("Failed to open config file")?;
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
	fn load_from_deser(config: ConfigDeser, show_warnings: bool) -> anyhow::Result<Self> {
		let mut auth = Auth::new();
		let mut instances = InstanceRegistry::new();
		let mut profiles = HashMap::new();
		// Preferences
		let (prefs, repositories) =
			ConfigPreferences::read(&config.preferences).context("Failed to read preferences")?;

		let mut packages = PkgRegistry::new(repositories, prefs.package_caching_strategy.clone());

		// Users
		for (user_id, user_config) in config.users.iter() {
			if !validate_identifier(user_id) {
				bail!("Invalid string '{user_id}'");
			}
			let user = user_config.to_user(user_id, show_warnings);
			if !validate_username(user.kind, &user.name) {
				bail!("Invalid string '{}'", user.name);
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
		} else if config.users.is_empty() && show_warnings {
			cprintln!("<y>Warning: Users are available but no default user is set.");
		} else if show_warnings {
			cprintln!("<y>Warning: No users are available.");
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
			let mut profile = profile_config.to_profile(&profile_id);

			if let Modloader::Forge = profile_config.modloader {
				if show_warnings {
					cprintln!("<y>Warning: Forge installation is currently unimplemented by mcvm. You will be expected to install it yourself for the time being.");
				}
			}

			if profile_config.instances.is_empty() && show_warnings {
				cprintln!(
					"<y>Warning: Profile '{}' does not have any instances",
					profile_id
				);
			}

			for (instance_id, instance_config) in profile_config.instances {
				if !validate_identifier(&instance_id) {
					bail!("Invalid string '{}'", instance_id.to_owned());
				}
				if instances.contains_key(&instance_id) {
					bail!("Duplicate instance '{instance_id}'");
				}
				let instance = read_instance_config(
					&instance_id,
					&instance_config,
					&profile,
					&config.instance_presets,
				)
				.with_context(|| format!("Failed to configure instance '{instance_id}'"))?;
				profile.add_instance(&instance_id);
				instances.insert(instance_id.to_string(), instance);
			}

			for package_config in profile_config.packages {
				let config = package_config
					.to_profile_config(profile_config.package_stability)
					.with_context(|| format!("Failed to configure package '{package_config}'"))?;

				if !validate_identifier(&config.req.name) {
					bail!("Invalid package name '{package_config}'");
				}

				for cfg in profile.packages.iter() {
					if cfg.req == config.req {
						bail!("Duplicate package '{package_config}' in profile '{profile_id}'");
					}
				}

				for feature in &config.features {
					if !validate_identifier(feature) {
						bail!("Invalid string '{feature}'");
					}
				}

				if let PackageConfig::Full(FullPackageConfig::Local {
					id: _,
					version,
					path,
					..
				}) = package_config
				{
					let path = shellexpand::tilde(&path);
					packages.insert_local(&config.req, version, &PathBuf::from(path.to_string()));
				}

				profile.packages.push(config);
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

	pub fn load(path: &Path, show_warnings: bool) -> anyhow::Result<Self> {
		let obj = Self::open(path)?;
		Self::load_from_deser(obj, show_warnings)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_default_config() {
		let deser = serde_json::from_value(default_config()).unwrap();
		Config::load_from_deser(deser, true).unwrap();
	}
}
