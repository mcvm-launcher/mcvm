use std::collections::{HashMap, HashSet};

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// User configuration for all plugins, stored in the plugins.json file
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct PluginsConfig {
	/// The list of enabled plugins
	pub plugins: HashSet<String>,
	/// Configuration for enabled plugins
	pub config: HashMap<String, serde_json::Value>,
}

/// User configuration for a plugin
#[derive(Debug)]
pub struct PluginConfig {
	/// The ID of the plugin
	pub id: String,
	/// The custom config for the plugin
	pub custom_config: Option<serde_json::Value>,
}
