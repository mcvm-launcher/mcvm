use std::collections::HashMap;

use anyhow::{anyhow, bail, ensure, Context};
use mcvm_core::io::java::args::MemoryNum;
use mcvm_core::io::java::install::JavaInstallationKind;
use mcvm_plugin::hooks::ModifyInstanceConfig;
use mcvm_shared::id::InstanceID;
use mcvm_shared::output::MCVMOutput;
use mcvm_shared::pkg::PackageStability;
use mcvm_shared::util::{merge_options, DefaultExt};
use mcvm_shared::Side;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::data::instance::launch::{LaunchOptions, WrapperCommand};
use crate::data::instance::{InstKind, Instance, InstanceStoredConfig};
use crate::data::profile::Profile;
use crate::io::files::paths::Paths;

use super::package::{PackageConfig, PackageConfigDeser, PackageConfigSource};
use super::plugin::PluginManager;

/// Different representations of configuration for an instance
#[derive(Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
					window: ClientWindowConfig::default(),
					common: CommonInstanceConfig::default(),
				},
				Side::Server => FullInstanceConfig::Server {
					common: CommonInstanceConfig::default(),
				},
			},
		}
	}

	/// Gets the common config
	pub fn get_common_config(&self) -> Option<&CommonInstanceConfig> {
		match self {
			Self::Full(
				FullInstanceConfig::Client { common, .. }
				| FullInstanceConfig::Server { common, .. },
			) => Some(common),
			_ => None,
		}
	}

	/// Checks if this config has the preset field filled out
	pub fn uses_preset(&self) -> bool {
		let config = self.get_common_config();
		if let Some(config) = config {
			config.preset.is_some()
		} else {
			false
		}
	}
}

/// The full representation of instance config
#[derive(Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum FullInstanceConfig {
	/// Config for the client
	Client {
		/// Common configuration
		#[serde(flatten)]
		common: CommonInstanceConfig,
		/// Window configuration
		#[serde(default)]
		#[serde(skip_serializing_if = "DefaultExt::is_default")]
		window: ClientWindowConfig,
	},
	/// Config for the server
	Server {
		/// Common configuration
		#[serde(flatten)]
		common: CommonInstanceConfig,
	},
}

/// Common full instance config for both client and server
#[derive(Deserialize, Serialize, Clone, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct CommonInstanceConfig {
	/// Launch configuration
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub launch: LaunchConfig,
	/// An instance preset to use
	#[serde(skip_serializing_if = "Option::is_none")]
	pub preset: Option<String>,
	/// The folder for global datapacks to be installed to
	#[serde(skip_serializing_if = "Option::is_none")]
	pub datapack_folder: Option<String>,
	/// Packages for this instance
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub packages: Vec<PackageConfigDeser>,
	/// Config for plugins
	#[serde(flatten)]
	#[serde(skip_serializing_if = "serde_json::Map::is_empty")]
	pub plugin_config: serde_json::Map<String, serde_json::Value>,
}

impl CommonInstanceConfig {
	/// Merge multiple common configs
	pub fn merge(&mut self, other: Self) -> &mut Self {
		self.launch.merge(other.launch);
		self.preset = merge_options(self.preset.clone(), other.preset);
		self.datapack_folder = merge_options(self.datapack_folder.clone(), other.datapack_folder);
		self.packages.extend(other.packages);

		self
	}
}

/// Different representations for JVM / game arguments
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct LaunchArgs {
	/// Arguments for the JVM
	#[serde(default)]
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub jvm: Args,
	/// Arguments for the game
	#[serde(default)]
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub game: Args,
}

/// Different representations of both memory arguments for the JVM
#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
	"auto".into()
}

fn default_flags_preset() -> String {
	"none".into()
}

