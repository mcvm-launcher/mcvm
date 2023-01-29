use crate::lib::json;
use crate::user::User;
use crate::user::UserKind;
use crate::user::AuthState;
use super::profile::InstanceRegistry;
use super::profile::Profile;

use color_print::cprintln;
use serde_json::json;

use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;

pub struct ConfigData<'a> {
	users: HashMap<String, User>,
	auth: AuthState<'a>,
	instances: InstanceRegistry,
	profiles: HashMap<String, Box<Profile>>
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
	#[error("Unknown user type {} for user {}", .0, .1)]
	UserType(String, String)
}

impl<'a> ConfigData<'a> {
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
		let users = json::access_object(&doc, "users")?;
		for (user_id, user_val) in users.iter() {
			let user_obj = json::ensure_type(user_val.as_object(), json::JsonType::Object)?;
			// Ok(User::new(UserKind::Microsoft, user_id, json::access_str(user_obj, "name")?))
			let kind: UserKind = match json::access_str(user_obj, "type")? {
				"mojang" => {
					Ok(UserKind::Microsoft)
				},
				"demo" => {
					Ok(UserKind::Demo)
				},
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

		Ok(config)
	}
}

pub struct Config<'a> {
	data: Option<ConfigData<'a>>,
	path: PathBuf
}

impl<'a> Config<'a> {
	pub fn new(path: &PathBuf) -> Self {
		Self {
			data: None,
			path: path.to_owned()
		}
	}

	pub fn load(&mut self) -> Result<(), ConfigError> {
		if let None = &mut self.data {
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
