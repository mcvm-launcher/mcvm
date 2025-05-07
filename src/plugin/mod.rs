/// Online plugin installation from verified GitHub repos
pub mod install;

use std::path::{Path, PathBuf};
use std::sync::{Arc, MutexGuard};

use crate::config::plugin::{PluginConfig, PluginsConfig};
use crate::io::paths::Paths;
use anyhow::{anyhow, Context};
use mcvm_core::io::{json_from_file, json_to_file_pretty};
use mcvm_plugin::hook_call::HookHandle;
use mcvm_plugin::hooks::Hook;
use mcvm_plugin::plugin::{Plugin, PluginManifest};
use mcvm_plugin::CorePluginManager;
use mcvm_shared::output::MCVMOutput;
use mcvm_shared::output::{MessageContents, MessageLevel};
use mcvm_shared::translate;
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
	/// Open the configuration
	pub fn open_config(paths: &Paths) -> anyhow::Result<PluginsConfig> {
		let path = Self::get_config_path(paths);

		if path.exists() {
			json_from_file(path).context("Failed to load plugin config from file")
		} else {
			let out = PluginsConfig::default();
			json_to_file_pretty(path, &out)
				.context("Failed to write default plugin config to file")?;

			Ok(out)
		}
	}

	/// Load the PluginManager from the plugins.json file
	pub fn load(paths: &Paths, o: &mut impl MCVMOutput) -> anyhow::Result<Self> {
		let config = Self::open_config(paths).context("Failed to open plugins config")?;

		let mut out = Self::new();

		for plugin in config.plugins {
			let config = config.config.get(&plugin).cloned();
			let plugin = PluginConfig {
				id: plugin,
				custom_config: config,
			};

			out.load_plugin(plugin, paths, o)
				.context("Failed to load plugin")?;
		}

		out.check_dependencies(o);

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

		if let Some(plugin_mcvm_version) = &plugin.get_manifest().mcvm_version {
			if let (Some(mcvm_version), Some(plugin_mcvm_version)) = (
				version_compare::Version::from(crate::VERSION),
				version_compare::Version::from(&plugin_mcvm_version),
			) {
				if plugin_mcvm_version > mcvm_version {
					o.display(
						MessageContents::Warning(translate!(
							o,
							PluginForNewerVersion,
							"plugin" = plugin.get_id()
						)),
						MessageLevel::Important,
					);
				}
			}
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
			o.display(
				MessageContents::Error(translate!(o, PluginNotFound, "plugin" = &plugin.id)),
				MessageLevel::Important,
			);

			return Ok(());
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

	/// Removes a plugin's files
	pub fn remove_plugin(plugin: &str, paths: &Paths) -> anyhow::Result<()> {
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

	/// Uninstalls a plugin by removing its files and disabling it
	pub fn uninstall_plugin(plugin: &str, paths: &Paths) -> anyhow::Result<()> {
		Self::remove_plugin(plugin, paths).context("Failed to remove plugin")?;

		Self::disable_plugin(plugin, paths)
			.context("Failed to disable the plugin after uninstalling it")?;

		Ok(())
	}

	/// Enabled a plugin
	pub fn enable_plugin(plugin: &str, paths: &Paths) -> anyhow::Result<()> {
		let config_path = Self::get_config_path(paths);
		let mut config = Self::open_config(paths).context("Failed to open plugin configuration")?;
		config.plugins.insert(plugin.to_string());
		json_to_file_pretty(config_path, &config).context("Failed to write to config file")
	}

	/// Disables a plugin
	pub fn disable_plugin(plugin: &str, paths: &Paths) -> anyhow::Result<()> {
		let config_path = Self::get_config_path(paths);
		let mut config = Self::open_config(paths).context("Failed to open plugin configuration")?;
		config.plugins.remove(plugin);
		json_to_file_pretty(config_path, &config).context("Failed to write to config file")
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

	/// Checks plugins to make sure that their dependencies are installed, outputting a warning if any are not
	pub fn check_dependencies(&self, o: &mut impl MCVMOutput) {
		let inner = self.inner.lock().expect("Failed to lock mutex");
		let ids: Vec<_> = inner
			.manager
			.iter_plugins()
			.map(|x| x.get_id().clone())
			.collect();

		for plugin in inner.manager.iter_plugins() {
			for dependency in &plugin.get_manifest().dependencies {
				if !ids.contains(dependency) {
					o.display(
						MessageContents::Warning(translate!(
							o,
							PluginDependencyMissing,
							"dependency" = dependency,
							"plugin" = plugin.get_id()
						)),
						MessageLevel::Important,
					);
				}
			}
		}
	}

	/// Checks whether a plugin is present in the manager
	pub fn has_plugin(&self, plugin: &str) -> bool {
		let inner = self.inner.lock().expect("Failed to lock mutex");
		inner.manager.has_plugin(plugin)
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
