use std::collections::HashMap;

use anyhow::{anyhow, Context};
use serde::Deserialize;
use serde_json::Value;

use crate::io::launch::LaunchOptions;
use crate::data::instance::{InstKind, Instance};
use crate::data::profile::Profile;
use crate::io::java::args::{MemoryNum, ArgsPreset};
use crate::io::java::JavaKind;

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
}

impl LaunchConfig {
	pub fn to_options(&self) -> LaunchOptions {
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
		LaunchOptions {
			jvm_args: self.args.jvm.parse(),
			game_args: self.args.game.parse(),
			min_mem,
			max_mem,
			java: JavaKind::from_str(&self.java),
			preset: ArgsPreset::from_str(&self.preset),
			env: self.env.clone(),
			wrapper: self.wrapper.clone(),
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
			preset: default_flags_preset(),
			env: HashMap::new(),
			wrapper: None,
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
) -> anyhow::Result<Instance> {
	let config = serde_json::from_value::<InstanceConfig>(val.clone())
		.context("Failed to parse instance config")?;
	let kind = match config.kind.as_str() {
		"client" => Ok(InstKind::Client),
		"server" => Ok(InstKind::Server),
		typ => Err(anyhow!("Unknown instance type '{typ}' on instance '{id}'")),
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
