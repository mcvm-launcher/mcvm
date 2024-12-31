#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// User configuration for all plugins, stored in the plugins.json file
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct PluginsConfig {
	/// The enabled plugins
	pub plugins: Vec<PluginConfigDeser>,
}

/// User configuration for a plugin
#[derive(Debug)]
pub struct PluginConfig {
	/// The ID of the plugin
	pub id: String,
	/// The custom config for the plugin
	pub custom_config: Option<serde_json::Value>,
}

/// Deserialized format for a plugin configuration
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum PluginConfigDeser {
	/// Simple configuration with just the plugin name
	Simple(String),
	/// Full configuration
	Full {
		/// The ID of the plugin
		id: String,
		/// The custom config for the plugin
		#[serde(default)]
		#[serde(rename = "config")]
		custom_config: Option<serde_json::Value>,
	},
}

impl PluginConfigDeser {
	/// Convert this deserialized plugin config to the final version
	pub fn to_config(&self) -> PluginConfig {
		let id = match self {
			Self::Simple(id) | Self::Full { id, .. } => id.clone(),
		};
		let custom_config = match self {
			Self::Simple(..) => None,
			Self::Full { custom_config, .. } => custom_config.clone(),
		};

		PluginConfig { id, custom_config }
	}
}
