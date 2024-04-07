use std::{collections::HashSet, ffi::OsString};

use serde::Deserialize;

use crate::hooks::Hook;

/// A plugin
#[derive(Debug)]
pub struct Plugin {
	/// The plugin's manifest
	manifest: PluginManifest,
}

impl Plugin {
	/// Create a new plugin from a manifest
	pub fn new(manifest: PluginManifest) -> Self {
		Self { manifest }
	}

	/// Call a hook on the plugin
	pub fn call_hook<H: Hook>(&self, hook: &H, arg: &H::Arg) -> anyhow::Result<Option<H::Result>> {
		if self.manifest.enabled_hooks.contains(hook.get_name()) {
			hook.call(&self.manifest.executable, arg).map(Some)
		} else {
			Ok(None)
		}
	}
}

/// Configuration for a plugin
#[derive(Deserialize, Debug)]
pub struct PluginManifest {
	/// The executable to use for the plugin
	pub executable: OsString,
	/// The enabled hooks for the plugin
	#[serde(default)]
	pub enabled_hooks: HashSet<String>,
}

impl PluginManifest {
	/// Create a new PluginManifest from the executable
	pub fn new(executable: OsString) -> Self {
		Self {
			executable,
			enabled_hooks: HashSet::new(),
		}
	}
}
