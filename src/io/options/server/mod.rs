mod file;

pub use file::create_keys;
pub use file::write_server_properties;

use std::fmt::Display;

use serde::Deserialize;

use crate::util::{json, ToInt};

use super::read::{EnumOrNumber, EnumOrString};

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct RconOptions {
	pub enable: Option<bool>,
	pub port: Option<u16>,
	pub password: Option<String>,
}

impl Default for RconOptions {
	fn default() -> Self {
		Self {
			enable: None,
			port: None,
			password: None,
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct QueryOptions {
	pub enable: Option<bool>,
	pub port: Option<u16>,
}

impl Default for QueryOptions {
	fn default() -> Self {
		Self {
			enable: None,
			port: None,
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct WhitelistOptions {
	pub enable: Option<bool>,
	pub enforce: Option<bool>,
}

impl Default for WhitelistOptions {
	fn default() -> Self {
		Self {
			enable: None,
			enforce: None,
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct GamemodeOptions {
	pub default: Option<EnumOrNumber<GameMode>>,
	pub force: Option<bool>,
}

impl Default for GamemodeOptions {
	fn default() -> Self {
		Self {
			default: None,
			force: None,
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct DatapacksOptions {
	pub function_permission_level: Option<u8>,
	pub initial_enabled: Option<Vec<String>>,
	pub initial_disabled: Option<Vec<String>>,
}

impl Default for DatapacksOptions {
	fn default() -> Self {
		Self {
			function_permission_level: None,
			initial_enabled: None,
			initial_disabled: None,
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct WorldOptions {
	pub name: Option<String>,
	pub seed: Option<String>,
	pub r#type: Option<EnumOrString<WorldType>>,
	pub structures: Option<bool>,
	pub generator_settings: Option<json::JsonObject>,
	pub max_size: Option<u32>,
	pub max_build_height: Option<u32>,
	pub allow_nether: Option<bool>,
}

impl Default for WorldOptions {
	fn default() -> Self {
		Self {
			name: None,
			seed: None,
			r#type: None,
			structures: None,
			generator_settings: None,
			max_size: None,
			max_build_height: None,
			allow_nether: None,
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct ResourcePackOptions {
	pub uri: Option<String>,
	pub prompt: Option<String>,
	pub sha1: Option<String>,
	pub required: Option<bool>,
}

impl Default for ResourcePackOptions {
	fn default() -> Self {
		Self {
			uri: None,
			prompt: None,
			sha1: None,
			required: None,
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct ServerOptions {
	pub rcon: RconOptions,
	pub query: QueryOptions,
	pub whitelist: WhitelistOptions,
	pub gamemode: GamemodeOptions,
	pub datapacks: DatapacksOptions,
	pub world: WorldOptions,
	pub resource_pack: ResourcePackOptions,
	pub allow_flight: Option<bool>,
	pub broadcast_console_to_ops: Option<bool>,
	pub broadcast_rcon_to_ops: Option<bool>,
	pub difficulty: Option<EnumOrNumber<Difficulty>>,
	pub allow_command_blocks: Option<bool>,
	pub jmx_monitoring: Option<bool>,
	pub enable_status: Option<bool>,
	pub enforce_secure_profile: Option<bool>,
	pub entity_broadcast_range: Option<u16>,
	pub hardcore: Option<bool>,
	pub hide_online_players: Option<bool>,
	pub max_chained_neighbor_updates: Option<i32>,
	pub max_players: Option<u32>,
	pub max_tick_time: Option<u64>,
	pub motd: Option<String>,
	pub network_compression_threshold: Option<EnumOrNumber<NetworkCompression>>,
	pub offline_mode: Option<bool>,
	pub op_permission_level: Option<u8>,
	pub player_idle_timeout: Option<u32>,
	pub prevent_proxy_connections: Option<bool>,
	pub enable_chat_preview: Option<bool>,
	pub enable_pvp: Option<bool>,
	pub rate_limit: Option<i16>,
	pub ip: Option<String>,
	pub port: Option<u16>,
	pub simulation_distance: Option<u8>,
	pub enable_snooper: Option<bool>,
	pub spawn_animals: Option<bool>,
	pub spawn_monsters: Option<bool>,
	pub spawn_npcs: Option<bool>,
	pub spawn_protection: Option<u32>,
	pub sync_chunk_writes: Option<bool>,
	pub use_native_transport: Option<bool>,
	pub view_distance: Option<u8>,
}

impl Default for ServerOptions {
	fn default() -> Self {
		Self {
			rcon: RconOptions::default(),
			query: QueryOptions::default(),
			whitelist: WhitelistOptions::default(),
			gamemode: GamemodeOptions::default(),
			datapacks: DatapacksOptions::default(),
			world: WorldOptions::default(),
			resource_pack: ResourcePackOptions::default(),
			allow_flight: None,
			broadcast_console_to_ops: None,
			broadcast_rcon_to_ops: None,
			difficulty: None,
			allow_command_blocks: None,
			jmx_monitoring: None,
			enable_status: None,
			enforce_secure_profile: None,
			entity_broadcast_range: None,
			hardcore: None,
			hide_online_players: None,
			max_chained_neighbor_updates: None,
			max_players: None,
			max_tick_time: None,
			motd: None,
			network_compression_threshold: None,
			offline_mode: None,
			op_permission_level: None,
			player_idle_timeout: None,
			prevent_proxy_connections: None,
			enable_chat_preview: None,
			enable_pvp: None,
			rate_limit: None,
			ip: None,
			port: None,
			simulation_distance: None,
			enable_snooper: None,
			spawn_animals: None,
			spawn_monsters: None,
			spawn_npcs: None,
			spawn_protection: None,
			sync_chunk_writes: None,
			use_native_transport: None,
			view_distance: None,
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
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

#[derive(Deserialize, Debug, Clone)]
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

#[derive(Deserialize, Debug, Clone)]
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

#[derive(Deserialize, Debug, Clone)]
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
