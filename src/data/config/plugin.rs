use crate::io::files::paths::Paths;
use anyhow::Context;
use mcvm_core::io::json_from_file;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use mcvm_plugin::{
	hooks::Hook,
	plugin::{Plugin, PluginManifest},
	PluginManager as LoadedPluginManager,
};

/// User configuration for a plugin
#[derive(Debug)]
pub struct PluginConfig {
	/// The name of the plugin
	pub name: String,
}

/// Deserialized format for a plugin configuration
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub enum PluginConfigDeser {
	/// Simple configuration with just the plugin name
	Simple(String),
	/// Full configuration
	Full {
		/// The name of the plugin
		name: String,
	},
}

impl PluginConfigDeser {
	/// Convert this deserialized plugin config to the final version
	pub fn to_config(&self) -> PluginConfig {
		let name = match self {
			Self::Simple(name) | Self::Full { name } => name.clone(),
		};

		PluginConfig { name }
	}
}

/// Manager for plugin configs and the actual loaded plugin manager
#[derive(Debug)]
pub struct PluginManager {
	manager: LoadedPluginManager,
	configs: Vec<PluginConfig>,
}

impl PluginManager {
	/// Create a new PluginManager
	pub fn new() -> Self {
		Self {
			manager: LoadedPluginManager::new(),
			configs: Vec::new(),
		}
	}

	/// Add a plugin to the manager
	pub fn add_plugin(&mut self, plugin: PluginConfig, manifest: PluginManifest) {
		self.configs.push(plugin);
		let plugin = Plugin::new(manifest);
		self.manager.add_plugin(plugin);
	}

	/// Load a plugin from the plugin directory
	pub fn load_plugin(&mut self, plugin: PluginConfig, paths: &Paths) -> anyhow::Result<()> {
		// Get the path for the manifest
		let path = paths.plugins.join(format!("{}.json", plugin.name));
		let path = if path.exists() {
			path
		} else {
			paths.plugins.join(&plugin.name).join("plugin.json")
		};
		let manifest = json_from_file(path).context("Failed to read plugin manifest from file")?;

		self.add_plugin(plugin, manifest);

		Ok(())
	}

	/// Call a plugin hook on the manager and collects the results into a Vec
	pub fn call_hook<H: Hook>(&self, hook: H, arg: &H::Arg) -> anyhow::Result<Vec<H::Result>> {
		self.manager.call_hook(hook, arg)
	}
}

impl Default for PluginManager {
	fn default() -> Self {
		Self::new()
	}
}
