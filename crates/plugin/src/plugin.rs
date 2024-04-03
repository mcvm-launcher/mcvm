use std::{collections::HashSet, ffi::OsString};

use serde::Deserialize;

use crate::hooks::Hook;

/// A plugin
pub struct Plugin {
	/// The plugin's configuration
	config: PluginConfiguration,
}

impl Plugin {
	/// Create a new plugin from configuration
	pub fn new(config: PluginConfiguration) -> Self {
		Self { config }
	}

	/// Call a hook on the plugin
	pub fn call_hook<H: Hook>(&self, hook: &H, arg: &H::Arg) -> anyhow::Result<Option<H::Result>> {
		if self.config.enabled_hooks.contains(hook.get_name()) {
			hook.call(&self.config.executable, arg).map(Some)
		} else {
			Ok(None)
		}
	}
}

/// Configuration for a plugin
#[derive(Deserialize)]
pub struct PluginConfiguration {
	/// The executable to use for the plugin
	pub executable: OsString,
	/// The enabled hooks for the plugin
	#[serde(default)]
	pub enabled_hooks: HashSet<String>,
}

impl PluginConfiguration {
	/// Create a new PluginConfiguration from the executable
	pub fn new(executable: OsString) -> Self {
		Self {
			executable,
			enabled_hooks: HashSet::new(),
		}
	}
}
