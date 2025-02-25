use std::collections::HashMap;

use anyhow::{bail, ensure, Context};
use mcvm_core::io::java::args::MemoryNum;
use mcvm_core::io::java::install::JavaInstallationKind;
use mcvm_core::util::versions::MinecraftVersionDeser;
use mcvm_plugin::hooks::ModifyInstanceConfig;
use mcvm_shared::id::{InstanceID, ProfileID};
use mcvm_shared::modifications::{ClientType, Modloader, ServerType};
use mcvm_shared::output::MCVMOutput;
use mcvm_shared::pkg::PackageStability;
use mcvm_shared::util::{merge_options, DefaultExt, DeserListOrSingle};
use mcvm_shared::Side;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::instance::launch::{LaunchOptions, WrapperCommand};
use crate::instance::{InstKind, Instance, InstanceStoredConfig};
use crate::io::paths::Paths;

use super::package::{PackageConfig, PackageConfigDeser, PackageConfigSource};
use super::profile::{GameModifications, ProfileConfig};
use crate::plugin::PluginManager;

/// Configuration for an instance
#[derive(Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct InstanceConfig {
	/// The type or side of this instance
	#[serde(rename = "type")]
	pub side: Option<Side>,
	/// The display name of this instance
	#[serde(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	/// A path to an icon file for this instance
	#[serde(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub icon: Option<String>,
	/// The common config of this instance
	#[serde(flatten)]
	pub common: CommonInstanceConfig,
	/// Window configuration
	#[serde(default)]
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub window: ClientWindowConfig,
}

/// Common full instance config for both client and server
#[derive(Deserialize, Serialize, Clone, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct CommonInstanceConfig {
	/// A profile to use
	#[serde(skip_serializing_if = "DeserListOrSingle::is_empty")]
	pub from: DeserListOrSingle<String>,
	/// The Minecraft version
	pub version: Option<MinecraftVersionDeser>,
	/// Configured modloader
	#[serde(default)]
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub modloader: Option<Modloader>,
	/// Configured client type
	#[serde(default)]
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub client_type: Option<ClientType>,
	/// Configured server type
	#[serde(default)]
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub server_type: Option<ServerType>,
	/// Default stability setting of packages on this instance
	#[serde(default)]
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub package_stability: Option<PackageStability>,
	/// Launch configuration
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub launch: LaunchConfig,
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
		self.from.merge(other.from);
		self.version = other.version.or(self.version.clone());
		self.modloader = other.modloader.or(self.modloader.clone());
		self.client_type = other.client_type.or(self.client_type.clone());
		self.server_type = other.server_type.or(self.server_type.clone());
		self.package_stability = other.package_stability.or(self.package_stability);
		self.launch.merge(other.launch);
		self.datapack_folder = other.datapack_folder.or(self.datapack_folder.clone());
		self.packages.extend(other.packages);
		mcvm_core::util::json::merge_objects(&mut self.plugin_config, other.plugin_config);

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
pub fn merge_instance_configs(preset: &InstanceConfig, config: InstanceConfig) -> InstanceConfig {
	let mut out = preset.clone();
	out.common.merge(config.common);
	out.name = config.name.or(out.name);
	out.side = config.side.or(out.side);
	out.window.merge(config.window);

	out
}

/// Read the config for an instance to create the instance
pub fn read_instance_config(
	id: InstanceID,
	mut config: InstanceConfig,
	profiles: &HashMap<ProfileID, ProfileConfig>,
	plugins: &PluginManager,
	paths: &Paths,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<Instance> {
	if !is_valid_instance_id(&id) {
		bail!("Invalid instance ID '{}'", id.to_string());
	}

	// Get the parent profile if it is specified
	let profiles: anyhow::Result<Vec<_>> = config
		.common
		.from
		.iter()
		.map(|x| {
			profiles
				.get(&ProfileID::from(x.clone()))
				.with_context(|| format!("Derived profile '{x}' does not exist"))
		})
		.collect();
	let profiles = profiles?;

	// Merge with the profile
	for profile in &profiles {
		config = merge_instance_configs(&profile.instance, config);
	}

	let side = config.side.context("Instance type was not specified")?;

	// Consolidate all of the package configs into the instance package config list
	let packages = consolidate_package_configs(profiles, &config, side);

	let kind = match side {
		Side::Client => InstKind::client(config.window),
		Side::Server => InstKind::server(),
	};

	let game_modifications = GameModifications::new(
		config.common.modloader.clone().unwrap_or_default(),
		config.common.client_type.clone().unwrap_or_default(),
		config.common.server_type.clone().unwrap_or_default(),
	);

	let version = config
		.common
		.version
		.clone()
		.context("Instance is missing a Minecraft version")?
		.to_mc_version();

	// Apply plugins
	let results = plugins
		.call_hook(ModifyInstanceConfig, &config.common.plugin_config, paths, o)
		.context("Failed to apply plugin instance modifications")?;
	for result in results {
		let result = result.result(o)?;
		config
			.common
			.launch
			.args
			.jvm
			.merge(Args::List(result.additional_jvm_args));
	}

	let stored_config = InstanceStoredConfig {
		name: config.name,
		icon: config.icon,
		version,
		modifications: game_modifications,
		launch: config.common.launch.to_options()?,
		datapack_folder: config.common.datapack_folder,
		packages,
		package_stability: config.common.package_stability.unwrap_or_default(),
		plugin_config: config.common.plugin_config,
	};

	let instance = Instance::new(kind, id, stored_config);

	Ok(instance)
}

/// Checks if an instance ID is valid
pub fn is_valid_instance_id(id: &str) -> bool {
	for c in id.chars() {
		if !c.is_ascii() {
			return false;
		}

		if c.is_ascii_punctuation() {
			match c {
				'_' | '-' | '.' | ':' => {}
				_ => return false,
			}
		}

		if c.is_ascii_whitespace() {
			return false;
		}
	}

	true
}

/// Combines all of the package configs from global, profile, and instance together into
/// the configurations for just one instance
fn consolidate_package_configs(
	profiles: Vec<&ProfileConfig>,
	instance: &InstanceConfig,
	side: Side,
) -> Vec<PackageConfig> {
	let stability = instance.common.package_stability.unwrap_or_default();
	// We use a map so that we can override packages from more general sources
	// with those from more specific ones
	let mut map = HashMap::new();
	for profile in profiles {
		for pkg in profile.packages.iter_global() {
			let pkg = pkg
				.clone()
				.to_package_config(stability, PackageConfigSource::Profile);
			map.insert(pkg.id.clone(), pkg);
		}
		for pkg in profile.packages.iter_side(side) {
			let pkg = pkg
				.clone()
				.to_package_config(stability, PackageConfigSource::Profile);
			map.insert(pkg.id.clone(), pkg);
		}
	}
	for pkg in &instance.common.packages {
		let pkg = pkg
			.clone()
			.to_package_config(stability, PackageConfigSource::Instance);
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
