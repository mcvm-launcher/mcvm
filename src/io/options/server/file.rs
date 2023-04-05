use std::fs::File;
use std::{path::Path, collections::HashMap, io::Write};

use anyhow::Context;
use itertools::Itertools;

use crate::io::options::read::read_options_file;
use crate::util::{versions::VersionPattern, ToInt};

use super::ServerOptions;

static SEP: char = '=';

/// Write server.properties to a file
pub async fn write_server_properties(
	options: HashMap<String, String>,
	path: &Path,
) -> anyhow::Result<()> {
	let options = merge_server_properties(path, options).await
		.context("Failed to merge properties")?;
	let mut file = File::create(path).context("Failed to open file")?;
	for (key, value) in options.iter().sorted_by_key(|x| x.0) {
		write_key(key, value, &mut file)
			.with_context(|| format!("Failed to write line for option {key} with value {value}"))?;
	}

	Ok(())
}

/// Collect a hashmap from an existing server.properties file so we can compare with it
async fn read_server_properties(path: &Path) -> anyhow::Result<HashMap<String, String>> {
	let contents = tokio::fs::read_to_string(path).await
		.context("Failed to read options.txt")?;
	read_options_file(&contents, SEP).await
}

/// Merge keys with an existing file
async fn merge_server_properties(
	path: &Path,
	keys: HashMap<String, String>
) -> anyhow::Result<HashMap<String, String>> {
	let mut file_keys = read_server_properties(path).await
		.context("Failed to open options file for merging")?;
	file_keys.extend(keys);
	Ok(file_keys)
}

/// Escape any unescaped colons. These will not work in the server.properties file
fn escape_colons(string: &str) -> String {
	// Remove any user-escaped colons
	let out = string.replace("\\:", ":");
	out.replace(":", "\\:")
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
