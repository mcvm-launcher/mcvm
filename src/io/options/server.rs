use std::{collections::HashMap, fmt::Display, io::Write};

use anyhow::Context;
use serde::Deserialize;

use crate::util::{json, versions::VersionPattern, ToInt};

use super::read::{EnumOrNumber, EnumOrString};

#[derive(Deserialize, Debug, Clone)]
pub struct RconOptions {
	#[serde(default = "default_rcon_enable")]
	pub enable: bool,
	#[serde(default = "default_rcon_port")]
	pub port: u16,
	#[serde(default = "default_rcon_password")]
	pub password: Option<String>,
}

impl Default for RconOptions {
	fn default() -> Self {
		Self {
			enable: default_rcon_enable(),
			port: default_rcon_port(),
			password: default_rcon_password(),
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
pub struct QueryOptions {
	#[serde(default = "default_query_enable")]
	pub enable: bool,
	#[serde(default = "default_query_port")]
	pub port: u16,
}

impl Default for QueryOptions {
	fn default() -> Self {
		Self {
			enable: default_query_enable(),
			port: default_query_port(),
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
pub struct WhitelistOptions {
	#[serde(default = "default_whitelist_enable")]
	pub enable: bool,
	#[serde(default = "default_whitelist_enforce")]
	pub enforce: bool,
}

impl Default for WhitelistOptions {
	fn default() -> Self {
		Self {
			enable: default_whitelist_enable(),
			enforce: default_whitelist_enforce(),
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
pub struct GamemodeOptions {
	#[serde(default = "default_gamemode_default")]
	pub default: EnumOrNumber<GameMode>,
	#[serde(default = "default_gamemode_force")]
	pub force: bool,
}

impl Default for GamemodeOptions {
	fn default() -> Self {
		Self {
			default: default_gamemode_default(),
			force: default_gamemode_force(),
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
pub struct DatapacksOptions {
	#[serde(default = "default_datapacks_function_permission_level")]
	pub function_permission_level: u8,
	#[serde(default = "default_datapacks_initial_enabled")]
	pub initial_enabled: Vec<String>,
	#[serde(default = "default_datapacks_initial_disabled")]
	pub initial_disabled: Vec<String>,
}

impl Default for DatapacksOptions {
	fn default() -> Self {
		Self {
			function_permission_level: default_datapacks_function_permission_level(),
			initial_enabled: default_datapacks_initial_enabled(),
			initial_disabled: default_datapacks_initial_disabled(),
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
pub struct WorldOptions {
	#[serde(default = "default_world_name")]
	pub name: String,
	#[serde(default = "default_world_seed")]
	pub seed: Option<String>,
	#[serde(default = "default_world_type")]
	pub r#type: EnumOrString<WorldType>,
	#[serde(default = "default_world_structures")]
	pub structures: bool,
	#[serde(default = "default_world_generator_settings")]
	pub generator_settings: json::JsonObject,
	#[serde(default = "default_world_max_size")]
	pub max_size: u32,
	#[serde(default = "default_world_max_build_height")]
	pub max_build_height: u32,
	#[serde(default = "default_world_allow_nether")]
	pub allow_nether: bool,
}

impl Default for WorldOptions {
	fn default() -> Self {
		Self {
			name: default_world_name(),
			seed: default_world_seed(),
			r#type: default_world_type(),
			structures: default_world_structures(),
			generator_settings: default_world_generator_settings(),
			max_size: default_world_max_size(),
			max_build_height: default_world_max_build_height(),
			allow_nether: default_world_allow_nether(),
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
pub struct ResourcePackOptions {
	#[serde(default = "default_resource_pack_uri")]
	pub uri: Option<String>,
	#[serde(default = "default_resource_pack_prompt")]
	pub prompt: Option<String>,
	#[serde(default = "default_resource_pack_sha1")]
	pub sha1: Option<String>,
	#[serde(default = "default_resource_pack_required")]
	pub required: bool,
}

impl Default for ResourcePackOptions {
	fn default() -> Self {
		Self {
			uri: default_resource_pack_uri(),
			prompt: default_resource_pack_prompt(),
			sha1: default_resource_pack_sha1(),
			required: default_resource_pack_required(),
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
pub struct ServerOptions {
	#[serde(default)]
	pub rcon: RconOptions,
	#[serde(default)]
	pub query: QueryOptions,
	#[serde(default)]
	pub whitelist: WhitelistOptions,
	#[serde(default)]
	pub gamemode: GamemodeOptions,
	#[serde(default)]
	pub datapacks: DatapacksOptions,
	#[serde(default)]
	pub world: WorldOptions,
	#[serde(default)]
	pub resource_pack: ResourcePackOptions,
	#[serde(default = "default_allow_flight")]
	pub allow_flight: bool,
	#[serde(default = "default_broadcast_console_to_ops")]
	pub broadcast_console_to_ops: bool,
	#[serde(default = "default_broadcast_rcon_to_ops")]
	pub broadcast_rcon_to_ops: bool,
	#[serde(default = "default_difficulty")]
	pub difficulty: EnumOrNumber<Difficulty>,
	#[serde(default = "default_allow_command_blocks")]
	pub allow_command_blocks: bool,
	#[serde(default = "default_jmx_monitoring")]
	pub jmx_monitoring: bool,
	#[serde(default = "default_enable_status")]
	pub enable_status: bool,
	#[serde(default = "default_enforce_secure_profile")]
	pub enforce_secure_profile: bool,
	#[serde(default = "default_entity_broadcast_range")]
	pub entity_broadcast_range: u16,
	#[serde(default = "default_hardcore")]
	pub hardcore: bool,
	#[serde(default = "default_hide_online_players")]
	pub hide_online_players: bool,
	#[serde(default = "default_max_chained_neighbor_updates")]
	pub max_chained_neighbor_updates: i32,
	#[serde(default = "default_max_players")]
	pub max_players: u32,
	#[serde(default = "default_max_tick_time")]
	pub max_tick_time: u64,
	#[serde(default = "default_motd")]
	pub motd: String,
	#[serde(default = "default_network_compression_threshold")]
	pub network_compression_threshold: EnumOrNumber<NetworkCompression>,
	#[serde(default = "default_offline_mode")]
	pub offline_mode: bool,
	#[serde(default = "default_op_permission_level")]
	pub op_permission_level: u8,
	#[serde(default = "default_player_idle_timeout")]
	pub player_idle_timeout: u32,
	#[serde(default = "default_prevent_proxy_connections")]
	pub prevent_proxy_connections: bool,
	#[serde(default = "default_enable_chat_preview")]
	pub enable_chat_preview: bool,
	#[serde(default = "default_enable_pvp")]
	pub enable_pvp: bool,
	#[serde(default = "default_rate_limit")]
	pub rate_limit: i16,
	#[serde(default = "default_ip")]
	pub ip: Option<String>,
	#[serde(default = "default_port")]
	pub port: u16,
	#[serde(default = "default_simulation_distance")]
	pub simulation_distance: u8,
	#[serde(default = "default_enable_snooper")]
	pub enable_snooper: bool,
	#[serde(default = "default_spawn_animals")]
	pub spawn_animals: bool,
	#[serde(default = "default_spawn_monsters")]
	pub spawn_monsters: bool,
	#[serde(default = "default_spawn_npcs")]
	pub spawn_npcs: bool,
	#[serde(default = "default_spawn_protection")]
	pub spawn_protection: u32,
	#[serde(default = "default_sync_chunk_writes")]
	pub sync_chunk_writes: bool,
	#[serde(default = "default_use_native_transport")]
	pub use_native_transport: bool,
	#[serde(default = "default_view_distance")]
	pub view_distance: u8,
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
			allow_flight: default_allow_flight(),
			broadcast_console_to_ops: default_broadcast_console_to_ops(),
			broadcast_rcon_to_ops: default_broadcast_rcon_to_ops(),
			difficulty: default_difficulty(),
			allow_command_blocks: default_allow_command_blocks(),
			jmx_monitoring: default_jmx_monitoring(),
			enable_status: default_enable_status(),
			enforce_secure_profile: default_enforce_secure_profile(),
			entity_broadcast_range: default_entity_broadcast_range(),
			hardcore: default_hardcore(),
			hide_online_players: default_hide_online_players(),
			max_chained_neighbor_updates: default_max_chained_neighbor_updates(),
			max_players: default_max_players(),
			max_tick_time: default_max_tick_time(),
			motd: default_motd(),
			network_compression_threshold: default_network_compression_threshold(),
			offline_mode: default_offline_mode(),
			op_permission_level: default_op_permission_level(),
			player_idle_timeout: default_player_idle_timeout(),
			prevent_proxy_connections: default_prevent_proxy_connections(),
			enable_chat_preview: default_enable_chat_preview(),
			enable_pvp: default_enable_pvp(),
			rate_limit: default_rate_limit(),
			ip: default_ip(),
			port: default_port(),
			simulation_distance: default_simulation_distance(),
			enable_snooper: default_enable_snooper(),
			spawn_animals: default_spawn_animals(),
			spawn_monsters: default_spawn_monsters(),
			spawn_npcs: default_spawn_npcs(),
			spawn_protection: default_spawn_protection(),
			sync_chunk_writes: default_sync_chunk_writes(),
			use_native_transport: default_use_native_transport(),
			view_distance: default_view_distance(),
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

fn default_allow_flight() -> bool {
	false
}
fn default_broadcast_console_to_ops() -> bool {
	true
}
fn default_broadcast_rcon_to_ops() -> bool {
	true
}
fn default_difficulty() -> EnumOrNumber<Difficulty> {
	EnumOrNumber::Enum(Difficulty::Easy)
}
fn default_allow_command_blocks() -> bool {
	false
}
fn default_jmx_monitoring() -> bool {
	false
}
fn default_rcon_enable() -> bool {
	false
}
fn default_rcon_port() -> u16 {
	25575
}
fn default_rcon_password() -> Option<String> {
	None
}
fn default_enable_status() -> bool {
	true
}
fn default_query_enable() -> bool {
	false
}
fn default_query_port() -> u16 {
	25565
}
fn default_enforce_secure_profile() -> bool {
	true
}
fn default_whitelist_enable() -> bool {
	false
}
fn default_whitelist_enforce() -> bool {
	false
}
fn default_entity_broadcast_range() -> u16 {
	50
}
fn default_gamemode_default() -> EnumOrNumber<GameMode> {
	EnumOrNumber::Enum(GameMode::Survival)
}
fn default_gamemode_force() -> bool {
	false
}
fn default_datapacks_function_permission_level() -> u8 {
	2
}
fn default_datapacks_initial_enabled() -> Vec<String> {
	vec![String::from("vanilla")]
}
fn default_datapacks_initial_disabled() -> Vec<String> {
	vec![]
}
fn default_hardcore() -> bool {
	false
}
fn default_hide_online_players() -> bool {
	false
}
fn default_world_name() -> String {
	String::from("world")
}
fn default_world_seed() -> Option<String> {
	None
}
fn default_world_type() -> EnumOrString<WorldType> {
	EnumOrString::Enum(WorldType::Normal)
}
fn default_world_structures() -> bool {
	true
}
fn default_world_generator_settings() -> json::JsonObject {
	json::empty_object()
}
fn default_world_max_size() -> u32 {
	29999984
}
fn default_world_max_build_height() -> u32 {
	256
}
fn default_world_allow_nether() -> bool {
	true
}
fn default_max_chained_neighbor_updates() -> i32 {
	1000000
}
fn default_max_players() -> u32 {
	20
}
fn default_max_tick_time() -> u64 {
	60000
}
fn default_motd() -> String {
	String::from("A Minecraft Server")
}
fn default_network_compression_threshold() -> EnumOrNumber<NetworkCompression> {
	EnumOrNumber::Num(256)
}
fn default_offline_mode() -> bool {
	false
}
fn default_op_permission_level() -> u8 {
	4
}
fn default_player_idle_timeout() -> u32 {
	0
}
fn default_prevent_proxy_connections() -> bool {
	false
}
fn default_enable_chat_preview() -> bool {
	false
}
fn default_enable_pvp() -> bool {
	false
}
fn default_rate_limit() -> i16 {
	0
}
fn default_resource_pack_uri() -> Option<String> {
	None
}
fn default_resource_pack_prompt() -> Option<String> {
	None
}
fn default_resource_pack_sha1() -> Option<String> {
	None
}
fn default_resource_pack_required() -> bool {
	false
}
fn default_ip() -> Option<String> {
	None
}
fn default_port() -> u16 {
	25565
}
fn default_simulation_distance() -> u8 {
	10
}
fn default_enable_snooper() -> bool {
	false
}
fn default_spawn_animals() -> bool {
	true
}
fn default_spawn_monsters() -> bool {
	true
}
fn default_spawn_npcs() -> bool {
	true
}
fn default_spawn_protection() -> u32 {
	16
}
fn default_sync_chunk_writes() -> bool {
	true
}
fn default_use_native_transport() -> bool {
	true
}
fn default_view_distance() -> u8 {
	10
}

/// Makes a list of datapacks
fn write_datapacks(datapacks: &[String]) -> String {
	datapacks.join(",")
}

/// Write server options to a list of keys
pub fn create_keys(
	options: &ServerOptions,
	version: &str,
	versions: &[String],
) -> anyhow::Result<HashMap<String, String>> {
	let mut out = HashMap::new();

	let after_18w42a =
		VersionPattern::After(String::from("18w42a")).matches_single(version, versions);

	out.insert(
		String::from("allow-flight"),
		options.allow_flight.to_string(),
	);
	out.insert(
		String::from("allow-nether"),
		options.world.allow_nether.to_string(),
	);
	out.insert(
		String::from("broadcast-console-to-ops"),
		options.broadcast_console_to_ops.to_string(),
	);
	out.insert(
		String::from("broadcast-rcon-to-ops"),
		options.broadcast_rcon_to_ops.to_string(),
	);
	out.insert(
		String::from("difficulty"),
		if after_18w42a {
			options.difficulty.to_string()
		} else {
			options.difficulty.to_int().to_string()
		},
	);
	out.insert(
		String::from("enable-command-block"),
		options.allow_command_blocks.to_string(),
	);
	out.insert(
		String::from("enable-jmx-monitoring"),
		options.jmx_monitoring.to_string(),
	);
	out.insert(String::from("enable-rcon"), options.rcon.enable.to_string());
	out.insert(
		String::from("enable-status"),
		options.enable_status.to_string(),
	);
	out.insert(
		String::from("enable-query"),
		options.query.enable.to_string(),
	);
	out.insert(
		String::from("enforce-secure-profile"),
		options.enforce_secure_profile.to_string(),
	);
	out.insert(
		String::from("enforce-whitelist"),
		options.whitelist.enforce.to_string(),
	);
	out.insert(
		String::from("entity-broadcast-range-percentage"),
		options.entity_broadcast_range.to_string(),
	);
	out.insert(
		String::from("force-gamemode"),
		options.gamemode.force.to_string(),
	);
	out.insert(
		String::from("function-permission-level"),
		options.datapacks.function_permission_level.to_string(),
	);
	out.insert(
		String::from("gamemode"),
		if after_18w42a {
			options.gamemode.default.to_string()
		} else {
			options.gamemode.default.to_int().to_string()
		},
	);
	out.insert(
		String::from("generate-structures"),
		options.world.structures.to_string(),
	);
	out.insert(
		String::from("generator-settings"),
		serde_json::to_string(&options.world.generator_settings)
			.context("Failed to convert generator settings to a string")?,
	);
	out.insert(String::from("hardcore"), options.hardcore.to_string());
	out.insert(
		String::from("hide-online-players"),
		options.hide_online_players.to_string(),
	);
	out.insert(
		String::from("initial-disabled-packs"),
		write_datapacks(&options.datapacks.initial_disabled),
	);
	out.insert(
		String::from("initial-enabled-packs"),
		write_datapacks(&options.datapacks.initial_enabled),
	);
	out.insert(String::from("level-name"), options.world.name.clone());
	out.insert(
		String::from("level-seed"),
		options.world.seed.clone().unwrap_or_default(),
	);
	out.insert(String::from("level-type"), options.world.r#type.to_string());
	out.insert(
		String::from("max-chained-neighbor-updates"),
		options.max_chained_neighbor_updates.to_string(),
	);
	out.insert(String::from("max-players"), options.max_players.to_string());
	out.insert(
		String::from("max-tick-time"),
		options.max_tick_time.to_string(),
	);
	out.insert(
		String::from("max-build-height"),
		options.world.max_build_height.to_string(),
	);
	out.insert(
		String::from("max-world-size"),
		options.world.max_size.to_string(),
	);
	out.insert(String::from("motd"), options.motd.clone());
	out.insert(
		String::from("network-compression-threshold"),
		options.network_compression_threshold.to_int().to_string(),
	);
	out.insert(
		String::from("online-mode"),
		(!options.offline_mode).to_string(),
	);
	out.insert(
		String::from("op-permission-level"),
		options.op_permission_level.to_string(),
	);
	out.insert(
		String::from("player-idle-timeout"),
		options.player_idle_timeout.to_string(),
	);
	out.insert(
		String::from("prevent-proxy-connections"),
		options.prevent_proxy_connections.to_string(),
	);
	out.insert(
		String::from("previews-chat"),
		options.enable_chat_preview.to_string(),
	);
	out.insert(String::from("pvp"), options.enable_pvp.to_string());
	out.insert(String::from("query.port"), options.query.port.to_string());
	out.insert(String::from("rate-limit"), options.rate_limit.to_string());
	out.insert(
		String::from("rcon.password"),
		options.rcon.password.clone().unwrap_or_default(),
	);
	out.insert(String::from("rcon.port"), options.rcon.port.to_string());
	out.insert(
		String::from("resource-pack"),
		options.resource_pack.uri.clone().unwrap_or_default(),
	);
	out.insert(
		String::from("resource-pack-prompt"),
		options.resource_pack.prompt.clone().unwrap_or_default(),
	);
	out.insert(
		String::from("require-resource-pack"),
		options.resource_pack.required.to_string(),
	);
	out.insert(
		String::from("server-ip"),
		options.ip.clone().unwrap_or_default(),
	);
	out.insert(String::from("server-port"), options.port.to_string());
	out.insert(
		String::from("simulation-distance"),
		options.simulation_distance.to_string(),
	);
	out.insert(
		String::from("snooper-enabled"),
		options.enable_snooper.to_string(),
	);
	out.insert(
		String::from("spawn-animals"),
		options.spawn_animals.to_string(),
	);
	out.insert(
		String::from("spawn-monsters"),
		options.spawn_monsters.to_string(),
	);
	out.insert(String::from("spawn-npcs"), options.spawn_npcs.to_string());
	out.insert(
		String::from("spawn-protection"),
		options.spawn_protection.to_string(),
	);
	out.insert(
		String::from("use-native-transport"),
		options.use_native_transport.to_string(),
	);
	out.insert(
		String::from("view-distance"),
		options.view_distance.to_string(),
	);
	out.insert(
		String::from("white-list"),
		options.whitelist.enable.to_string(),
	);

	Ok(out)
}

/// Escape any unescaped colons. These will not work in the server.properties file
fn escape_colons(string: &str) -> String {
	// Remove any user-escaped colons
	let out = string.replace("\\:", ":");
	out.replace(":", "\\:")
}

/// Write a server options key to a writer
pub fn write_key<W: Write>(key: &str, value: &str, writer: &mut W) -> anyhow::Result<()> {
	writeln!(writer, "{key}={}", escape_colons(value))?;

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::io::options::read::parse_options_str;

	#[test]
	fn test_escape_colons() {
		assert_eq!(escape_colons("hello"), "hello");
		assert_eq!(escape_colons("minecraft:flat"), "minecraft\\:flat");
		assert_eq!(escape_colons("one\\:two:three"), "one\\:two\\:three");
	}

	#[test]
	fn test_create_keys() {
		let options = parse_options_str(r#"{"client": {}, "server": {}}"#).unwrap();
		let versions = [String::from("1.18"), String::from("1.19.3")];
		create_keys(&options.server.unwrap(), "1.19.3", &versions).unwrap();
	}
}
