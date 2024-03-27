use std::fs::File;
use std::io::BufWriter;
use std::{collections::HashMap, io::Write, path::Path};

use anyhow::Context;
use itertools::Itertools;

use crate::read::read_options_file;
use mcvm_shared::util::ToInt;
use mcvm_shared::versions::{VersionInfo, VersionPattern};

use super::ServerOptions;

const SEP: char = '=';

/// Write server.properties to a file
pub async fn write_server_properties(
	options: HashMap<String, String>,
	path: &Path,
) -> anyhow::Result<()> {
	let options = merge_server_properties(path, options)
		.await
		.context("Failed to merge properties")?;
	let file = File::create(path).context("Failed to open file")?;
	let mut file = BufWriter::new(file);
	for (key, value) in options.iter().sorted_by_key(|x| x.0) {
		write_key(key, value, &mut file)
			.with_context(|| format!("Failed to write line for option {key} with value {value}"))?;
	}

	Ok(())
}

/// Collect a hashmap from an existing server.properties file so we can compare with it
async fn read_server_properties(path: &Path) -> anyhow::Result<HashMap<String, String>> {
	if path.exists() {
		let contents = std::fs::read_to_string(path).context("Failed to read options.txt")?;
		read_options_file(&contents, SEP)
	} else {
		Ok(HashMap::new())
	}
}

