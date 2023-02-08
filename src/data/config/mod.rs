pub mod preferences;

use preferences::ConfigPreferences;
use super::user::{User, UserKind, AuthState, Auth};
use super::profile::{Profile, InstanceRegistry};
use super::instance::{Instance, InstKind};
use crate::package::{Package, PkgKind};
use crate::util::{json, versions::MinecraftVersion};

use color_print::cprintln;
use serde_json::json;

use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;

#[derive(Debug)]
pub struct Config {
	pub auth: Auth,
	pub instances: InstanceRegistry,
	pub profiles: HashMap<String, Box<Profile>>,
	pub packages: HashMap<String, Box<Package>>,
	pub prefs: ConfigPreferences
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
	#[error("Unknown default user '{}'", .0)]
	DefaultUserNotFound(String),
	#[error("Duplicate instance '{}'", .0)]
	DuplicateInstance(String)
}

impl Config {
	fn open(path: &PathBuf) -> Result<Box<json::JsonObject>, ConfigError> {
		if path.exists() {
			let doc = json::parse_object(&fs::read_to_string(path)?)?;
			Ok(doc)
		} else {
			let doc = json!(
				{
					"users": {},
					"profiles": {}
				}
			);
			fs::write(path, serde_json::to_string_pretty(&doc)?)?;
			Ok(Box::new(json::ensure_type(doc.as_object(), json::JsonType::Object)?.clone()))
		}
	}

	pub fn load(path: &PathBuf) -> Result<Self, ConfigError> {
		let doc = Self::open(path)?;
		let mut auth = Auth::new();
		let mut instances = InstanceRegistry::new();
		let mut profiles = HashMap::new();
		let mut packages = HashMap::new();

		// Users
		let users = json::access_object(&doc, "users")?;
		for (user_id, user_val) in users.iter() {
			let user_obj = json::ensure_type(user_val.as_object(), json::JsonType::Object)?;
			let kind = match json::access_str(user_obj, "type")? {
				"microsoft" => Ok(UserKind::Microsoft),
				"demo" => Ok(UserKind::Demo),
				typ => Err(ContentError::UserType(typ.to_string(), user_id.to_string()))
			}?;
			let mut user = User::new(kind, user_id, json::access_str(user_obj, "name")?);

			match user_obj.get("uuid") {
				Some(uuid) => user.set_uuid(json::ensure_type(uuid.as_str(), json::JsonType::Str)?),
				None => cprintln!("<y>Warning: It is recommended to have your uuid in the configuration for user {}", user_id)
			};
			
			auth.users.insert(user_id.to_string(), user);
		}

		if let Some(user_val) = doc.get("default_user") {
			let user_id = json::ensure_type(user_val.as_str(), json::JsonType::Str)?.to_string();
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
		let doc_profiles = json::access_object(&doc, "profiles")?;
		for (profile_id, profile_val) in doc_profiles {
			let profile_obj = json::ensure_type(profile_val.as_object(), json::JsonType::Object)?;
			let version =  MinecraftVersion::from(json::access_str(profile_obj, "version")?);

			let mut profile = Profile::new(profile_id, &version);
			
			// Instances
			if let Some(instances_val) = profile_obj.get("instances") {
				let doc_instances = json::ensure_type(instances_val.as_object(), json::JsonType::Object)?;
				for (instance_id, instance_val) in doc_instances {
					if instances.contains_key(instance_id) {
						return Err(ConfigError::from(ContentError::DuplicateInstance(instance_id.to_string())));
					}

					let instance_obj = json::ensure_type(instance_val.as_object(), json::JsonType::Object)?;
					let kind = match json::access_str(instance_obj, "type")? {
						"client" => Ok(InstKind::Client),
						"server" => Ok(InstKind::Server),
						typ => Err(ContentError::InstType(typ.to_string(), instance_id.to_string()))
					}?;

					let instance = Instance::new(kind, instance_id, &version);
					profile.add_instance(instance_id);
					instances.insert(instance_id.to_string(), instance);
				}
			}

			if let Some(packages_val) = profile_obj.get("packages") {
				let doc_packages = json::ensure_type(packages_val.as_array(), json::JsonType::Array)?;
				for package_val in doc_packages {
					let package_obj = json::ensure_type(package_val.as_object(), json::JsonType::Object)?;
					let package_type = json::access_str(package_obj, "type")?;
					let kind = match package_type {
						"local" => {
							let package_path = json::access_str(package_obj, "path")?;
							Ok(PkgKind::Local(PathBuf::from(package_path)))
						},
						typ => Err(ContentError::PkgType(typ.to_string(), "package".to_string()))
					}?;
				}
			}
			
			profiles.insert(profile_id.to_string(), Box::new(profile));
		}

		// Preferences
		let prefs = ConfigPreferences::new(doc.get("preferences"))?;

		Ok(Self {
			auth,
			instances,
			profiles,
			packages,
			prefs
		})
	}
}
