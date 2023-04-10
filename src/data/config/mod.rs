pub mod instance;
pub mod package;
mod preferences;
pub mod profile;

use self::instance::read_instance_config;
use self::package::{read_package_config, FullPackageConfig, PackageConfig};
use self::profile::parse_profile_config;
use anyhow::{anyhow, bail, Context};
use preferences::ConfigPreferences;

use super::addon::game_modifications_compatible;
use super::profile::{InstanceRegistry, Profile};
use super::user::{validate_username, Auth, AuthState, User, UserKind};
use crate::package::reg::PkgRegistry;
use crate::util::json::{self, JsonType};
use crate::util::validate_identifier;

use color_print::cprintln;
use serde_json::json;

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// Default program configuration
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

#[derive(Debug)]
pub struct Config {
	pub auth: Auth,
	pub instances: InstanceRegistry,
	pub profiles: HashMap<String, Box<Profile>>,
	pub packages: PkgRegistry,
	pub prefs: ConfigPreferences,
}

impl Config {
	fn open(path: &PathBuf) -> anyhow::Result<Box<json::JsonObject>> {
		if path.exists() {
			let doc = json::parse_object(&fs::read_to_string(path)?)?;
			Ok(doc)
		} else {
			let doc = default_config();
			fs::write(path, serde_json::to_string_pretty(&doc)?)?;
			Ok(Box::new(
				json::ensure_type(doc.as_object(), JsonType::Obj)?.clone(),
			))
		}
	}

	fn load_from_obj(obj: &json::JsonObject) -> anyhow::Result<Self> {
		let mut auth = Auth::new();
		let mut instances = InstanceRegistry::new();
		let mut profiles = HashMap::new();
		// Preferences
		let (prefs, repositories) = ConfigPreferences::read(obj.get("preferences"))
			.context("Failed to read preferences")?;

		let mut packages = PkgRegistry::new(repositories);

		// Users
		let users = json::access_object(obj, "users")?;
		for (user_id, user_val) in users.iter() {
			if !validate_identifier(user_id) {
				bail!("Invalid string '{}'", user_id.to_owned());
			}
			let user_obj = json::ensure_type(user_val.as_object(), JsonType::Obj)?;
			let kind = match json::access_str(user_obj, "type")? {
				"microsoft" => Ok(UserKind::Microsoft),
				"demo" => Ok(UserKind::Demo),
				"unverified" => Ok(UserKind::Unverified),
				typ => Err(anyhow!("Unknown user type '{typ}' on user '{user_id}'")),
			}?;
			let username = json::access_str(user_obj, "name")?;
			if !validate_username(kind.clone(), username) {
				bail!("Invalid string '{}'", username.to_owned());
			}
			let mut user = User::new(kind.clone(), user_id, username);

			match user_obj.get("uuid") {
				Some(uuid) => user.set_uuid(json::ensure_type(uuid.as_str(), JsonType::Str)?),
				None => match kind {
					UserKind::Microsoft | UserKind::Demo => {
						cprintln!("<y>Warning: It is recommended to have your uuid in the configuration for user {}", user_id);
					}
					UserKind::Unverified => {}
				}
			};

			auth.users.insert(user_id.to_string(), user);
		}

		if let Some(user_val) = obj.get("default_user") {
			let user_id = json::ensure_type(user_val.as_str(), JsonType::Str)?.to_string();
			match auth.users.get(&user_id) {
				Some(..) => auth.state = AuthState::Authed(user_id),
				None => {
					bail!("Provided default user '{user_id}' does not exist");
				}
			}
		} else if users.is_empty() {
			cprintln!("<y>Warning: Users are available but no default user is set.");
		} else {
			cprintln!("<y>Warning: No users are available.");
		}

		// Profiles
		let doc_profiles = json::access_object(obj, "profiles")?;
		for (profile_id, profile_val) in doc_profiles {
			let profile_config = parse_profile_config(profile_val)
				.with_context(|| format!("Failed to parse profile {profile_id}"))?;
			let mut profile = profile_config.to_profile(profile_id);

			if !game_modifications_compatible(
				&profile_config.modloader,
				&profile_config.plugin_loader,
			) {
				bail!("Modloader and Plugin Loader are incompatible for profile {profile_id}");
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
		Self::load_from_obj(&obj)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_default_config() {
		let obj = json::ensure_type(default_config().as_object(), JsonType::Obj)
			.unwrap()
			.clone();
		Config::load_from_obj(&obj).unwrap();
	}
}
