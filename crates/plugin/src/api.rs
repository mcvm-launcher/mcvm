/// Output back to the main MCVM process
pub mod output;

use std::env::Args;

use anyhow::Context;
use serde::de::DeserializeOwned;

use crate::hooks::{Hook, OnLoad, Subcommand};
use crate::output::OutputAction;

use self::output::PluginOutput;
pub use mcvm_shared::output::*;

/// A plugin definition
pub struct CustomPlugin {
	name: String,
	args: Args,
	hook: String,
	ctx: HookContext,
}

impl CustomPlugin {
	/// Create a new plugin definition
	pub fn new(name: &str) -> anyhow::Result<Self> {
		let mut args = std::env::args();
		args.nth(0);
		let hook = args.nth(0).context("Missing hook to run")?;
		let custom_config = std::env::var("MCVM_CUSTOM_CONFIG").ok();
		let ctx = HookContext {
			custom_config,
			output: PluginOutput::new(),
		};
		Ok(Self {
			name: name.into(),
			args,
			hook,
			ctx,
		})
	}

	/// Get the name of the plugin
	pub fn get_name(&self) -> &str {
		&self.name
	}

	/// Bind to the on_load hook
	pub fn on_load(
		&mut self,
		f: impl FnOnce(&mut HookContext, ()) -> anyhow::Result<()>,
	) -> anyhow::Result<()> {
		self.handle_hook::<OnLoad>(|_| Ok(()), f)
	}

	/// Bind to the subcommand hook
	pub fn subcommand(
		&mut self,
		f: impl FnOnce(&mut HookContext, Vec<String>) -> anyhow::Result<()>,
	) -> anyhow::Result<()> {
		self.handle_hook::<Subcommand>(Self::get_hook_arg, f)
	}

	/// Handle a hook
	fn handle_hook<H: Hook>(
		&mut self,
		arg: impl FnOnce(&mut Self) -> anyhow::Result<H::Arg>,
		f: impl FnOnce(&mut HookContext, H::Arg) -> anyhow::Result<H::Result>,
	) -> anyhow::Result<()> {
		if self.hook == H::get_name_static() {
			let arg = arg(self)?;
			let result = f(&mut self.ctx, arg)?;
			if !H::get_takes_over() {
				let serialized = serde_json::to_string(&result)?;
				let action = OutputAction::SetResult(serialized);
				println!(
					"{}",
					action
						.serialize()
						.context("Failed to serialize hook result")?
				);
			}
			Ok(())
		} else {
			Ok(())
		}
	}

	/// Get the first argument as the hook input
	fn get_hook_arg<Arg: DeserializeOwned>(&mut self) -> anyhow::Result<Arg> {
		let arg = self.args.nth(0).context("Hook argument missing")?;
		serde_json::from_str(&arg).context("Failed to deserialize arg")
	}
}

/// Argument passed to every hook
pub struct HookContext {
	custom_config: Option<String>,
	output: PluginOutput,
}

impl HookContext {
	/// Get the custom configuration for the plugin passed into the hook
	pub fn get_custom_config(&self) -> Option<&str> {
		self.custom_config.as_deref()
	}

	/// Get the plugin's output stream
	pub fn get_output(&mut self) -> &mut PluginOutput {
		&mut self.output
	}
}
