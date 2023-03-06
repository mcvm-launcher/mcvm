use serde::Deserialize;
use serde_json::Value;

use crate::data::{instance::{Instance, InstKind}, profile::Profile};

use super::{ConfigError, ContentError};

#[derive(Deserialize, Debug)]
pub struct LaunchArgs {
	#[serde(default)]
	pub jvm: Vec<String>,
	#[serde(default)]
	pub game: Vec<String>
}

#[derive(Deserialize, Debug)]
pub struct LaunchOptions {
	pub args: LaunchArgs
}

impl Default for LaunchOptions {
	fn default() -> Self {
		Self {
			args: LaunchArgs {
				jvm: Vec::new(),
				game: Vec::new()
			}
		}
	}
}

#[derive(Deserialize)]
struct InstanceConfig {
	#[serde(rename = "type")]
	kind: String,
	#[serde(default)]
	launch_options: LaunchOptions
}

pub fn parse_instance_config(id: &str, val: &Value, profile: &Profile) -> Result<Instance, ConfigError> {
	let config = serde_json::from_value::<InstanceConfig>(val.clone())?;
	let kind = match config.kind.as_str() {
		"client" => Ok(InstKind::Client),
		"server" => Ok(InstKind::Server),
		typ => Err(ContentError::InstType(typ.to_string(), id.to_owned()))
	}?;

	let instance = Instance::new(
		kind,
		id,
		&profile.version,
		profile.modloader.clone(),
		profile.pluginloader.clone(),
		config.launch_options
	);

	Ok(instance)
}
