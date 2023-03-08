use serde::Deserialize;
use serde_json::Value;

use crate::data::instance::launch::LaunchOptions;
use crate::data::instance::{InstKind, Instance};
use crate::data::profile::Profile;
use crate::io::java::args::MemoryNum;
use crate::io::java::JavaKind;

use super::{ConfigError, ContentError};

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Args {
	List(Vec<String>),
	String(String),
}

impl Args {
	pub fn parse(&self) -> Vec<String> {
		match self {
			Self::List(vec) => vec.clone(),
			Self::String(string) => string.split(' ').map(|string| string.to_owned()).collect(),
		}
	}
}

impl Default for Args {
	fn default() -> Self {
		Self::List(Vec::new())
	}
}

#[derive(Deserialize, Debug, Default)]
pub struct LaunchArgs {
	#[serde(default)]
	pub jvm: Args,
	#[serde(default)]
	pub game: Args,
}

#[derive(Deserialize, Debug, Default)]
#[serde(untagged)]
pub enum LaunchMemory {
	#[default]
	None,
	Single(String),
	Both {
		init: String,
		max: String,
	},
}

fn default_java() -> String {
	String::from("adoptium")
}

#[derive(Deserialize, Debug)]
pub struct LaunchConfig {
	#[serde(default)]
	pub args: LaunchArgs,
	#[serde(default)]
	pub memory: LaunchMemory,
	#[serde(default = "default_java")]
	pub java: String,
}

impl LaunchConfig {
	pub fn to_options(&self) -> LaunchOptions {
		let init_mem = match &self.memory {
			LaunchMemory::None => None,
			LaunchMemory::Single(string) => MemoryNum::from_str(&string),
			LaunchMemory::Both { init, .. } => MemoryNum::from_str(&init),
		};
		let max_mem = match &self.memory {
			LaunchMemory::None => None,
			LaunchMemory::Single(string) => MemoryNum::from_str(&string),
			LaunchMemory::Both { max, .. } => MemoryNum::from_str(&max),
		};
		LaunchOptions {
			jvm_args: self.args.jvm.parse(),
			game_args: self.args.game.parse(),
			init_mem,
			max_mem,
			java: JavaKind::from_str(&self.java),
		}
	}
}

impl Default for LaunchConfig {
	fn default() -> Self {
		Self {
			args: LaunchArgs {
				jvm: Args::default(),
				game: Args::default(),
			},
			memory: LaunchMemory::default(),
			java: default_java(),
		}
	}
}

#[derive(Deserialize)]
struct InstanceConfig {
	#[serde(rename = "type")]
	kind: String,
	#[serde(default)]
	launch: LaunchConfig,
}

pub fn parse_instance_config(
	id: &str,
	val: &Value,
	profile: &Profile,
) -> Result<Instance, ConfigError> {
	let config = serde_json::from_value::<InstanceConfig>(val.clone())?;
	let kind = match config.kind.as_str() {
		"client" => Ok(InstKind::Client),
		"server" => Ok(InstKind::Server),
		typ => Err(ContentError::InstType(typ.to_string(), id.to_owned())),
	}?;

	let instance = Instance::new(
		kind,
		id,
		&profile.version,
		profile.modloader.clone(),
		profile.plugin_loader.clone(),
		config.launch.to_options(),
	);

	Ok(instance)
}