/// Merge keys with an existing file
async fn merge_server_properties(
	path: &Path,
	keys: HashMap<String, String>,
) -> anyhow::Result<HashMap<String, String>> {
	let mut file_keys = read_server_properties(path)
		.await
		.context("Failed to open options file for merging")?;
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

/// Write server options to a list of keys
pub fn create_keys(
	options: &ServerOptions,
	version_info: &VersionInfo,
) -> anyhow::Result<HashMap<String, String>> {
	let mut out = HashMap::new();

	let after_18w42a = VersionPattern::After("18w42a".into()).matches_info(version_info);

	if let Some(value) = options.allow_flight {
		out.insert("allow-flight".into(), value.to_string());
	}
	if let Some(value) = options.world.allow_nether {
		out.insert("allow-nether".into(), value.to_string());
	}
	if let Some(value) = options.broadcast_console_to_ops {
		out.insert("broadcast-console-to-ops".into(), value.to_string());
	}
	if let Some(value) = options.broadcast_rcon_to_ops {
		out.insert("broadcast-rcon-to-ops".into(), value.to_string());
	}
	if let Some(value) = &options.difficulty {
		out.insert(
			"difficulty".into(),
			if after_18w42a {
				value.to_string()
			} else {
				value.to_int().to_string()
			},
		);
	}
	if let Some(value) = options.allow_command_blocks {
		out.insert("enable-command-block".into(), value.to_string());
	}
	if let Some(value) = options.jmx_monitoring {
		out.insert("enable-jmx-monitoring".into(), value.to_string());
	}
	if let Some(value) = options.rcon.enable {
		out.insert("enable-rcon".into(), value.to_string());
	}
	if let Some(value) = options.enable_status {
		out.insert("enable-status".into(), value.to_string());
	}
	if let Some(value) = options.query.enable {
		out.insert("enable-query".into(), value.to_string());
	}
	if let Some(value) = options.enforce_secure_profile {
		out.insert("enforce-secure-profile".into(), value.to_string());
	}
	if let Some(value) = options.whitelist.enforce {
		out.insert("enforce-whitelist".into(), value.to_string());
	}
	if let Some(value) = options.entity_broadcast_range {
		out.insert(
			"entity-broadcast-range-percentage".into(),
			value.to_string(),
		);
	}
	if let Some(value) = options.gamemode.force {
		out.insert("force-gamemode".into(), value.to_string());
	}
	if let Some(value) = options.datapacks.function_permission_level {
		out.insert("function-permission-level".into(), value.to_string());
	}
	if let Some(value) = &options.gamemode.default {
		out.insert(
			"gamemode".into(),
			if after_18w42a {
				value.to_string()
			} else {
				value.to_int().to_string()
			},
		);
	}
	if let Some(value) = options.world.structures {
		out.insert("generate-structures".into(), value.to_string());
	}
	if let Some(value) = &options.world.generator_settings {
		out.insert(
			"generator-settings".into(),
			serde_json::to_string(&value)
				.context("Failed to convert generator settings to a string")?,
		);
	}
	if let Some(value) = options.hardcore {
		out.insert("hardcore".into(), value.to_string());
	}
	if let Some(value) = options.hide_online_players {
		out.insert("hide-online-players".into(), value.to_string());
	}
	if let Some(value) = &options.datapacks.initial_disabled {
		out.insert("initial-disabled-packs".into(), write_datapacks(value));
	}
	if let Some(value) = &options.datapacks.initial_enabled {
		out.insert("initial-enabled-packs".into(), write_datapacks(value));
	}
	if let Some(value) = &options.world.name {
		out.insert("level-name".into(), value.clone());
	}
	if let Some(value) = &options.world.seed {
		out.insert("level-seed".into(), value.clone());
	}
	if let Some(value) = &options.world.r#type {
		out.insert("level-type".into(), value.to_string());
	}
	if let Some(value) = options.max_chained_neighbor_updates {
		out.insert("max-chained-neighbor-updates".into(), value.to_string());
	}
	if let Some(value) = options.max_players {
		out.insert("max-players".into(), value.to_string());
	}
	if let Some(value) = options.max_tick_time {
		out.insert("max-tick-time".into(), value.to_string());
	}
	if let Some(value) = options.world.max_build_height {
		out.insert("max-build-height".into(), value.to_string());
	}
	if let Some(value) = options.world.max_size {
		out.insert("max-world-size".into(), value.to_string());
	}
	if let Some(value) = &options.motd {
		out.insert("motd".into(), value.clone());
	}
	if let Some(value) = &options.network_compression_threshold {
		out.insert(
			"network-compression-threshold".into(),
			value.to_int().to_string(),
		);
	}
	if let Some(value) = options.offline_mode {
		out.insert("online-mode".into(), (!value).to_string());
	}
	if let Some(value) = options.op_permission_level {
		out.insert("op-permission-level".into(), value.to_string());
	}
	if let Some(value) = options.player_idle_timeout {
		out.insert("player-idle-timeout".into(), value.to_string());
	}
	if let Some(value) = options.prevent_proxy_connections {
		out.insert("prevent-proxy-connections".into(), value.to_string());
	}
	if let Some(value) = options.enable_chat_preview {
		out.insert("previews-chat".into(), value.to_string());
	}
	if let Some(value) = options.enable_pvp {
		out.insert("pvp".into(), value.to_string());
	}
	if let Some(value) = options.query.port {
		out.insert("query.port".into(), value.to_string());
	}
	if let Some(value) = options.rate_limit {
		out.insert("rate-limit".into(), value.to_string());
	}
	if let Some(value) = &options.rcon.password {
		out.insert("rcon.password".into(), value.clone());
	}
	if let Some(value) = options.rcon.port {
		out.insert("rcon.port".into(), value.to_string());
	}
	if let Some(value) = &options.resource_pack.uri {
		out.insert("resource-pack".into(), value.clone());
	}
	if let Some(value) = &options.resource_pack.prompt {
		out.insert("resource-pack-prompt".into(), value.clone());
	}
	if let Some(value) = options.resource_pack.required {
		out.insert("require-resource-pack".into(), value.to_string());
	}
	if let Some(value) = &options.ip {
		out.insert("server-ip".into(), value.clone());
	}
	if let Some(value) = options.port {
		out.insert("server-port".into(), value.to_string());
	}
	if let Some(value) = options.simulation_distance {
		out.insert("simulation-distance".into(), value.to_string());
	}
	if let Some(value) = options.enable_snooper {
		out.insert("snooper-enabled".into(), value.to_string());
	}
	if let Some(value) = options.spawn_animals {
		out.insert("spawn-animals".into(), value.to_string());
	}
	if let Some(value) = options.spawn_monsters {
		out.insert("spawn-monsters".into(), value.to_string());
	}
	if let Some(value) = options.spawn_npcs {
		out.insert("spawn-npcs".into(), value.to_string());
	}
	if let Some(value) = options.spawn_protection {
		out.insert("spawn-protection".into(), value.to_string());
	}
	if let Some(value) = options.use_native_transport {
		out.insert("use-native-transport".into(), value.to_string());
	}
	if let Some(value) = options.view_distance {
		out.insert("view-distance".into(), value.to_string());
	}
	if let Some(value) = options.whitelist.enable {
		out.insert("white-list".into(), value.to_string());
	}

	let custom_clone = options.custom.clone();
	out.extend(custom_clone);

	Ok(out)
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
