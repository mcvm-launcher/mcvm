use std::collections::HashMap;

use anyhow::{ensure, Context};
use serde::Deserialize;
use serde_json::Value;

use crate::data::instance::{InstKind, Instance};
use crate::data::profile::Profile;
use crate::io::java::args::{ArgsPreset, MemoryNum};
use crate::io::java::JavaKind;
use crate::io::launch::LaunchOptions;
use crate::io::options::client::ClientOptions;
use crate::io::options::server::ServerOptions;

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
		min: String,
		max: String,
	},
}

fn default_java() -> String {
	String::from("adoptium")
}

fn default_flags_preset() -> String {
	String::from("none")
}

#[derive(Deserialize, Debug, PartialEq, Default, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum QuickPlay {
	World { world: String },
	Server {
		server: String,
		port: Option<u16>,
	},
	Realm { realm: String },
	#[default]
	None,
}

#[derive(Deserialize, Debug)]
pub struct LaunchConfig {
	#[serde(default)]
	pub args: LaunchArgs,
	#[serde(default)]
	pub memory: LaunchMemory,
	#[serde(default = "default_java")]
	pub java: String,
	#[serde(default = "default_flags_preset")]
	pub preset: String,
	#[serde(default)]
	pub env: HashMap<String, String>,
	#[serde(default)]
	pub wrapper: Option<String>,
	#[serde(default)]
	pub quick_play: QuickPlay,
}

impl LaunchConfig {
	pub fn to_options(&self) -> anyhow::Result<LaunchOptions> {
		let min_mem = match &self.memory {
			LaunchMemory::None => None,
			LaunchMemory::Single(string) => MemoryNum::from_str(string),
			LaunchMemory::Both { min, .. } => MemoryNum::from_str(min),
		};
		let max_mem = match &self.memory {
			LaunchMemory::None => None,
			LaunchMemory::Single(string) => MemoryNum::from_str(string),
			LaunchMemory::Both { max, .. } => MemoryNum::from_str(max),
		};
		if let Some(min_mem) = &min_mem {
			if let Some(max_mem) = &max_mem {
				ensure!(
					min_mem.to_bytes() <= max_mem.to_bytes(),
					"Minimum memory must be less than or equal to maximum memory"
				);
			}
		}
		Ok(LaunchOptions {
			jvm_args: self.args.jvm.parse(),
			game_args: self.args.game.parse(),
			min_mem,
			max_mem,
			java: JavaKind::from_str(&self.java),
			preset: ArgsPreset::from_str(&self.preset),
			env: self.env.clone(),
			wrapper: self.wrapper.clone(),
			quick_play: self.quick_play.clone(),
		})
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
			preset: default_flags_preset(),
			env: HashMap::new(),
			wrapper: None,
			quick_play: QuickPlay::default(),
		}
	}
}

#[derive(Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum InstanceConfig {
	Client {
		#[serde(default)]
		launch: LaunchConfig,
		#[serde(default)]
		options: Option<Box<ClientOptions>>,
	},
	Server {
		#[serde(default)]
		launch: LaunchConfig,
		#[serde(default)]
		options: Option<Box<ServerOptions>>,
	},
}

pub fn parse_instance_config(id: &str, val: &Value, profile: &Profile) -> anyhow::Result<Instance> {
	let config = serde_json::from_value::<InstanceConfig>(val.clone())
		.context("Failed to parse instance config")?;
	let (kind, launch) = match config {
		InstanceConfig::Client { launch, options } => (InstKind::Client { options }, launch),
		InstanceConfig::Server { launch, options } => (InstKind::Server { options }, launch),
	};

	let instance = Instance::new(
		kind,
		id,
		profile.modloader.clone(),
		profile.plugin_loader.clone(),
		launch.to_options()?,
	);

	Ok(instance)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_quickplay_deser() {
		#[derive(Deserialize)]
		struct Test {
			quick_play: QuickPlay,
		}

		let test = serde_json::from_str::<Test>(r#"{
			"quick_play": {
				"type": "server",
				"server": "localhost",
				"port": 25565,
				"world": "test",
				"realm": "my_realm"
			}	
		}"#).unwrap();
		assert_eq!(
			test.quick_play,
			QuickPlay::Server {
				server: String::from("localhost"),
				port: Some(25565)
			}
		);
	}
}
