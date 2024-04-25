/// Writing to the server.properties file
mod file;

use mcvm_shared::util::{DefaultExt, ToInt};

pub use file::create_keys;
pub use file::get_world_name;
pub use file::write_server_properties;

use std::collections::HashMap;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::read::{EnumOrNumber, EnumOrString};

// I do not want to document all of these
pub use deser::*;
#[allow(missing_docs)]
pub mod deser {
	#[cfg(feature = "schema")]
	use schemars::JsonSchema;

	use super::*;

	#[derive(Deserialize, Serialize, Debug, Clone, Default)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
	#[serde(default)]
	pub struct ServerOptions {
		#[serde(skip_serializing_if = "DefaultExt::is_default")]
		pub rcon: RconOptions,
		#[serde(skip_serializing_if = "DefaultExt::is_default")]
		pub query: QueryOptions,
		#[serde(skip_serializing_if = "DefaultExt::is_default")]
		pub whitelist: WhitelistOptions,
		#[serde(skip_serializing_if = "DefaultExt::is_default")]
		pub gamemode: GamemodeOptions,
		#[serde(skip_serializing_if = "DefaultExt::is_default")]
		pub datapacks: DatapacksOptions,
		#[serde(skip_serializing_if = "DefaultExt::is_default")]
		pub world: WorldOptions,
		#[serde(skip_serializing_if = "DefaultExt::is_default")]
		pub resource_pack: ResourcePackOptions,
		#[serde(skip_serializing_if = "HashMap::is_empty")]
		pub custom: HashMap<String, String>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub allow_flight: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub broadcast_console_to_ops: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub broadcast_rcon_to_ops: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub difficulty: Option<EnumOrNumber<Difficulty>>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub allow_command_blocks: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub jmx_monitoring: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub enable_status: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub enforce_secure_profile: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub entity_broadcast_range: Option<u16>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub hardcore: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub hide_online_players: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub max_chained_neighbor_updates: Option<i32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub max_players: Option<u32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub max_tick_time: Option<u64>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub motd: Option<String>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub network_compression_threshold: Option<EnumOrNumber<NetworkCompression>>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub offline_mode: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub op_permission_level: Option<u8>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub player_idle_timeout: Option<u32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub prevent_proxy_connections: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub enable_chat_preview: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub enable_pvp: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub rate_limit: Option<i16>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub ip: Option<String>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub port: Option<u16>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub simulation_distance: Option<u8>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub enable_snooper: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub spawn_animals: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub spawn_monsters: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub spawn_npcs: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub spawn_protection: Option<u32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub sync_chunk_writes: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub use_native_transport: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub view_distance: Option<u8>,
	}

	#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
	#[serde(default)]
	pub struct RconOptions {
		#[serde(skip_serializing_if = "Option::is_none")]
		pub enable: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub port: Option<u16>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub password: Option<String>,
	}

	#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
	#[serde(default)]
	pub struct QueryOptions {
		#[serde(skip_serializing_if = "Option::is_none")]
		pub enable: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub port: Option<u16>,
	}

	#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
	#[serde(default)]
	pub struct WhitelistOptions {
		#[serde(skip_serializing_if = "Option::is_none")]
		pub enable: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub enforce: Option<bool>,
	}

	#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
	#[serde(default)]
	pub struct GamemodeOptions {
		#[serde(skip_serializing_if = "Option::is_none")]
		pub default: Option<EnumOrNumber<GameMode>>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub force: Option<bool>,
	}

	#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
	#[serde(default)]
	pub struct DatapacksOptions {
		#[serde(skip_serializing_if = "Option::is_none")]
		pub function_permission_level: Option<u8>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub initial_enabled: Option<Vec<String>>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub initial_disabled: Option<Vec<String>>,
	}

	#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
	#[serde(default)]
	pub struct WorldOptions {
		#[serde(skip_serializing_if = "Option::is_none")]
		pub name: Option<String>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub seed: Option<String>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub r#type: Option<EnumOrString<WorldType>>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub structures: Option<bool>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub generator_settings: Option<serde_json::Map<String, serde_json::Value>>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub max_size: Option<u32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub max_build_height: Option<u32>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub allow_nether: Option<bool>,
	}

	#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
	#[serde(default)]
	pub struct ResourcePackOptions {
		#[serde(skip_serializing_if = "Option::is_none")]
		pub uri: Option<String>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub prompt: Option<String>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub sha1: Option<String>,
		#[serde(skip_serializing_if = "Option::is_none")]
		pub required: Option<bool>,
	}

	#[derive(Deserialize, Serialize, Debug, Clone)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
	#[serde(rename_all = "snake_case")]
	pub enum Difficulty {
		Peaceful,
		Easy,
		Normal,
		Hard,
	}

	impl Display for Difficulty {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(
				f,
				"{}",
				match self {
					Self::Peaceful => "peaceful",
					Self::Easy => "easy",
					Self::Normal => "normal",
					Self::Hard => "hard",
				}
			)
		}
	}

	impl ToInt for Difficulty {
		fn to_int(&self) -> i32 {
			self.clone() as i32
		}
	}

	#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
	#[serde(rename_all = "snake_case")]
	pub enum GameMode {
		Survival,
		Creative,
		Adventure,
		Spectator,
	}

	impl Display for GameMode {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(
				f,
				"{}",
				match self {
					Self::Survival => "survival",
					Self::Creative => "creative",
					Self::Adventure => "adventure",
					Self::Spectator => "spectator",
				}
			)
		}
	}

	impl ToInt for GameMode {
		fn to_int(&self) -> i32 {
			self.clone() as i32
		}
	}

	#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
	#[serde(rename_all = "snake_case")]
	pub enum WorldType {
		Normal,
		Flat,
		LargeBiomes,
		Amplified,
		SingleBiome,
		Buffet,
		Custom,
	}

	impl Display for WorldType {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(
				f,
				"{}",
				match self {
					WorldType::Normal => "minecraft:normal",
					WorldType::Flat => "minecraft:flat",
					WorldType::LargeBiomes => "miecraft:large_biomes",
					WorldType::Amplified => "minecraft:amplified",
					WorldType::SingleBiome => "minecraft:single_biome_surface",
					WorldType::Buffet => "buffet",
					WorldType::Custom => "customized",
				}
			)
		}
	}

	#[derive(Deserialize, Serialize, Debug, Clone)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
	#[serde(rename_all = "snake_case")]
	pub enum NetworkCompression {
		Disabled,
		All,
	}

	impl ToInt for NetworkCompression {
		fn to_int(&self) -> i32 {
			self.clone() as i32
		}
	}
}
