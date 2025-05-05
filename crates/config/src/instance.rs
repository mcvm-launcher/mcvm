use std::collections::HashMap;

use mcvm_core::util::versions::MinecraftVersionDeser;
use mcvm_shared::modifications::{ClientType, Modloader, ServerType};
use mcvm_shared::pkg::PackageStability;
use mcvm_shared::util::{merge_options, DefaultExt, DeserListOrSingle};
use mcvm_shared::versions::VersionPattern;
use mcvm_shared::Side;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::package::PackageConfigDeser;

/// Configuration for an instance
#[derive(Deserialize, Serialize, Clone, Debug)]
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
#[derive(Deserialize, Serialize, Clone, Default, Debug)]
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
	/// The version of whatever game modification is applied to this instance
	#[serde(default)]
	#[serde(skip_serializing_if = "DefaultExt::is_default")]
	pub game_modification_version: Option<VersionPattern>,
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

/// A wrapper command
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct WrapperCommand {
	/// The command to run
	pub cmd: String,
	/// The command's arguments
	#[serde(default)]
	pub args: Vec<String>,
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

/// Game modifications
#[derive(Clone, Debug)]
pub struct GameModifications {
	modloader: Modloader,
	/// Type of the client
	client_type: ClientType,
	/// Type of the server
	server_type: ServerType,
}

impl GameModifications {
	/// Create a new GameModifications
	pub fn new(modloader: Modloader, client_type: ClientType, server_type: ServerType) -> Self {
		Self {
			modloader,
			client_type,
			server_type,
		}
	}

	/// Gets the client type
	pub fn client_type(&self) -> ClientType {
		if let ClientType::None = self.client_type {
			match &self.modloader {
				Modloader::Vanilla => ClientType::Vanilla,
				Modloader::Forge => ClientType::Forge,
				Modloader::NeoForged => ClientType::NeoForged,
				Modloader::Fabric => ClientType::Fabric,
				Modloader::Quilt => ClientType::Quilt,
				Modloader::LiteLoader => ClientType::LiteLoader,
				Modloader::Risugamis => ClientType::Risugamis,
				Modloader::Rift => ClientType::Rift,
				Modloader::Unknown(modloader) => ClientType::Unknown(modloader.clone()),
			}
		} else {
			self.client_type.clone()
		}
	}

	/// Gets the server type
	pub fn server_type(&self) -> ServerType {
		if let ServerType::None = self.server_type {
			match &self.modloader {
				Modloader::Vanilla => ServerType::Vanilla,
				Modloader::Forge => ServerType::Forge,
				Modloader::NeoForged => ServerType::NeoForged,
				Modloader::Fabric => ServerType::Fabric,
				Modloader::Quilt => ServerType::Quilt,
				Modloader::LiteLoader => ServerType::Unknown("liteloader".into()),
				Modloader::Risugamis => ServerType::Risugamis,
				Modloader::Rift => ServerType::Rift,
				Modloader::Unknown(modloader) => ServerType::Unknown(modloader.clone()),
			}
		} else {
			self.server_type.clone()
		}
	}

	/// Gets the modloader of a side
	pub fn get_modloader(&self, side: Side) -> Modloader {
		match side {
			Side::Client => match self.client_type {
				ClientType::None => self.modloader.clone(),
				ClientType::Vanilla => Modloader::Vanilla,
				ClientType::Forge => Modloader::Forge,
				ClientType::NeoForged => Modloader::NeoForged,
				ClientType::Fabric => Modloader::Fabric,
				ClientType::Quilt => Modloader::Quilt,
				ClientType::LiteLoader => Modloader::LiteLoader,
				ClientType::Risugamis => Modloader::Risugamis,
				ClientType::Rift => Modloader::Rift,
				_ => Modloader::Vanilla,
			},
			Side::Server => match self.server_type {
				ServerType::None => self.modloader.clone(),
				ServerType::Forge | ServerType::SpongeForge => Modloader::Forge,
				ServerType::NeoForged => Modloader::NeoForged,
				ServerType::Fabric => Modloader::Fabric,
				ServerType::Quilt => Modloader::Quilt,
				ServerType::Risugamis => Modloader::Risugamis,
				ServerType::Rift => Modloader::Rift,
				_ => Modloader::Vanilla,
			},
		}
	}

	/// Gets whether both client and server have the same modloader
	pub fn common_modloader(&self) -> bool {
		matches!(
			(&self.client_type, &self.server_type),
			(ClientType::None, ServerType::None)
				| (ClientType::Vanilla, ServerType::Vanilla)
				| (ClientType::Forge, ServerType::Forge)
				| (ClientType::NeoForged, ServerType::NeoForged)
				| (ClientType::Fabric, ServerType::Fabric)
				| (ClientType::Quilt, ServerType::Quilt)
				| (ClientType::Risugamis, ServerType::Risugamis)
				| (ClientType::Rift, ServerType::Rift)
		)
	}
}

/// Check if a client type can be installed by MCVM
pub fn can_install_client_type(client_type: &ClientType) -> bool {
	matches!(client_type, ClientType::None | ClientType::Vanilla)
}

/// Check if a server type can be installed by MCVM
pub fn can_install_server_type(server_type: &ServerType) -> bool {
	matches!(
		server_type,
		ServerType::None
			| ServerType::Vanilla
			| ServerType::Paper
			| ServerType::Folia
			| ServerType::Sponge
	)
}
