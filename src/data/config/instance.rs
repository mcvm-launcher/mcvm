use std::collections::HashMap;
use std::str::FromStr;

use anyhow::{anyhow, bail, ensure, Context};
use mcvm_shared::Side;
use serde::{Deserialize, Serialize};

use crate::data::id::InstanceID;
use crate::data::instance::launch::{LaunchOptions, WrapperCommand};
use crate::data::instance::{InstKind, Instance, InstanceStoredConfig};
use crate::data::profile::Profile;
use crate::io::java::args::{ArgsPreset, MemoryNum};
use crate::io::java::install::JavaInstallationKind;
use crate::io::options::client::ClientOptions;
use crate::io::options::server::ServerOptions;
use crate::io::snapshot;
use crate::util::merge_options;

use super::package::PackageConfig;

/// Different representations of configuration for an instance
#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
#[serde(rename_all = "snake_case")]
pub enum InstanceConfig {
	/// Simple configuration with just a side
	Simple(Side),
	/// Full configuration with all options available
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
					packages: Vec::new(),
				},
				Side::Server => FullInstanceConfig::Server {
					launch: LaunchConfig::default(),
					options: None,
					preset: None,
					datapack_folder: None,
					snapshots: None,
					packages: Vec::new(),
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

/// The full representation of instance config
#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum FullInstanceConfig {
	/// Config for the client
	Client {
		/// Launch configuration
		#[serde(default)]
		launch: LaunchConfig,
		/// Game options
		#[serde(default)]
		options: Option<Box<ClientOptions>>,
		/// Window configuration
		#[serde(default)]
		window: ClientWindowConfig,
		/// An instance preset to use
		#[serde(default)]
		preset: Option<String>,
		/// The folder for global datapacks to be installed to
		#[serde(default)]
		datapack_folder: Option<String>,
		/// Options for snapshot config
		#[serde(default)]
		snapshots: Option<snapshot::Config>,
		/// Packages for this instance
		#[serde(default)]
		packages: Vec<PackageConfig>,
	},
	/// Config for the server
	Server {
		/// Launch configuration
		#[serde(default)]
		launch: LaunchConfig,
		/// Game options
		#[serde(default)]
		options: Option<Box<ServerOptions>>,
		/// An instance preset to use
		#[serde(default)]
		preset: Option<String>,
		/// The folder for global datapacks to be installed to
		#[serde(default)]
		datapack_folder: Option<String>,
		/// Options for snapshot config
		#[serde(default)]
		snapshots: Option<snapshot::Config>,
		/// Packages for this instance
		#[serde(default)]
		packages: Vec<PackageConfig>,
	},
}

/// Different representations for JVM / game arguments
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum Args {
	/// A list of separate arguments
	List(Vec<String>),
	/// A single string of arguments
	String(String),
}

impl Args {
	/// Parse the arguments into a vector
	pub fn parse(&self) -> Vec<String> {
		match self {
			Self::List(vec) => vec.clone(),
			Self::String(string) => string.split(' ').map(|string| string.to_string()).collect(),
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

/// Arguments for the process when launching
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct LaunchArgs {
	/// Arguments for the JVM
	#[serde(default)]
	pub jvm: Args,
	/// Arguments for the game
	#[serde(default)]
	pub game: Args,
}

/// Different representations of both memory arguments for the JVM
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(untagged)]
pub enum LaunchMemory {
	/// No memory arguments
	#[default]
	None,
	/// A single memory argument shared for both
	Single(String),
	/// Different memory arguments for both
	Both {
		/// The minimum memory
		min: String,
		/// The maximum memory
		max: String,
	},
}

fn default_java() -> String {
	"adoptium".into()
}

fn default_flags_preset() -> String {
	"none".into()
}

/// Merges two lists of instance packages
fn merge_package_lists(mut a: Vec<PackageConfig>, b: Vec<PackageConfig>) -> Vec<PackageConfig> {
	a.extend(b);
	a
}

/// Options for the Minecraft QuickPlay feature
#[derive(Deserialize, Serialize, Debug, PartialEq, Default, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum QuickPlay {
	/// QuickPlay a world
	World {
		/// The world to play
		world: String,
	},
	/// QuickPlay a server
	Server {
		/// The server address to join
		server: String,
		/// The port for the server to connect to
		port: Option<u16>,
	},
	/// QuickPlay a realm
	Realm {
		/// The realm name to join
		realm: String,
	},
	/// Don't do any QuickPlay
	#[default]
	None,
}

/// Configuration for the launching of the game
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LaunchConfig {
	/// The arguments for the process
	#[serde(default)]
	pub args: LaunchArgs,
	/// JVM memory options
	#[serde(default)]
	pub memory: LaunchMemory,
	/// The java installation to use
	#[serde(default = "default_java")]
	pub java: String,
	/// The preset for flags
	#[serde(default = "default_flags_preset")]
	pub preset: String,
	/// Environment variables
	#[serde(default)]
	pub env: HashMap<String, String>,
	/// A wrapper command
	#[serde(default)]
	pub wrapper: Option<WrapperCommand>,
	/// QuickPlay options
	#[serde(default)]
	pub quick_play: QuickPlay,
	/// Whether or not to use the Log4J configuration
	#[serde(default)]
	pub use_log4j_config: bool,
}

impl LaunchConfig {
	/// Parse and finalize this LaunchConfig into LaunchOptions
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
			java: JavaInstallationKind::parse(&self.java),
			preset: ArgsPreset::from_str(&self.preset)?,
			env: self.env.clone(),
			wrapper: self.wrapper.clone(),
			quick_play: self.quick_play.clone(),
			use_log4j_config: self.use_log4j_config,
		})
	}

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
			use_log4j_config: false,
		}
	}
}

