use std::path::Path;
use std::sync::{Arc, MutexGuard};

use crate::io::files::paths::Paths;
use anyhow::{anyhow, Context};
use mcvm_core::io::{json_from_file, json_to_file_pretty};
use mcvm_shared::output::MCVMOutput;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use mcvm_plugin::hooks::{Hook, HookHandle};
use mcvm_plugin::plugin::{Plugin, PluginManifest};
use mcvm_plugin::PluginManager as LoadedPluginManager;
use std::sync::Mutex;

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
#[derive(Debug, Clone)]
pub struct PluginManager {
	inner: Arc<Mutex<PluginManagerInner>>,
}

/// Inner for the PluginManager
#[derive(Debug)]
pub struct PluginManagerInner {
	/// The core PluginManager
	pub manager: LoadedPluginManager,
	/// Plugin configurations
	pub configs: Vec<PluginConfig>,
}

impl PluginManager {
	/// Load the PluginManager from the plugins.json file
	pub fn load(paths: &Paths, o: &mut impl MCVMOutput) -> anyhow::Result<Self> {
		let path = paths.project.config_dir().join("plugins.json");
		let config = if path.exists() {
			json_from_file(path).context("Failed to load plugin config from file")?
		} else {
			let out = PluginsConfig::default();
			json_to_file_pretty(path, &out)
				.context("Failed to write default plugin config to file")?;

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
			inner: Arc::new(Mutex::new(PluginManagerInner {
				manager: LoadedPluginManager::new(),
				configs: Vec::new(),
			})),
		}
	}

	/// Add a plugin to the manager
	pub fn add_plugin(
		&mut self,
		plugin: PluginConfig,
		manifest: PluginManifest,
		paths: &Paths,
		plugin_dir: Option<&Path>,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		let custom_config = plugin.custom_config.clone();
		let id = plugin.id.clone();
		let mut inner = self.inner.lock().map_err(|x| anyhow!("{x}"))?;
		inner.configs.push(plugin);
		let mut plugin = Plugin::new(id, manifest);
		if let Some(custom_config) = custom_config {
			plugin.set_custom_config(custom_config)?;
		}
		if let Some(plugin_dir) = plugin_dir {
			plugin.set_working_dir(plugin_dir.to_owned());
		}

		inner.manager.add_plugin(plugin, &paths.core, o)?;

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
		let (path, plugin_dir) = if path.exists() {
			(path, None)
		} else {
			let dir = paths.plugins.join(&plugin.id);
			(dir.join("plugin.json"), Some(dir))
		};
		let manifest = json_from_file(path).context("Failed to read plugin manifest from file")?;

		self.add_plugin(plugin, manifest, paths, plugin_dir.as_deref(), o)?;

		Ok(())
	}

	/// Call a plugin hook on the manager and collects the results into a Vec
	pub fn call_hook<H: Hook>(
		&self,
		hook: H,
		arg: &H::Arg,
		paths: &Paths,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Vec<HookHandle<H>>> {
		let inner = self.inner.lock().map_err(|x| anyhow!("{x}"))?;
		inner.manager.call_hook(hook, arg, &paths.core, o)
	}

	/// Get a lock for the inner mutex
	pub fn get_lock(&self) -> anyhow::Result<MutexGuard<PluginManagerInner>> {
		let inner = self.inner.lock().map_err(|x| anyhow!("{x}"))?;
		Ok(inner)
	}
}

impl Default for PluginManager {
	fn default() -> Self {
		Self::new()
	}
}
