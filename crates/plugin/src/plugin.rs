use std::collections::HashSet;

use anyhow::Context;
use mcvm_shared::{lang::translate::LanguageMap, output::MCVMOutput};
use serde::Deserialize;

use crate::hooks::Hook;

/// A plugin
#[derive(Debug)]
pub struct Plugin {
	/// The plugin's manifest
	manifest: PluginManifest,
	/// The custom config for the plugin, serialized from JSON
	custom_config: Option<String>,
}

impl Plugin {
	/// Create a new plugin from a manifest
	pub fn new(manifest: PluginManifest) -> Self {
		Self {
			manifest,
			custom_config: None,
		}
	}

	/// Get the manifest of the plugin
	pub fn get_manifest(&self) -> &PluginManifest {
		&self.manifest
	}

	/// Call a hook on the plugin
	pub fn call_hook<H: Hook>(
		&self,
		hook: &H,
		arg: &H::Arg,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Option<H::Result>> {
		let Some(executable) = self.manifest.executable.as_ref() else {
			return Ok(None);
		};
		if !self.manifest.enabled_hooks.contains(hook.get_name()) {
			return Ok(None);
		}
		hook.call(executable, arg, self.custom_config.clone(), o)
			.map(Some)
	}

	/// Set the custom config of the plugin
	pub fn set_custom_config(&mut self, config: serde_json::Value) -> anyhow::Result<()> {
		let serialized =
			serde_json::to_string(&config).context("Failed to serialize custom plugin config")?;
		self.custom_config = Some(serialized);
		Ok(())
	}
}

/// Configuration for a plugin
#[derive(Deserialize, Debug)]
pub struct PluginManifest {
	/// The executable to use for the plugin
	#[serde(default)]
	pub executable: Option<String>,
	/// The enabled hooks for the plugin
	#[serde(default)]
	pub enabled_hooks: HashSet<String>,
	/// The lanugage map the plugin provides
	#[serde(default)]
	pub language_map: LanguageMap,
}

impl PluginManifest {
	/// Create a new PluginManifest with no executable
	pub fn new() -> Self {
		Self {
			executable: None,
			enabled_hooks: HashSet::new(),
			language_map: LanguageMap::new(),
		}
	}

	/// Create a new PluginManifest with an executable
	pub fn with_executable(executable: String) -> Self {
		Self {
			executable: Some(executable),
			enabled_hooks: HashSet::new(),
			language_map: LanguageMap::new(),
		}
	}
}