/// Resolution for a client window
#[derive(Deserialize, Serialize, Clone, Debug, Copy)]
pub struct WindowResolution {
	/// The width of the window
	pub width: u32,
	/// The height of the window
	pub height: u32,
}

/// Configuration for the client window
#[derive(Deserialize, Serialize, Default, Clone, Debug)]
#[serde(default)]
pub struct ClientWindowConfig {
	/// The resolution of the window
	pub resolution: Option<WindowResolution>,
}

impl ClientWindowConfig {
	/// Merge two ClientWindowConfigs
	pub fn merge(&mut self, other: Self) -> &mut Self {
		self.resolution = merge_options(self.resolution, other.resolution);
		self
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
				packages,
				..
			},
			FullInstanceConfig::Client {
				launch: launch2,
				options: options2,
				window: window2,
				datapack_folder: datapack_folder2,
				snapshots: snapshots2,
				packages: packages2,
				..
			},
		) => Ok::<FullInstanceConfig, anyhow::Error>(FullInstanceConfig::Client {
			launch: launch.merge(launch2).clone(),
			options: merge_options(options, options2),
			window: window.merge(window2).clone(),
			preset: None,
			datapack_folder: merge_options(datapack_folder, datapack_folder2),
			snapshots: merge_options(snapshots, snapshots2),
			packages: merge_package_lists(packages, packages2),
		}),
		(
			FullInstanceConfig::Server {
				mut launch,
				options,
				datapack_folder,
				snapshots,
				packages,
				..
			},
			FullInstanceConfig::Server {
				launch: launch2,
				options: options2,
				datapack_folder: datapack_folder2,
				snapshots: snapshots2,
				packages: packages2,
				..
			},
		) => Ok::<FullInstanceConfig, anyhow::Error>(FullInstanceConfig::Server {
			launch: launch.merge(launch2).clone(),
			options: merge_options(options, options2),
			preset: None,
			datapack_folder: merge_options(datapack_folder, datapack_folder2),
			snapshots: merge_options(snapshots, snapshots2),
			packages: merge_package_lists(packages, packages2),
		}),
		_ => bail!("Instance types do not match"),
	}?;

	Ok(InstanceConfig::Full(out))
}

/// Read the config for an instance to create the instance
pub fn read_instance_config(
	id: InstanceID,
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
	let (kind, launch, datapack_folder, snapshot_config, packages) = match config {
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
			Vec::new(),
		),
		InstanceConfig::Full(config) => match config {
			FullInstanceConfig::Client {
				launch,
				options,
				window,
				datapack_folder,
				snapshots,
				packages,
				..
			} => (
				InstKind::Client { options, window },
				launch,
				datapack_folder,
				snapshots,
				packages,
			),
			FullInstanceConfig::Server {
				launch,
				options,
				datapack_folder,
				snapshots,
				packages,
				..
			} => (
				InstKind::Server { options },
				launch,
				datapack_folder,
				snapshots,
				packages,
			),
		},
	};

	let instance = Instance::new(
		kind,
		id,
		InstanceStoredConfig {
			modifications: profile.modifications.clone(),
			launch: launch.to_options()?,
			datapack_folder,
			snapshot_config: snapshot_config.unwrap_or_default(),
			packages,
		},
	);

	Ok(instance)
}

#[cfg(test)]
mod tests {
	use super::*;

	use crate::data::config::profile::ProfilePackageConfiguration;
	use crate::data::{config::profile::GameModifications, id::ProfileID};
	use crate::util::versions::MinecraftVersion;
	use mcvm_shared::modifications::{ClientType, Modloader, ServerType};
	use mcvm_shared::pkg::PackageStability;

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
			ProfileID::from("foo"),
			MinecraftVersion::Latest,
			GameModifications::new(Modloader::Vanilla, ClientType::Vanilla, ServerType::Vanilla),
			ProfilePackageConfiguration::default(),
			PackageStability::Latest,
		);

		let instance = read_instance_config(
			InstanceID::from("foo"),
			&test.instance,
			&profile,
			&HashMap::new(),
		)
		.unwrap();
		assert_eq!(instance.id, InstanceID::from("foo"));
		assert!(matches!(instance.kind, InstKind::Client { .. }));
	}

	#[test]
	fn test_instance_config_merging() {
		let presets = {
			let mut presets = HashMap::new();
			presets.insert(
				"hello".into(),
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
					packages: Vec::new(),
				}),
			);
			presets
		};

		let profile = Profile::new(
			ProfileID::from("foo"),
			MinecraftVersion::Latest,
			GameModifications::new(Modloader::Vanilla, ClientType::Vanilla, ServerType::Vanilla),
			ProfilePackageConfiguration::default(),
			PackageStability::Latest,
		);

		let config = InstanceConfig::Full(FullInstanceConfig::Client {
			launch: LaunchConfig::default(),
			options: None,
			window: ClientWindowConfig::default(),
			preset: Some("hello".into()),
			datapack_folder: None,
			snapshots: None,
			packages: Vec::new(),
		});
		let instance = read_instance_config(InstanceID::from("test"), &config, &profile, &presets)
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
			preset: Some("hello".into()),
			datapack_folder: None,
			snapshots: None,
			packages: Vec::new(),
		});
		read_instance_config(InstanceID::from("test"), &config, &profile, &presets)
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
				server: "localhost".into(),
				port: Some(25565)
			}
		);
	}
}
