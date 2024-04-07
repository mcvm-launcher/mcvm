#![warn(missing_docs)]
#![deny(unsafe_code)]

//! This library is used by both MCVM to load plugins, and as a framework for definining
//! Rust plugins for MCVM to use

use anyhow::Context;
use hooks::{Hook, OnLoad};
use plugin::Plugin;

/// API for Rust-based plugins to use to define plugins
#[cfg(feature = "api")]
pub mod api;
/// Plugin hooks and their definitions
pub mod hooks;
/// Plugins
pub mod plugin;

/// A manager for plugins that is used to call their hooks
#[derive(Debug)]
pub struct PluginManager {
	plugins: Vec<Plugin>,
}

impl PluginManager {
	/// Construct a new PluginManager
	pub fn new() -> Self {
		Self {
			plugins: Vec::new(),
		}
	}

	/// Add a plugin to the manager
	pub fn add_plugin(&mut self, plugin: Plugin) -> anyhow::Result<()> {
		// Call the on_load hook
		plugin
			.call_hook(&OnLoad, &())
			.context("Failed to call on_load hook of plugin")?;

		self.plugins.push(plugin);

		Ok(())
	}

	/// Call a plugin hook on the manager and collects the results into a Vec
	pub fn call_hook<H: Hook>(&self, hook: H, arg: &H::Arg) -> anyhow::Result<Vec<H::Result>> {
		let mut out = Vec::with_capacity(self.plugins.len());
		for plugin in &self.plugins {
			let result = plugin.call_hook(&hook, arg).context("Plugin hook failed")?;
			out.extend(result);
		}

		Ok(out)
	}
}
