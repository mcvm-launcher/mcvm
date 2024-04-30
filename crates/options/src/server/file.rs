use std::fs::File;
use std::io::BufWriter;
use std::{collections::HashMap, io::Write, path::Path};

use anyhow::Context;
use itertools::Itertools;

use crate::read::read_options_file;
use crate::{match_key, match_key_int};
use mcvm_shared::util::ToInt;
use mcvm_shared::versions::{VersionInfo, VersionPattern};

use super::ServerOptions;

const SEP: char = '=';

/// Write server.properties to a file
pub fn write_server_properties(
	options: HashMap<String, String>,
	path: &Path,
) -> anyhow::Result<()> {
	let options = merge_server_properties(path, options).context("Failed to merge properties")?;
	let file = File::create(path).context("Failed to open file")?;
	let mut file = BufWriter::new(file);
	for (key, value) in options.iter().sorted_by_key(|x| x.0) {
		write_key(key, value, &mut file)
			.with_context(|| format!("Failed to write line for option {key} with value {value}"))?;
	}

	Ok(())
}

/// Collect a hashmap from an existing server.properties file so we can compare with it
fn read_server_properties(path: &Path) -> anyhow::Result<HashMap<String, String>> {
	if path.exists() {
		let contents = std::fs::read_to_string(path).context("Failed to read options.txt")?;
		read_options_file(&contents, SEP)
	} else {
		Ok(HashMap::new())
	}
}

/// Merge keys with an existing file
fn merge_server_properties(
	path: &Path,
	keys: HashMap<String, String>,
) -> anyhow::Result<HashMap<String, String>> {
	let mut file_keys =
		read_server_properties(path).context("Failed to open options file for merging")?;
	file_keys.extend(keys);
	Ok(file_keys)
}

/// Escape any unescaped colons. These will not work in the server.properties file
fn escape_colons(string: &str) -> String {
	// Remove any user-escaped colons
	let out = string.replace("\\:", ":");
	out.replace(':', "\\:")
}

/// Write a server options key to a writer
fn write_key<W: Write>(key: &str, value: &str, writer: &mut W) -> anyhow::Result<()> {
	writeln!(writer, "{key}={}", escape_colons(value))?;

	Ok(())
}

/// Makes a list of datapacks
fn write_datapacks(datapacks: &[String]) -> String {
	datapacks.join(",")
}

/// The key of the world name property
const WORLD_NAME_KEY: &str = "level-name";

