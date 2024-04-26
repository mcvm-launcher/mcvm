#![warn(missing_docs)]
#![deny(unsafe_code)]

//! This library is used by both MCVM to load plugins, and as a framework for definining
//! Rust plugins for MCVM to use

use anyhow::Context;
use hooks::{Hook, HookHandle, OnLoad};
use mcvm_core::Paths;
use mcvm_shared::output::MCVMOutput;
use plugin::Plugin;

/// API for Rust-based plugins to use to define plugins
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
		}
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
			.call_hook(&OnLoad, &(), paths, o)
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
				.call_hook(&hook, arg, paths, o)
				.context("Plugin hook failed")?;
			out.extend(result);
		}

		Ok(out)
	}

	/// Iterate over the plugins
	pub fn iter_plugins(&self) -> impl Iterator<Item = &Plugin> {
		self.plugins.iter()
	}
}