/// Options for the Minecraft QuickPlay feature
#[derive(Deserialize, Serialize, Debug, PartialEq, Default, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct LaunchConfig {
	/// The arguments for the process
	#[serde(default)]
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub args: LaunchArgs,
	/// JVM memory options
	#[serde(default)]
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub memory: LaunchMemory,
	/// The java installation to use
	#[serde(default = "default_java")]
	pub java: String,
	/// The preset for flags
	#[serde(default = "default_flags_preset")]
	pub preset: String,
	/// Environment variables
	#[serde(default)]
	#[serde(skip_serializing_if = "HashMap::is_empty")]
	pub env: HashMap<String, String>,
	/// A wrapper command
	#[serde(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub wrapper: Option<WrapperCommand>,
	/// QuickPlay options
	#[serde(default)]
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub quick_play: QuickPlay,
	/// Whether or not to use the Log4J configuration
	#[serde(default)]
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub use_log4j_config: bool,
}

impl LaunchConfig {
	/// Parse and finalize this LaunchConfig into LaunchOptions
	pub fn to_options(self) -> anyhow::Result<LaunchOptions> {
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
			env: self.env,
			wrapper: self.wrapper,
			quick_play: self.quick_play,
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
#[derive(Deserialize, Serialize, Clone, Debug, Copy, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct WindowResolution {
	/// The width of the window
	pub width: u32,
	/// The height of the window
	pub height: u32,
}

/// Configuration for the client window
#[derive(Deserialize, Serialize, Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct ClientWindowConfig {
	/// The resolution of the window
	#[serde(skip_serializing_if = "Option::is_none")]
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
	config: FullInstanceConfig,
) -> anyhow::Result<FullInstanceConfig> {
	let mut out = preset.make_full();
	let applied = config;
	out = match (out, applied) {
		(
			FullInstanceConfig::Client {
				mut window,
				mut common,
				..
			},
			FullInstanceConfig::Client {
				window: window2,
				common: common2,
				..
			},
		) => Ok::<FullInstanceConfig, anyhow::Error>(FullInstanceConfig::Client {
			window: {
				window.merge(window2);
				window
			},
			common: {
				common.merge(common2);
				common
			},
		}),
		(
			FullInstanceConfig::Server { mut common, .. },
			FullInstanceConfig::Server {
				common: common2, ..
			},
		) => Ok::<FullInstanceConfig, anyhow::Error>(FullInstanceConfig::Server {
			common: {
				common.merge(common2);
				common
			},
		}),
		_ => bail!("Instance types do not match"),
	}?;

	Ok(out)
}

