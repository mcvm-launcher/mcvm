use serde::Deserialize;
use serde_json::Value;

use crate::data::{instance::{Instance, InstKind}, profile::Profile};

use super::{ConfigError, ContentError};

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Args {
	List(Vec<String>),
	String(String)
}

impl Args {
	pub fn parse(&self) -> Vec<String> {
		match self {
			Self::List(vec) => vec.clone(),
			Self::String(string) => string.split(' ').map(|string| string.to_owned()).collect()
		}
	}
}

impl Default for Args {
	fn default() -> Self {
		Self::List(Vec::new())
	}
}

#[derive(Deserialize, Debug)]
pub struct LaunchArgs {
	#[serde(default)]
	pub jvm: Args,
	#[serde(default)]
	pub game: Args
}

#[derive(Deserialize, Debug)]
pub struct LaunchOptions {
	pub args: LaunchArgs
}

impl Default for LaunchOptions {
	fn default() -> Self {
		Self {
			args: LaunchArgs {
				jvm: Args::default(),
				game: Args::default()
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
		profile.plugin_loader.clone(),
		config.launch_options
	);

	Ok(instance)
}
