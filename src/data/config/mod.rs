mod preferences;
pub mod instance;

use preferences::ConfigPreferences;
use self::instance::parse_instance_config;

use super::addon::{PluginLoader, Modloader, game_modifications_compatible};
use super::user::{User, UserKind, AuthState, Auth, validate_username};
use super::profile::{Profile, InstanceRegistry};
use crate::package::PkgConfig;
use crate::package::eval::eval::EvalPermissions;
use crate::package::reg::{PkgRegistry, PkgRequest};
use crate::util::validate_identifier;
use crate::util::versions::VersionPattern;
use crate::util::json::{self, JsonType};

use color_print::cprintln;
use serde_json::json;

use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;

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

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
	#[error("{}", .0)]
	File(#[from] std::io::Error),
	#[error("Failed to parse json:\n{}", .0)]
	Json(#[from] json::JsonError),
	#[error("Json operation failed:\n{}", .0)]
	SerdeJson(#[from] serde_json::Error),
	#[error("Invalid config content:\n{}", .0)]
	Content(#[from] ContentError)
}

#[derive(Debug, thiserror::Error)]
pub enum ContentError {
	#[error("Unknown type {} for user {}", .0, .1)]
	UserType(String, String),
	#[error("Unknown type {} for instance {}", .0, .1)]
	InstType(String, String),
	#[error("Unknown type {} for package {}", .0, .1)]
	PkgType(String, String),
	#[error("Unknown package permissions '{}'", .0)]
	UnknownPkgPerms(String),
	#[error("Unknown default user '{}'", .0)]
	DefaultUserNotFound(String),
	#[error("Duplicate instance '{}'", .0)]
	DuplicateInstance(String),
	#[error("Package '{}': Local packages must specify their exact version without special patterns", .0)]
	LocalPackageVersion(String),
	#[error("Duplicate package '{}' in profile '{}'", .0, .1)]
	DuplicatePackage(String, String),
	#[error("String '{}' is invalid", .0)]
	InvalidString(String),
	#[error("Unknown modloader '{}'", .0)]
	UnknownModloader(String),
	#[error("Unknown plugin_loader '{}'", .0)]
	UnknownPluginLoader(String),
	#[error("Modloader and plugin loader are incompatible for profile '{}'", .0)]
	IncompatibleGameMods(String)
}

#[derive(Debug)]
pub struct Config {
	pub auth: Auth,
	pub instances: InstanceRegistry,
	pub profiles: HashMap<String, Box<Profile>>,
	pub packages: PkgRegistry,
	pub prefs: ConfigPreferences
}

impl Config {
	fn open(path: &PathBuf) -> Result<Box<json::JsonObject>, ConfigError> {
		if path.exists() {
			let doc = json::parse_object(&fs::read_to_string(path)?)?;
			Ok(doc)
		} else {
			let doc = default_config();
			fs::write(path, serde_json::to_string_pretty(&doc)?)?;
			Ok(Box::new(json::ensure_type(doc.as_object(), JsonType::Obj)?.clone()))
		}
	}
	
	fn load_from_obj(obj: &json::JsonObject) -> Result<Self, ConfigError> {
		let mut auth = Auth::new();
		let mut instances = InstanceRegistry::new();
		let mut profiles = HashMap::new();
		// Preferences
		let (prefs, repositories) = ConfigPreferences::read(obj.get("preferences"))?;

		let mut packages = PkgRegistry::new(repositories);

		// Users
		let users = json::access_object(obj, "users")?;
		for (user_id, user_val) in users.iter() {
			if !validate_identifier(&user_id) {
				Err(ContentError::InvalidString(user_id.to_owned()))?
			}
			let user_obj = json::ensure_type(user_val.as_object(), JsonType::Obj)?;
			let kind = match json::access_str(user_obj, "type")? {
				"microsoft" => Ok(UserKind::Microsoft),
				"demo" => Ok(UserKind::Demo),
				typ => Err(ContentError::UserType(typ.to_string(), user_id.to_string()))
			}?;
			let username = json::access_str(user_obj, "name")?;
			if !validate_username(kind.clone(), username) {
				Err(ContentError::InvalidString(username.to_owned()))?
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
				None => return Err(ConfigError::from(ContentError::DefaultUserNotFound(user_id)))
			}
		} else if users.is_empty() {
			cprintln!("<y>Warning: Users are available but no default user is set. Starting in offline mode");
		} else {
			cprintln!("<y>Warning: No users are available. Starting in offline mode");
		}

		// Profiles
		let doc_profiles = json::access_object(obj, "profiles")?;
		for (profile_id, profile_val) in doc_profiles {
			if !validate_identifier(&profile_id) {
				Err(ContentError::InvalidString(profile_id.to_owned()))?
			}
			let profile_obj = json::ensure_type(profile_val.as_object(), JsonType::Obj)?;
			let version = json::access_str(profile_obj, "version")?;
			if !VersionPattern::validate(version) {
				Err(ContentError::InvalidString(version.to_owned()))?
			}

			let modloader = match profile_obj.get("modloader") {
				Some(loader) => json::ensure_type(loader.as_str(), JsonType::Str),
				None => Ok("vanilla")
			}?;
			let modloader = Modloader::from_str(modloader)
				.ok_or(ContentError::UnknownModloader(modloader.to_owned()))?;

			let plugin_loader = match profile_obj.get("plugin_loader") {
				Some(loader) => json::ensure_type(loader.as_str(), JsonType::Str),
				None => Ok("vanilla")
			}?;
			let plugin_loader = PluginLoader::from_str(plugin_loader)
				.ok_or(ContentError::UnknownPluginLoader(plugin_loader.to_owned()))?;

			if !game_modifications_compatible(&modloader, &plugin_loader) {
				return Err(ConfigError::Content(ContentError::IncompatibleGameMods(profile_id.clone())));
			}

			let mut profile = Profile::new(profile_id, version, modloader.clone(), plugin_loader.clone());
			
			// Instances
			if let Some(instances_val) = profile_obj.get("instances") {
				let doc_instances = json::ensure_type(instances_val.as_object(), JsonType::Obj)?;
				for (instance_id, instance_val) in doc_instances {
					if !validate_identifier(&instance_id) {
						Err(ContentError::InvalidString(instance_id.to_owned()))?
					}
					if instances.contains_key(instance_id) {
						return Err(ConfigError::from(ContentError::DuplicateInstance(instance_id.to_string())));
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
						if !validate_identifier(&package_id) {
							Err(ContentError::InvalidString(package_id.to_owned()))?
						}
						
						let req = PkgRequest::new(package_id);
						for cfg in profile.packages.iter() {
							if cfg.req == req {
								Err(ContentError::DuplicatePackage(req.name.clone(), profile_id.clone()))?;
							}
						}
						if let Some(val) = package_obj.get("type") {
							match json::ensure_type(val.as_str(), JsonType::Str)? {
								"local" => {
									let package_path = json::access_str(package_obj, "path")?;
									let package_path = shellexpand::tilde(package_path);
									let package_version = match json::access_str(package_obj, "version") {
										Ok(version) => Ok(version),
										Err(..) => Err(ContentError::LocalPackageVersion(package_id.to_owned()))
									}?;
									packages.insert_local(
										&req,
										package_version,
										&PathBuf::from(package_path.to_string())
									);
								},
								"remote" => {}
								typ => Err(ContentError::PkgType(typ.to_string(), String::from("package")))?
							}
						}
						let features = match package_obj.get("features") {
							Some(features) => {
								let features = json::ensure_type(features.as_array(), JsonType::Arr)?;
								let mut out = Vec::new();
								for feature in features {
									let feature = json::ensure_type(feature.as_str(), JsonType::Str)?;
									if !validate_identifier(&feature) {
										Err(ContentError::InvalidString(feature.to_owned()))?
									}
									out.push(feature.to_owned());
								}
								out
							}
							None => Vec::new()
						};

						let perms = match package_obj.get("permissions") {
							Some(perms) => {
								let perms = json::ensure_type(perms.as_str(), JsonType::Str)?;
								EvalPermissions::from_str(perms).ok_or(ContentError::UnknownPkgPerms(perms.to_owned()))?
							}
							None => EvalPermissions::Standard
						};

						let pkg = PkgConfig {
							req,
							features,
							permissions: perms
						};
						profile.packages.push(pkg);
					} else if let Some(package_id) = package_val.as_str() {
						if !validate_identifier(&package_id) {
							Err(ContentError::InvalidString(package_id.to_owned()))?
						}
						let req = PkgRequest::new(package_id);
						let pkg = PkgConfig {
							req,
							features: Vec::new(),
							permissions: EvalPermissions::Standard
						};
						profile.packages.push(pkg);
					} else {
						return Err(ConfigError::Json(json::JsonError::Type(vec![JsonType::Obj, JsonType::Str])));
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
			prefs
		})
	}

	pub fn load(path: &PathBuf) -> Result<Self, ConfigError> {
		let obj = Self::open(path)?;
		Self::load_from_obj(&obj)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_default_config() {
		let obj = json::ensure_type(default_config().as_object(),
			JsonType::Obj).unwrap().clone();
		Config::load_from_obj(&obj).unwrap();
	}
}
