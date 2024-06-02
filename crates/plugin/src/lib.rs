#![warn(missing_docs)]

//! This library is used by both MCVM to load plugins, and as a framework for defining
//! Rust plugins for MCVM to use

use anyhow::{bail, Context};
use hooks::{Hook, HookHandle, OnLoad};
use mcvm_core::Paths;
use mcvm_shared::output::MCVMOutput;
use plugin::Plugin;

/// API for Rust-based plugins to use
#[cfg(feature = "api")]
pub mod api;
/// Plugin hooks and their definitions
pub mod hooks;
/// Serialized output format for plugins
pub mod output;
/// Plugins
pub mod plugin;

/// A manager for plugins that is used to call their hooks
#[derive(Debug)]
pub struct PluginManager {
	plugins: Vec<Plugin>,
	mcvm_version: Option<&'static str>,
}

impl Default for PluginManager {
	fn default() -> Self {
		Self::new()
	}
}

impl PluginManager {
	/// Construct a new PluginManager
	pub fn new() -> Self {
		Self {
			plugins: Vec::new(),
			mcvm_version: None,
		}
	}

	/// Set the MCVM version of the manager
	pub fn set_mcvm_version(&mut self, version: &'static str) {
		self.mcvm_version = Some(version);
	}

	/// Add a plugin to the manager
	pub fn add_plugin(
		&mut self,
		plugin: Plugin,
		paths: &Paths,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		// Call the on_load hook
		let result = plugin
			.call_hook(&OnLoad, &(), paths, self.mcvm_version, o)
			.context("Failed to call on_load hook of plugin")?;
		if let Some(result) = result {
			result.result(o)?;
		}

		self.plugins.push(plugin);

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
		let mut out = Vec::new();
		for plugin in &self.plugins {
			let result = plugin
				.call_hook(&hook, arg, paths, self.mcvm_version, o)
				.context("Plugin hook failed")?;
			out.extend(result);
		}

		Ok(out)
	}

	/// Call a plugin hook on the manager on a specific plugin
	pub fn call_hook_on_plugin<H: Hook>(
		&self,
		hook: H,
		plugin_id: &str,
		arg: &H::Arg,
		paths: &Paths,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Option<HookHandle<H>>> {
		for plugin in &self.plugins {
			if plugin.get_id() == plugin_id {
				let result = plugin
					.call_hook(&hook, arg, paths, self.mcvm_version, o)
					.context("Plugin hook failed")?;
				return Ok(result);
			}
		}

		bail!("No plugin found that matched the given ID")
	}

	/// Iterate over the plugins
	pub fn iter_plugins(&self) -> impl Iterator<Item = &Plugin> {
		self.plugins.iter()
	}
}
