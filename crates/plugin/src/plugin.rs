use std::collections::HashSet;

use anyhow::Context;
use mcvm_shared::{lang::translate::TranslationMap, output::MCVMOutput};
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
		if self.manifest.enabled_hooks.contains(hook.get_name()) {
			hook.call(
				&self.manifest.executable,
				arg,
				self.custom_config.clone(),
				o,
			)
			.map(Some)
		} else {
			Ok(None)
		}
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
	pub executable: String,
	/// The enabled hooks for the plugin
	#[serde(default)]
	pub enabled_hooks: HashSet<String>,
	/// The translation map the plugin provides
	#[serde(default)]
	pub translation_map: TranslationMap,
}

impl PluginManifest {
	/// Create a new PluginManifest from the executable
	pub fn new(executable: String) -> Self {
		Self {
			executable,
			enabled_hooks: HashSet::new(),
			translation_map: TranslationMap::new(),
		}
	}
}
