use crate::io::files::paths::Paths;
use anyhow::Context;
use mcvm_core::io::{json_from_file, json_to_file};
use mcvm_shared::output::MCVMOutput;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use mcvm_plugin::hooks::Hook;
use mcvm_plugin::plugin::{Plugin, PluginManifest};
use mcvm_plugin::PluginManager as LoadedPluginManager;

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

/// Manager for plugin configs and the actual loaded plugin manager
#[derive(Debug)]
pub struct PluginManager {
	manager: LoadedPluginManager,
	configs: Vec<PluginConfig>,
}

impl PluginManager {
	/// Load the PluginManager from the plugins.json file
	pub fn load(paths: &Paths, o: &mut impl MCVMOutput) -> anyhow::Result<Self> {
		let path = paths.project.config_dir().join("plugins.json");
		let config = if path.exists() {
			json_from_file(path).context("Failed to load plugin config from file")?
		} else {
			let out = PluginsConfig::default();
			json_to_file(path, &out).context("Failed to write default plugin config to file")?;

			out
		};

		let mut out = Self::new();

		for plugin in config.plugins {
			let plugin = plugin.to_config();
			out.load_plugin(plugin, paths, o)
				.context("Failed to load plugin")?;
		}

		Ok(out)
	}

	/// Create a new PluginManager with no plugins
	pub fn new() -> Self {
		Self {
			manager: LoadedPluginManager::new(),
			configs: Vec::new(),
		}
	}

	/// Add a plugin to the manager
	pub fn add_plugin(
		&mut self,
		plugin: PluginConfig,
		manifest: PluginManifest,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		let custom_config = plugin.custom_config.clone();
		let id = plugin.id.clone();
		self.configs.push(plugin);
		let mut plugin = Plugin::new(id, manifest);
		if let Some(custom_config) = custom_config {
			plugin.set_custom_config(custom_config)?;
		}

		self.manager.add_plugin(plugin, o)?;

		Ok(())
	}

	/// Load a plugin from the plugin directory
	pub fn load_plugin(
		&mut self,
		plugin: PluginConfig,
		paths: &Paths,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		// Get the path for the manifest
		let path = paths.plugins.join(format!("{}.json", plugin.id));
		let path = if path.exists() {
			path
		} else {
			paths.plugins.join(&plugin.id).join("plugin.json")
		};
		let manifest = json_from_file(path).context("Failed to read plugin manifest from file")?;

		self.add_plugin(plugin, manifest, o)?;

		Ok(())
	}

	/// Call a plugin hook on the manager and collects the results into a Vec
	pub fn call_hook<H: Hook>(
		&self,
		hook: H,
		arg: &H::Arg,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Vec<H::Result>> {
		self.manager.call_hook(hook, arg, o)
	}

	/// Iterate over the stored plugins
	pub fn iter_plugins(&self) -> impl Iterator<Item = &Plugin> {
		self.manager.iter_plugins()
	}
}

impl Default for PluginManager {
	fn default() -> Self {
		Self::new()
	}
}
