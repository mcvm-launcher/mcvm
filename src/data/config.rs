use crate::util::json;
use crate::user::{User, UserKind, AuthState};
use crate::util::versions::MinecraftVersion;
use super::profile::{Profile, InstanceRegistry};
use super::instance::{Instance, InstKind};

use color_print::cprintln;
use serde_json::json;

use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;

#[derive(Debug)]
pub struct ConfigData {
	pub users: HashMap<String, User>,
	pub auth: AuthState,
	pub instances: InstanceRegistry,
	pub profiles: HashMap<String, Box<Profile>>
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
	InstType(String, String)
}

impl ConfigData {
	pub fn new() -> Self {
		Self {
			users: HashMap::new(),
			auth: AuthState::Offline,
			instances: InstanceRegistry::new(),
			profiles: HashMap::new()
		}
	}

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
			Ok(Box::new(doc.as_object().expect("Expected config to be an object").clone()))
		}
	}

	pub fn load(path: &PathBuf) -> Result<Self, ConfigError> {
		let mut config = Self::new();
		let doc = Self::open(path)?;

		// Users
		if let Some(user_val) = doc.get("default_user") {
			config.auth = AuthState::Authed(json::ensure_type(user_val.as_str(), json::JsonType::Str)?.to_string());
		}

		let users = json::access_object(&doc, "users")?;
		for (user_id, user_val) in users.iter() {
			let user_obj = json::ensure_type(user_val.as_object(), json::JsonType::Object)?;
			let kind = match json::access_str(user_obj, "type")? {
				"microsoft" => {
					if let AuthState::Offline = config.auth {
						config.auth = AuthState::Authed(user_id.to_string());
					}
					Ok(UserKind::Microsoft)
				},
				"demo" => Ok(UserKind::Demo),
				typ => Err(ContentError::UserType(typ.to_string(), user_id.to_string()))
			}?;
			let mut user = User::new(kind, user_id, json::access_str(user_obj, "name")?);

			if let Some(uuid) = user_obj.get("uuid") {
				user.set_uuid(json::ensure_type(uuid.as_str(), json::JsonType::Str)?);
			} else {
				cprintln!("<y>Warning: It is recommended to have your uuid in the configuration for user {}", user_id);
			}

			config.users.insert(user_id.to_string(), user);
		}

		// Profiles
		let profiles = json::access_object(&doc, "profiles")?;
		for (profile_id, profile_val) in profiles.iter() {
			let profile_obj = json::ensure_type(profile_val.as_object(), json::JsonType::Object)?;
			let version =  MinecraftVersion::from(json::access_str(profile_obj, "version")?);

			let mut profile = Profile::new(profile_id, &version);
			
			// Instances
			if let Some(instances_val) = profile_obj.get("instances") {
				let instances = json::ensure_type(instances_val.as_object(), json::JsonType::Object)?;
				for (instance_id, instance_val) in instances.iter() {
					let instance_obj = json::ensure_type(instance_val.as_object(), json::JsonType::Object)?;
					let kind = match json::access_str(instance_obj, "type")? {
						"client" => {
							Ok(InstKind::Client)
						},
						"server" => {
							Ok(InstKind::Server)
						},
						typ => Err(ContentError::InstType(typ.to_string(), instance_id.to_string()))
					}?;

					let instance = Instance::new(kind, instance_id, &version);
					profile.add_instance(instance_id);
					config.instances.insert(instance_id.to_string(), instance);
				}
			}

			// TODO: Packages
			
			config.profiles.insert(profile_id.to_string(), Box::new(profile));
		}

		Ok(config)
	}
}

#[derive(Debug)]
pub struct Config {
	pub data: Option<ConfigData>,
	path: PathBuf
}

impl Config {
	pub fn new(path: &PathBuf) -> Self {
		Self {
			data: None,
			path: path.to_owned()
		}
	}

	pub fn load(&mut self) -> Result<(), ConfigError> {
		if self.data.is_none() {
			self.data = match ConfigData::load(&self.path) {
				Ok(data) => Some(data),
				Err(err) => {
					cprintln!("<r>Error when evaluating config:\n{}", err);
					None
				}
			}
		}
		Ok(())
	}
}
