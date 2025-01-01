/// Online plugin installation from verified GitHub repos
pub mod install;

use std::path::{Path, PathBuf};
use std::sync::{Arc, MutexGuard};

use crate::config::plugin::{PluginConfig, PluginsConfig};
use crate::io::paths::Paths;
use anyhow::{anyhow, bail, Context};
use mcvm_core::io::{json_from_file, json_to_file_pretty};
use mcvm_plugin::hooks::{Hook, HookHandle};
use mcvm_plugin::plugin::{Plugin, PluginManifest};
use mcvm_plugin::CorePluginManager;
use mcvm_shared::output::MCVMOutput;
use std::sync::Mutex;

/// Manager for plugin configs and the actual loaded plugin manager
#[derive(Debug, Clone)]
pub struct PluginManager {
	inner: Arc<Mutex<PluginManagerInner>>,
}

/// Inner for the PluginManager
#[derive(Debug)]
pub struct PluginManagerInner {
	/// The core PluginManager
	pub manager: CorePluginManager,
	/// Plugin configurations
	pub configs: Vec<PluginConfig>,
}

impl PluginManager {
	/// Load the PluginManager from the plugins.json file
	pub fn load(paths: &Paths, o: &mut impl MCVMOutput) -> anyhow::Result<Self> {
		let path = Self::get_config_path(paths);
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

			if config.disabled.contains(&plugin.id) {
				continue;
			}

			out.load_plugin(plugin, paths, o)
				.context("Failed to load plugin")?;
		}

		Ok(out)
	}

	/// Get the path to the config file
	pub fn get_config_path(paths: &Paths) -> PathBuf {
		paths.project.config_dir().join("plugins.json")
	}

	/// Write the default config file if it does not exist
	pub fn create_default(paths: &Paths) -> anyhow::Result<()> {
		let path = Self::get_config_path(paths);
		if !path.exists() {
			let out = PluginsConfig::default();
			json_to_file_pretty(path, &out)
				.context("Failed to write default plugin config to file")?;
		}

		Ok(())
	}

	/// Create a new PluginManager with no plugins
	pub fn new() -> Self {
		let mut manager = CorePluginManager::new();
		manager.set_mcvm_version(crate::VERSION);
		Self {
			inner: Arc::new(Mutex::new(PluginManagerInner {
				manager,
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
		if !path.exists() {
			bail!("Could not find plugin '{}'", &plugin.id);
		}
		let manifest = json_from_file(path).context("Failed to read plugin manifest from file")?;

		self.add_plugin(plugin, manifest, paths, plugin_dir.as_deref(), o)?;

		Ok(())
	}

	/// Gets all the available plugins from the plugin directory.
	/// Returns a list of tuples of the plugin ID and file path
	pub fn get_available_plugins(paths: &Paths) -> anyhow::Result<Vec<(String, PathBuf)>> {
		let reader = paths
			.plugins
			.read_dir()
			.context("Failed to read plugin directory")?;

		let mut out = Vec::with_capacity(reader.size_hint().0);
		for entry in reader {
			let Ok(entry) = entry else {
				continue;
			};

			let Ok(file_type) = entry.file_type() else {
				continue;
			};

			if file_type.is_dir() {
				let config_path = entry.path().join("plugin.json");
				if config_path.exists() {
					out.push((entry.file_name().to_string_lossy().to_string(), config_path));
				}
			} else {
				let file_name = entry.file_name().to_string_lossy().to_string();
				if file_name.ends_with(".json") {
					out.push((file_name.replace(".json", ""), entry.path()));
				}
			}
		}

		Ok(out)
	}

	/// Uninstalls a plugin by removing its files
	pub fn uninstall_plugin(plugin: &str, paths: &Paths) -> anyhow::Result<()> {
		let json_path = paths.plugins.join(format!("{plugin}.json"));
		if json_path.exists() {
			std::fs::remove_file(json_path).context("Failed to remove plugin JSON")?;
		}

		let dir_path = paths.plugins.join(plugin);
		if dir_path.exists() {
			std::fs::remove_dir_all(dir_path).context("Failed to remove plugin directory")?;
		}

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

	/// Call a plugin hook on a specific plugin
	pub fn call_hook_on_plugin<H: Hook>(
		&self,
		hook: H,
		plugin_id: &str,
		arg: &H::Arg,
		paths: &Paths,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Option<HookHandle<H>>> {
		let inner = self.inner.lock().map_err(|x| anyhow!("{x}"))?;
		inner
			.manager
			.call_hook_on_plugin(hook, plugin_id, arg, &paths.core, o)
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
