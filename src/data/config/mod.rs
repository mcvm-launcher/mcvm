pub mod instance;
mod preferences;

use self::instance::parse_instance_config;
use anyhow::{bail, anyhow, Context};
use preferences::ConfigPreferences;

use super::addon::{game_modifications_compatible, Modloader, PluginLoader};
use super::profile::{InstanceRegistry, Profile};
use super::user::{validate_username, Auth, AuthState, User, UserKind};
use crate::package::eval::eval::EvalPermissions;
use crate::package::reg::{PkgRegistry, PkgRequest};
use crate::package::PkgConfig;
use crate::util::json::{self, JsonType};
use crate::util::validate_identifier;
use crate::util::versions::VersionPattern;

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
			},
			"packages": []
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
				typ => Err(anyhow!("Unknown user type '{typ}' on user '{user_id}'")),
			}?;
			let username = json::access_str(user_obj, "name")?;
			if !validate_username(kind.clone(), username) {
				bail!("Invalid string '{}'", username.to_owned());
			}
			let mut user = User::new(kind, user_id, username);

			match user_obj.get("uuid") {
				Some(uuid) => user.set_uuid(json::ensure_type(uuid.as_str(), JsonType::Str)?),
				None => cprintln!("<y>Warning: It is recommended to have your uuid in the configuration for user {}", user_id)
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
			cprintln!("<y>Warning: Users are available but no default user is set. Starting in offline mode");
		} else {
			cprintln!("<y>Warning: No users are available. Starting in offline mode");
		}

		// Profiles
		let doc_profiles = json::access_object(obj, "profiles")?;
		for (profile_id, profile_val) in doc_profiles {
			if !validate_identifier(profile_id) {
				bail!("Invalid string '{}'", profile_id.to_owned());
			}
			let profile_obj = json::ensure_type(profile_val.as_object(), JsonType::Obj)?;
			let version = json::access_str(profile_obj, "version")?;
			if !VersionPattern::validate(version) {
				bail!("Invalid string '{}'", version.to_owned());
			}

			let modloader = match profile_obj.get("modloader") {
				Some(loader) => json::ensure_type(loader.as_str(), JsonType::Str),
				None => Ok("vanilla"),
			}?;
			let modloader = Modloader::from_str(modloader)
				.ok_or(anyhow!("Unknown modloader '{modloader}'"))?;

			let plugin_loader = match profile_obj.get("plugin_loader") {
				Some(loader) => json::ensure_type(loader.as_str(), JsonType::Str),
				None => Ok("vanilla"),
			}?;
			let plugin_loader = PluginLoader::from_str(plugin_loader)
				.ok_or(anyhow!("Unknown plugin loader '{plugin_loader}'"))?;

			if !game_modifications_compatible(&modloader, &plugin_loader) {
				bail!("Game modifications for profile '{profile_id}' are incompatible");
			}

			let mut profile = Profile::new(
				profile_id,
				version,
				modloader.clone(),
				plugin_loader.clone(),
			);

			// Instances
			if let Some(instances_val) = profile_obj.get("instances") {
				let doc_instances = json::ensure_type(instances_val.as_object(), JsonType::Obj)?;
				for (instance_id, instance_val) in doc_instances {
					if !validate_identifier(instance_id) {
						bail!("Invalid string '{}'", instance_id.to_owned());
					}
					if instances.contains_key(instance_id) {
						bail!("Duplicate instance '{instance_id}'");
					}

					let instance = parse_instance_config(instance_id, instance_val, &profile)?;

					profile.add_instance(instance_id);
					instances.insert(instance_id.to_string(), instance);
				}
			}

			// Packages
			if let Some(packages_val) = profile_obj.get("packages") {
				let doc_packages = json::ensure_type(packages_val.as_array(), JsonType::Arr)?;
				for package_val in doc_packages {
					if let Some(package_obj) = package_val.as_object() {
						let package_id = json::access_str(package_obj, "id")?;
						if !validate_identifier(package_id) {
							bail!("Invalid string '{}'", package_id.to_owned());
						}

						let req = PkgRequest::new(package_id);
						for cfg in profile.packages.iter() {
							if cfg.req == req {
								bail!("Duplicate package '{}' in profile '{profile_id}'", req.name);
							}
						}
						if let Some(val) = package_obj.get("type") {
							match json::ensure_type(val.as_str(), JsonType::Str)? {
								"local" => {
									let package_path = json::access_str(package_obj, "path")?;
									let package_path = shellexpand::tilde(package_path);
									let package_version =
										match json::access_str(package_obj, "version") {
											Ok(version) => Ok(version),
											Err(..) => Err(anyhow!("Local packages must specify a version")),
										}?;
									packages.insert_local(
										&req,
										package_version,
										&PathBuf::from(package_path.to_string()),
									);
								}
								"remote" => {}
								typ => bail!("Unknown package type '{typ}' for package '{package_id}'"),
							}
						}
						let features = match package_obj.get("features") {
							Some(features) => {
								let features =
									json::ensure_type(features.as_array(), JsonType::Arr)?;
								let mut out = Vec::new();
								for feature in features {
									let feature =
										json::ensure_type(feature.as_str(), JsonType::Str)?;
									if !validate_identifier(feature) {
										bail!("Invalid string '{}'", feature.to_owned());
									}
									out.push(feature.to_owned());
								}
								out
							}
							None => Vec::new(),
						};

						let perms = match package_obj.get("permissions") {
							Some(perms) => {
								let perms = json::ensure_type(perms.as_str(), JsonType::Str)?;
								EvalPermissions::from_str(perms)
									.ok_or(anyhow!("Unknown package permissions {perms}"))?
							}
							None => EvalPermissions::Standard,
						};

						let pkg = PkgConfig {
							req,
							features,
							permissions: perms,
						};
						profile.packages.push(pkg);
					} else if let Some(package_id) = package_val.as_str() {
						if !validate_identifier(package_id) {
							bail!("Invalid string '{}'", package_id.to_owned());
						}
						let req = PkgRequest::new(package_id);
						let pkg = PkgConfig {
							req,
							features: Vec::new(),
							permissions: EvalPermissions::Standard,
						};
						profile.packages.push(pkg);
					} else {
						bail!("Expected an Object or a String for a package in profile '{profile_id}'");
					}
				}
			}

			profiles.insert(profile_id.to_string(), Box::new(profile));
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