/// Read the config for an instance to create the instance
pub fn read_instance_config(
	id: InstanceID,
	config: &InstanceConfig,
	profile: &Profile,
	global_packages: &[PackageConfigDeser],
	presets: &HashMap<String, InstanceConfig>,
	plugins: &PluginManager,
	paths: &Paths,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<Instance> {
	let config = config.make_full();
	let config = if let FullInstanceConfig::Client {
		common: CommonInstanceConfig {
			preset: Some(ref preset),
			..
		},
		..
	}
	| FullInstanceConfig::Server {
		common: CommonInstanceConfig {
			preset: Some(ref preset),
			..
		},
		..
	} = config
	{
		let preset = presets
			.get(preset)
			.ok_or(anyhow!("Preset '{preset}' does not exist"))?;
		merge_instance_configs(preset, config).context("Failed to merge preset with instance")?
	} else {
		config
	};
	let (kind, mut common) = match config {
		FullInstanceConfig::Client { window, common } => (InstKind::client(window), common),
		FullInstanceConfig::Server { common } => (InstKind::server(), common),
	};

	// Apply plugins
	let results = plugins
		.call_hook(ModifyInstanceConfig, &common.plugin_config, paths, o)
		.context("Failed to apply plugin instance modifications")?;
	for result in results {
		let result = result.result(o)?;
		common
			.launch
			.args
			.jvm
			.merge(Args::List(result.additional_jvm_args));
	}

	// Consolidate all of the package configs into the instance package config list
	let packages =
		consolidate_package_configs(profile, global_packages, &common.packages, kind.to_side());

	let stored_config = InstanceStoredConfig {
		modifications: profile.modifications.clone(),
		launch: common.launch.to_options()?,
		datapack_folder: common.datapack_folder,
		packages,
		plugin_config: common.plugin_config,
	};

	let instance = Instance::new(kind, id, profile.id.clone(), stored_config);

	Ok(instance)
}

/// Combines all of the package configs from global, profile, and instance together into
/// the configurations for just one instance
fn consolidate_package_configs(
	profile: &Profile,
	global_packages: &[PackageConfigDeser],
	instance_packages: &[PackageConfigDeser],
	side: Side,
) -> Vec<PackageConfig> {
	// We use a map so that we can override packages from more general sources
	// with those from more specific ones
	let mut map = HashMap::new();
	for pkg in global_packages {
		let pkg = pkg
			.clone()
			.to_package_config(PackageStability::default(), PackageConfigSource::Global);
		map.insert(pkg.id.clone(), pkg);
	}
	for pkg in profile.packages.iter_global() {
		let pkg = pkg
			.clone()
			.to_package_config(profile.default_stability, PackageConfigSource::Profile);
		map.insert(pkg.id.clone(), pkg);
	}
	for pkg in profile.packages.iter_side(side) {
		let pkg = pkg
			.clone()
			.to_package_config(profile.default_stability, PackageConfigSource::Profile);
		map.insert(pkg.id.clone(), pkg);
	}
	for pkg in instance_packages {
		let pkg = pkg
			.clone()
			.to_package_config(profile.default_stability, PackageConfigSource::Instance);
		map.insert(pkg.id.clone(), pkg);
	}

	let mut out = Vec::new();
	for pkg in map.values() {
		out.push(pkg.clone());
	}

	out
}

#[cfg(test)]
mod tests {
	use super::*;

	use crate::data::config::profile::GameModifications;
	use crate::data::config::profile::ProfilePackageConfiguration;
	use mcvm_core::util::versions::MinecraftVersion;
	use mcvm_plugin::api::NoOp;
	use mcvm_shared::id::ProfileID;
	use mcvm_shared::modifications::{ClientType, Modloader, Proxy, ServerType};
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
			GameModifications::new(
				Modloader::Vanilla,
				ClientType::Vanilla,
				ServerType::Vanilla,
				Proxy::None,
			),
			ProfilePackageConfiguration::default(),
			PackageStability::Latest,
		);

		let paths = Paths::new_no_create().unwrap();

		let instance = read_instance_config(
			InstanceID::from("foo"),
			&test.instance,
			&profile,
			&[],
			&HashMap::new(),
			&PluginManager::new(),
			&paths,
			&mut NoOp,
		)
		.unwrap();
		assert_eq!(instance.id, InstanceID::from("foo"));
		assert!(matches!(instance.kind, InstKind::Client { .. }));
	}

	#[test]
	fn test_instance_config_merging() {
		let paths = Paths::new_no_create().unwrap();

		let presets = {
			let mut presets = HashMap::new();
			presets.insert(
				"hello".into(),
				InstanceConfig::Full(FullInstanceConfig::Client {
					window: ClientWindowConfig {
						resolution: Some(WindowResolution {
							width: 200,
							height: 100,
						}),
					},
					common: CommonInstanceConfig::default(),
				}),
			);
			presets
		};

		let profile = Profile::new(
			ProfileID::from("foo"),
			MinecraftVersion::Latest,
			GameModifications::new(
				Modloader::Vanilla,
				ClientType::Vanilla,
				ServerType::Vanilla,
				Proxy::None,
			),
			ProfilePackageConfiguration::default(),
			PackageStability::Latest,
		);

		let config = InstanceConfig::Full(FullInstanceConfig::Client {
			window: ClientWindowConfig::default(),
			common: CommonInstanceConfig {
				preset: Some("hello".into()),
				..Default::default()
			},
		});
		let instance = read_instance_config(
			InstanceID::from("test"),
			&config,
			&profile,
			&[],
			&presets,
			&PluginManager::new(),
			&paths,
			&mut NoOp,
		)
		.expect("Failed to read instance config");
		if !matches!(
			instance.kind,
			InstKind::Client {
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
			common: CommonInstanceConfig {
				preset: Some("hello".into()),
				..Default::default()
			},
		});
		read_instance_config(
			InstanceID::from("test"),
			&config,
			&profile,
			&[],
			&presets,
			&PluginManager::new(),
			&paths,
			&mut NoOp,
		)
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