/// Write server options to a list of keys
#[rustfmt::skip]
pub fn create_keys(
	options: &ServerOptions,
	version_info: &VersionInfo,
) -> anyhow::Result<HashMap<String, String>> {
	let mut out = HashMap::new();

	let after_18w42a = VersionPattern::After("18w42a".into()).matches_info(version_info);

	match_key!(out, options.allow_flight, "allow-flight");
	match_key!(out, options.world.allow_nether, "allow-nether");
	match_key!(out, options.broadcast_console_to_ops, "broadcast-console-to-ops");
	match_key!(out, options.broadcast_rcon_to_ops, "broadcast-rcon-to-ops");
	match_key!(out, &options.difficulty, "difficulty", after_18w42a);
	match_key_int!(out, &options.difficulty, "difficulty", !after_18w42a);
	match_key!(out, options.allow_command_blocks, "enable-command-block");
	match_key!(out, options.jmx_monitoring, "enable-jmx-monitoring");
	match_key!(out, options.rcon.enable, "enable-rcon");
	match_key!(out, options.enable_status, "enable-status");
	match_key!(out, options.query.enable, "enable-query");
	match_key!(out, options.enforce_secure_profile, "enforce-secure-profile");
	match_key!(out, options.whitelist.enforce, "enforce-whitelist");
	match_key!( out, options.entity_broadcast_range, "entity-broadcast-range-percentage");
	match_key!(out, options.gamemode.force, "force-gamemode");
	match_key!( out, options.datapacks.function_permission_level, "function-permission-level");
	match_key!(out, &options.gamemode.default, "gamemode", after_18w42a);
	match_key_int!(out, &options.gamemode.default, "gamemode", !after_18w42a);
	match_key!(out, options.world.structures, "generate-structures");
	if let Some(value) = &options.world.generator_settings {
		out.insert(
			"generator-settings".into(),
			serde_json::to_string(&value)
				.context("Failed to convert generator settings to a string")?,
		);
	}
	match_key!(out, options.hardcore, "hardcore");
	match_key!(out, options.hide_online_players, "hide-online-players");
	if let Some(value) = &options.datapacks.initial_disabled {
		out.insert("initial-disabled-packs".into(), write_datapacks(value));
	}
	if let Some(value) = &options.datapacks.initial_enabled {
		out.insert("initial-enabled-packs".into(), write_datapacks(value));
	}
	match_key!(out, &options.world.name, WORLD_NAME_KEY);
	match_key!(out, &options.world.seed, "level-seed");
	match_key!(out, &options.world.r#type, "level-type");
	match_key!( out, options.max_chained_neighbor_updates, "max-chained-neighbor-updates");
	match_key!(out, options.max_players, "max-players");
	match_key!(out, options.max_tick_time, "max-tick-time");
	match_key!(out, options.world.max_build_height, "max-build-height");
	match_key!(out, options.world.max_size, "max-world-size");
	match_key!(out, &options.motd, "motd");
	match_key_int!( out, &options.network_compression_threshold, "network-compression-threshold");
	if let Some(value) = options.offline_mode {
		out.insert("online-mode".into(), (!value).to_string());
	}
	match_key!(out, options.op_permission_level, "op-permission-level");
	match_key!(out, options.player_idle_timeout, "player-idle-timeout");
	match_key!( out, options.prevent_proxy_connections, "prevent-proxy-connections");
	match_key!(out, options.enable_chat_preview, "previews-chat");
	match_key!(out, options.enable_pvp, "pvp");
	match_key!(out, options.query.port, "query.port");
	match_key!(out, options.rate_limit, "rate-limit");
	match_key!(out, &options.rcon.password, "rcon.password");
	match_key!(out, options.rcon.port, "rcon.port");
	match_key!(out, &options.resource_pack.uri, "resource-pack");
	match_key!(out, &options.resource_pack.prompt, "resource-pack-prompt");
	match_key!( out, &options.resource_pack.required, "require-resource-pack");
	match_key!(out, &options.ip, "server-ip");
	match_key!(out, options.port, "server-port");
	match_key!(out, options.simulation_distance, "simulation-distance");
	match_key!(out, options.enable_snooper, "snooper-enabled");
	match_key!(out, options.spawn_animals, "spawn-animals");
	match_key!(out, options.spawn_monsters, "spawn-monsters");
	match_key!(out, options.spawn_npcs, "spawn-npcs");
	match_key!(out, options.spawn_protection, "spawn-protection");
	match_key!(out, options.use_native_transport, "use-native-transport");
	match_key!(out, options.view_distance, "view-distance");
	match_key!(out, options.whitelist.enable, "white-list");

	let custom_clone = options.custom.clone();
	out.extend(custom_clone);

	Ok(out)
}

/// Gets the world name property from the rendered options
pub fn get_world_name(options: &HashMap<String, String>) -> Option<&String> {
	options.get(WORLD_NAME_KEY)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::read::parse_options_str;

	#[test]
	fn test_escape_colons() {
		assert_eq!(escape_colons("hello"), "hello");
		assert_eq!(escape_colons("minecraft:flat"), "minecraft\\:flat");
		assert_eq!(escape_colons("one\\:two:three"), "one\\:two\\:three");
	}

	#[test]
	fn test_create_keys() {
		let options = parse_options_str(r#"{"client": {}, "server": {}}"#).unwrap();
		let versions = vec!["1.18".to_string(), "1.19.3".to_string()];
		let info = VersionInfo {
			version: "1.19.3".to_string(),
			versions,
		};
		create_keys(&options.server.unwrap(), &info).unwrap();
	}
}
