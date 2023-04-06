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

	if let Some(value) = options.allow_flight {
		out.insert(
			String::from("allow-flight"),
			value.to_string(),
		);
	}
	if let Some(value) = options.world.allow_nether {
		out.insert(
			String::from("allow-nether"),
			value.to_string(),
		);
	}
	if let Some(value) = options.broadcast_console_to_ops {
		out.insert(
			String::from("broadcast-console-to-ops"),
			value.to_string(),
		);
	}
	if let Some(value) = options.broadcast_rcon_to_ops {
		out.insert(
			String::from("broadcast-rcon-to-ops"),
			value.to_string(),
		);
	}
	if let Some(value) = &options.difficulty {
		out.insert(
			String::from("difficulty"),
			if after_18w42a {
				value.to_string()
			} else {
				value.to_int().to_string()
			},
		);
	}
	if let Some(value) = options.allow_command_blocks {
		out.insert(
			String::from("enable-command-block"),
			value.to_string(),
		);
	}
	if let Some(value) = options.jmx_monitoring {
		out.insert(
			String::from("enable-jmx-monitoring"),
			value.to_string(),
		);
	}
	if let Some(value) = options.rcon.enable {
		out.insert(String::from("enable-rcon"), value.to_string());
	}
	if let Some(value) = options.enable_status {
		out.insert(
			String::from("enable-status"),
			value.to_string(),
		);
	}
	if let Some(value) = options.query.enable {
		out.insert(
			String::from("enable-query"),
			value.to_string(),
		);
	}
	if let Some(value) = options.enforce_secure_profile {
		out.insert(
			String::from("enforce-secure-profile"),
			value.to_string(),
		);
	}
	if let Some(value) = options.whitelist.enforce {
		out.insert(
			String::from("enforce-whitelist"),
			value.to_string(),
		);
	}
	if let Some(value) = options.entity_broadcast_range {
		out.insert(
			String::from("entity-broadcast-range-percentage"),
			value.to_string(),
		);
	}
	if let Some(value) = options.gamemode.force {
		out.insert(
			String::from("force-gamemode"),
			value.to_string(),
		);
	}
	if let Some(value) = options.datapacks.function_permission_level {
		out.insert(
			String::from("function-permission-level"),
			value.to_string(),
		);
	}
	if let Some(value) = &options.gamemode.default {
		out.insert(
			String::from("gamemode"),
			if after_18w42a {
				value.to_string()
			} else {
				value.to_int().to_string()
			},
		);
	}
	if let Some(value) = options.world.structures {
		out.insert(
			String::from("generate-structures"),
			value.to_string(),
		);
	}
	if let Some(value) = &options.world.generator_settings {
		out.insert(
			String::from("generator-settings"),
			serde_json::to_string(&value)
				.context("Failed to convert generator settings to a string")?,
		);
	}
	if let Some(value) = options.hardcore {
		out.insert(String::from("hardcore"), value.to_string());
	}
	if let Some(value) = options.hide_online_players {
		out.insert(
			String::from("hide-online-players"),
			value.to_string(),
		);
	}
	if let Some(value) = &options.datapacks.initial_disabled {
		out.insert(
			String::from("initial-disabled-packs"),
			write_datapacks(&value),
		);
	}
	if let Some(value) = &options.datapacks.initial_enabled {
		out.insert(
			String::from("initial-enabled-packs"),
			write_datapacks(&value),
		);
	}
	if let Some(value) = &options.world.name {
		out.insert(String::from("level-name"), value.clone());
	}
	if let Some(value) = &options.world.seed {
		out.insert(
			String::from("level-seed"),
			value.clone(),
		);
	}
	if let Some(value) = &options.world.r#type {
		out.insert(String::from("level-type"), value.to_string());
	}
	if let Some(value) = options.max_chained_neighbor_updates {
		out.insert(
			String::from("max-chained-neighbor-updates"),
			value.to_string(),
		);
	}
	if let Some(value) = options.max_players {
		out.insert(String::from("max-players"), value.to_string());
	}
	if let Some(value) = options.max_tick_time {
		out.insert(
			String::from("max-tick-time"),
			value.to_string(),
		);
	}
	if let Some(value) = options.world.max_build_height {
		out.insert(
			String::from("max-build-height"),
			value.to_string(),
		);
	}
	if let Some(value) = options.world.max_size {
		out.insert(
			String::from("max-world-size"),
			value.to_string(),
		);
	}
	if let Some(value) = &options.motd {
		out.insert(String::from("motd"), value.clone());
	}
	if let Some(value) = &options.network_compression_threshold {
		out.insert(
			String::from("network-compression-threshold"),
			value.to_int().to_string(),
		);
	}
	if let Some(value) = options.offline_mode {
		out.insert(
			String::from("online-mode"),
			(!value).to_string(),
		);
	}
	if let Some(value) = options.op_permission_level {
		out.insert(
			String::from("op-permission-level"),
			value.to_string(),
		);
	}
	if let Some(value) = options.player_idle_timeout {
		out.insert(
			String::from("player-idle-timeout"),
			value.to_string(),
		);
	}
	if let Some(value) = options.prevent_proxy_connections {
		out.insert(
			String::from("prevent-proxy-connections"),
			value.to_string(),
		);
	}
	if let Some(value) = options.enable_chat_preview {
		out.insert(
			String::from("previews-chat"),
			value.to_string(),
		);
	}
	if let Some(value) = options.enable_pvp {
		out.insert(String::from("pvp"), value.to_string());
	}
	if let Some(value) = options.query.port {
		out.insert(String::from("query.port"), value.to_string());
	}
	if let Some(value) = options.rate_limit {
		out.insert(String::from("rate-limit"), value.to_string());
	}
	if let Some(value) = &options.rcon.password {
		out.insert(
			String::from("rcon.password"),
			value.clone(),
		);
	}
	if let Some(value) = options.rcon.port {
		out.insert(String::from("rcon.port"), value.to_string());
	}
	if let Some(value) = &options.resource_pack.uri {
		out.insert(
			String::from("resource-pack"),
			value.clone(),
		);
	}
	if let Some(value) = &options.resource_pack.prompt {
		out.insert(
			String::from("resource-pack-prompt"),
			value.clone(),
		);
	}
	if let Some(value) = options.resource_pack.required {
		out.insert(
			String::from("require-resource-pack"),
			value.to_string(),
		);
	}
	if let Some(value) = &options.ip {
		out.insert(
			String::from("server-ip"),
			value.clone(),
		);
	}
	if let Some(value) = options.port {
		out.insert(String::from("server-port"), value.to_string());
	}
	if let Some(value) = options.simulation_distance {
		out.insert(
			String::from("simulation-distance"),
			value.to_string(),
		);
	}
	if let Some(value) = options.enable_snooper {
		out.insert(
			String::from("snooper-enabled"),
			value.to_string(),
		);
	}
	if let Some(value) = options.spawn_animals {
		out.insert(
			String::from("spawn-animals"),
			value.to_string(),
		);
	}
	if let Some(value) = options.spawn_monsters {
		out.insert(
			String::from("spawn-monsters"),
			value.to_string(),
		);
	}
	if let Some(value) = options.spawn_npcs {
		out.insert(String::from("spawn-npcs"), value.to_string());
	}
	if let Some(value) = options.spawn_protection {
		out.insert(
			String::from("spawn-protection"),
			value.to_string(),
		);
	}
	if let Some(value) = options.use_native_transport {
		out.insert(
			String::from("use-native-transport"),
			value.to_string(),
		);
	}
	if let Some(value) = options.view_distance {
		out.insert(
			String::from("view-distance"),
			value.to_string(),
		);
	}
	if let Some(value) = options.whitelist.enable {
		out.insert(
			String::from("white-list"),
			value.to_string(),
		);
	}

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
