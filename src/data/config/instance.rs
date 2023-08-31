use std::collections::HashMap;
use std::str::FromStr;

use anyhow::{anyhow, bail, ensure, Context};
use mcvm_shared::instance::Side;
use serde::{Deserialize, Serialize};

use crate::data::instance::launch::{LaunchOptions, WrapperCommand};
use crate::data::instance::{InstKind, Instance};
use crate::data::profile::Profile;
use crate::io::java::args::{ArgsPreset, MemoryNum};
use crate::io::java::JavaKind;
use crate::io::options::client::ClientOptions;
use crate::io::options::server::ServerOptions;
use crate::io::snapshot;
use crate::util::merge_options;

#[derive(Deserialize, Serialize, Debug, Clone)]
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

	/// Merge Args
	pub fn merge(&mut self, other: Self) {
		let mut out = self.parse();
		out.extend(other.parse());
		*self = Self::List(out);
	}
}

impl Default for Args {
	fn default() -> Self {
		Self::List(Vec::new())
	}
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct LaunchArgs {
	#[serde(default)]
	pub jvm: Args,
	#[serde(default)]
	pub game: Args,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
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

#[derive(Deserialize, Serialize, Debug, PartialEq, Default, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum QuickPlay {
	World {
		world: String,
	},
	Server {
		server: String,
		port: Option<u16>,
	},
	Realm {
		realm: String,
	},
	#[default]
	None,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
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
	pub wrapper: Option<WrapperCommand>,
	#[serde(default)]
	pub quick_play: QuickPlay,
}

impl LaunchConfig {
	pub fn to_options(&self) -> anyhow::Result<LaunchOptions> {
		let min_mem = match &self.memory {
			LaunchMemory::None => None,
			LaunchMemory::Single(string) => MemoryNum::parse(string),
			LaunchMemory::Both { min, .. } => MemoryNum::parse(min),
		};
		let max_mem = match &self.memory {
			LaunchMemory::None => None,
			LaunchMemory::Single(string) => MemoryNum::parse(string),
			LaunchMemory::Both { max, .. } => MemoryNum::parse(max),
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
			java: JavaKind::parse(&self.java),
			preset: ArgsPreset::from_str(&self.preset)?,
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

impl LaunchConfig {
	/// Merge multiple LaunchConfigs
	pub fn merge(&mut self, other: Self) -> &mut Self {
		self.args.jvm.merge(other.args.jvm);
		self.args.game.merge(other.args.game);
		if !matches!(other.memory, LaunchMemory::None) {
			self.memory = other.memory;
		}
		self.java = other.java;
		if other.preset != "none" {
			self.preset = other.preset;
		}
		self.env.extend(other.env);
		if other.wrapper.is_some() {
			self.wrapper = other.wrapper;
		}
		if !matches!(other.quick_play, QuickPlay::None) {
			self.quick_play = other.quick_play;
		}

		self
	}
}

#[derive(Deserialize, Serialize, Clone, Debug, Copy)]
pub struct WindowResolution {
	pub width: u32,
	pub height: u32,
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
#[serde(default)]
pub struct ClientWindowConfig {
	pub resolution: Option<WindowResolution>,
}

impl ClientWindowConfig {
	/// Merge two ClientWindowConfigs
	pub fn merge(&mut self, other: Self) -> &mut Self {
		self.resolution = merge_options(self.resolution, other.resolution);
		self
	}
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum FullInstanceConfig {
	Client {
		#[serde(default)]
		launch: LaunchConfig,
		#[serde(default)]
		options: Option<Box<ClientOptions>>,
		#[serde(default)]
		window: ClientWindowConfig,
		#[serde(default)]
		preset: Option<String>,
		#[serde(default)]
		datapack_folder: Option<String>,
		#[serde(default)]
		snapshots: Option<snapshot::Config>,
	},
	Server {
		#[serde(default)]
		launch: LaunchConfig,
		#[serde(default)]
		options: Option<Box<ServerOptions>>,
		#[serde(default)]
		preset: Option<String>,
		#[serde(default)]
		datapack_folder: Option<String>,
		#[serde(default)]
		snapshots: Option<snapshot::Config>,
	},
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
#[serde(rename_all = "snake_case")]
pub enum InstanceConfig {
	Simple(Side),
	Full(FullInstanceConfig),
}

impl InstanceConfig {
	/// Converts simple config into full
	pub fn make_full(&self) -> FullInstanceConfig {
		match self {
			Self::Full(config) => config.clone(),
			Self::Simple(side) => match side {
				Side::Client => FullInstanceConfig::Client {
					launch: LaunchConfig::default(),
					options: None,
					window: ClientWindowConfig::default(),
					preset: None,
					datapack_folder: None,
					snapshots: None,
				},
				Side::Server => FullInstanceConfig::Server {
					launch: LaunchConfig::default(),
					options: None,
					preset: None,
					datapack_folder: None,
					snapshots: None,
				},
			},
		}
	}

	/// Checks if this config has the preset field filled out
	pub fn uses_preset(&self) -> bool {
		matches!(
			self,
			Self::Full(
				FullInstanceConfig::Client {
					preset: Some(..),
					..
				} | FullInstanceConfig::Server {
					preset: Some(..),
					..
				}
			)
		)
	}
}

/// Merge an InstanceConfig with a preset
///
/// Some values will be merged while others will have the right side take precendence
pub fn merge_instance_configs(
	preset: &InstanceConfig,
	config: &InstanceConfig,
) -> anyhow::Result<InstanceConfig> {
	let mut out = preset.make_full();
	let applied = config.make_full();
	out = match (out, applied) {
		(
			FullInstanceConfig::Client {
				mut launch,
				options,
				mut window,
				datapack_folder,
				snapshots,
				..
			},
			FullInstanceConfig::Client {
				launch: launch2,
				options: options2,
				window: window2,
				datapack_folder: datapack_folder2,
				snapshots: snapshots2,
				..
			},
		) => Ok::<FullInstanceConfig, anyhow::Error>(FullInstanceConfig::Client {
			launch: launch.merge(launch2).clone(),
			options: merge_options(options, options2),
			window: window.merge(window2).clone(),
			preset: None,
			datapack_folder: merge_options(datapack_folder, datapack_folder2),
			snapshots: merge_options(snapshots, snapshots2),
		}),
		(
			FullInstanceConfig::Server {
				mut launch,
				options,
				datapack_folder,
				snapshots,
				..
			},
			FullInstanceConfig::Server {
				launch: launch2,
				options: options2,
				datapack_folder: datapack_folder2,
				snapshots: snapshots2,
				..
			},
		) => Ok::<FullInstanceConfig, anyhow::Error>(FullInstanceConfig::Server {
			launch: launch.merge(launch2).clone(),
			options: merge_options(options, options2),
			preset: None,
			datapack_folder: merge_options(datapack_folder, datapack_folder2),
			snapshots: merge_options(snapshots, snapshots2),
		}),
		_ => bail!("Instance types do not match"),
	}?;

	Ok(InstanceConfig::Full(out))
}

pub fn read_instance_config(
	id: &str,
	config: &InstanceConfig,
	profile: &Profile,
	presets: &HashMap<String, InstanceConfig>,
) -> anyhow::Result<Instance> {
	let config = if let InstanceConfig::Full(
		FullInstanceConfig::Client {
			preset: Some(preset),
			..
		}
		| FullInstanceConfig::Server {
			preset: Some(preset),
			..
		},
	) = config
	{
		let preset = presets
			.get(preset)
			.ok_or(anyhow!("Preset '{preset}' does not exist"))?;
		merge_instance_configs(preset, config).context("Failed to merge preset with instance")?
	} else {
		config.clone()
	};
	let (kind, launch, datapack_folder, snapshot_config) = match config {
		InstanceConfig::Simple(side) => (
			match side {
				Side::Client => InstKind::Client {
					options: None,
					window: ClientWindowConfig::default(),
				},
				Side::Server => InstKind::Server { options: None },
			},
			LaunchConfig::default(),
			None,
			None,
		),
		InstanceConfig::Full(config) => match config {
			FullInstanceConfig::Client {
				launch,
				options,
				window,
				datapack_folder,
				snapshots,
				..
			} => (
				InstKind::Client { options, window },
				launch,
				datapack_folder,
				snapshots,
			),
			FullInstanceConfig::Server {
				launch,
				options,
				datapack_folder,
				snapshots,
				..
			} => (
				InstKind::Server { options },
				launch,
				datapack_folder,
				snapshots,
			),
		},
	};

	let instance = Instance::new(
		kind,
		id,
		profile.modifications.clone(),
		launch.to_options()?,
		datapack_folder,
		snapshot_config.unwrap_or_default(),
	);

	Ok(instance)
}

#[cfg(test)]
mod tests {
	use super::*;

	use crate::{data::config::profile::GameModifications, util::versions::MinecraftVersion};
	use mcvm_shared::modifications::{ClientType, Modloader, ServerType};

	#[test]
	fn test_instance_deser() {
		#[derive(Deserialize)]
		struct Test {
			instance: InstanceConfig,
		}

		let test = serde_json::from_str::<Test>(
			r#"
			{
				"instance": "client"
			}
			"#,
		)
		.unwrap();

		let profile = Profile::new(
			"foo",
			MinecraftVersion::Latest,
			GameModifications::new(Modloader::Vanilla, ClientType::Vanilla, ServerType::Vanilla),
		);

		let instance =
			read_instance_config("foo", &test.instance, &profile, &HashMap::new()).unwrap();
		assert_eq!(instance.id, "foo");
		assert!(matches!(instance.kind, InstKind::Client { .. }));
	}

	#[test]
	fn test_instance_config_merging() {
		let presets = {
			let mut presets = HashMap::new();
			presets.insert(
				String::from("hello"),
				InstanceConfig::Full(FullInstanceConfig::Client {
					launch: LaunchConfig::default(),
					options: None,
					window: ClientWindowConfig {
						resolution: Some(WindowResolution {
							width: 200,
							height: 100,
						}),
					},
					preset: None,
					datapack_folder: None,
					snapshots: None,
				}),
			);
			presets
		};

		let profile = Profile::new(
			"foo",
			MinecraftVersion::Latest,
			GameModifications::new(Modloader::Vanilla, ClientType::Vanilla, ServerType::Vanilla),
		);

		let config = InstanceConfig::Full(FullInstanceConfig::Client {
			launch: LaunchConfig::default(),
			options: None,
			window: ClientWindowConfig::default(),
			preset: Some(String::from("hello")),
			datapack_folder: None,
			snapshots: None,
		});
		let instance = read_instance_config("test", &config, &profile, &presets)
			.expect("Failed to read instance config");
		if !matches!(
			instance.kind,
			InstKind::Client {
				options: None,
				window: ClientWindowConfig {
					resolution: Some(WindowResolution {
						width: 200,
						height: 100,
					})
				},
			}
		) {
			panic!("Does not match: {:?}", instance.kind);
		}

		let config = InstanceConfig::Full(FullInstanceConfig::Server {
			launch: LaunchConfig::default(),
			options: None,
			preset: Some(String::from("hello")),
			datapack_folder: None,
			snapshots: None,
		});
		read_instance_config("test", &config, &profile, &presets)
			.expect_err("Instance kinds should be incompatible");
	}

	#[test]
	fn test_quickplay_deser() {
		#[derive(Deserialize)]
		struct Test {
			quick_play: QuickPlay,
		}

		let test = serde_json::from_str::<Test>(
			r#"{
			"quick_play": {
				"type": "server",
				"server": "localhost",
				"port": 25565,
				"world": "test",
				"realm": "my_realm"
			}	
		}"#,
		)
		.unwrap();
		assert_eq!(
			test.quick_play,
			QuickPlay::Server {
				server: String::from("localhost"),
				port: Some(25565)
			}
		);
	}
}
