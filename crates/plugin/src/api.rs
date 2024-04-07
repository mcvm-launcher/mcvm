use std::env::Args;

use anyhow::Context;

use crate::hooks::{Hook, OnLoad};

/// A plugin definition
pub struct CustomPlugin {
	name: String,
	_args: Args,
	hook: String,
	ctx: HookContext,
}

impl CustomPlugin {
	/// Create a new plugin definition
	pub fn new(name: &str) -> anyhow::Result<Self> {
		let mut args = std::env::args();
		let hook = args.nth(1).context("Missing hook to run")?;
		let custom_config = std::env::var("MCVM_CUSTOM_CONFIG").ok();
		let ctx = HookContext { custom_config };
		Ok(Self {
			name: name.into(),
			_args: args,
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
		&self,
		f: impl FnOnce(&HookContext, ()) -> anyhow::Result<()>,
	) -> anyhow::Result<()> {
		self.handle_hook::<OnLoad>(|_| Ok(()), f)
	}

	/// Handle a hook
	fn handle_hook<H: Hook>(
		&self,
		arg: impl FnOnce(&Self) -> anyhow::Result<H::Arg>,
		f: impl FnOnce(&HookContext, H::Arg) -> anyhow::Result<H::Result>,
	) -> anyhow::Result<()> {
		if self.hook == H::get_name_static() {
			let arg = arg(&self)?;
			let result = f(&self.ctx, arg)?;
			let serialized = serde_json::to_string(&result)?;
			println!("{serialized}");
			Ok(())
		} else {
			Ok(())
		}
	}
}

/// Argument passed to every hook
pub struct HookContext {
	custom_config: Option<String>,
}

impl HookContext {
	/// Get the custom configuration for the plugin passed into the hook
	pub fn get_custom_config(&self) -> Option<&str> {
		self.custom_config.as_deref()
	}
}
