/// Output back to the main MCVM process
pub mod output;

use std::env::Args;
use std::marker::PhantomData;
use std::path::PathBuf;

use anyhow::Context;
use mcvm_core::net::game_files::version_manifest::VersionEntry;
use serde::de::DeserializeOwned;

use crate::hooks::{
	AddVersions, Hook, ModifyInstanceConfig, ModifyInstanceConfigResult, OnInstanceSetup,
	OnInstanceSetupArg, OnLoad, Subcommand, WhileInstanceLaunch, WhileInstanceLaunchArg,
};
use crate::output::OutputAction;

use self::output::PluginOutput;
pub use mcvm_shared::output::*;

/// A plugin definition
pub struct CustomPlugin {
	name: String,
	args: Args,
	hook: String,
	ctx: StoredHookContext,
}

impl CustomPlugin {
	/// Create a new plugin definition
	pub fn new(name: &str) -> anyhow::Result<Self> {
		let mut args = std::env::args();
		args.nth(0);
		let hook = args.nth(0).context("Missing hook to run")?;
		let custom_config = std::env::var("MCVM_CUSTOM_CONFIG").ok();
		let ctx = StoredHookContext {
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
		f: impl FnOnce(HookContext<OnLoad>, ()) -> anyhow::Result<()>,
	) -> anyhow::Result<()> {
		self.handle_hook::<OnLoad>(|_| Ok(()), f)
	}

	/// Bind to the subcommand hook
	pub fn subcommand(
		&mut self,
		f: impl FnOnce(HookContext<Subcommand>, Vec<String>) -> anyhow::Result<()>,
	) -> anyhow::Result<()> {
		self.handle_hook::<Subcommand>(Self::get_hook_arg, f)
	}

	/// Bind to the modify_instance_config hook
	pub fn modify_instance_config(
		&mut self,
		f: impl FnOnce(
			HookContext<ModifyInstanceConfig>,
			serde_json::Map<String, serde_json::Value>,
		) -> anyhow::Result<ModifyInstanceConfigResult>,
	) -> anyhow::Result<()> {
		self.handle_hook::<ModifyInstanceConfig>(Self::get_hook_arg, f)
	}

	/// Bind to the add_versions hook
	pub fn add_versions(
		&mut self,
		f: impl FnOnce(HookContext<AddVersions>, ()) -> anyhow::Result<Vec<VersionEntry>>,
	) -> anyhow::Result<()> {
		self.handle_hook::<AddVersions>(Self::get_hook_arg, f)
	}

	/// Bind to the on_instance_setup hook
	pub fn on_instance_setup(
		&mut self,
		f: impl FnOnce(HookContext<OnInstanceSetup>, OnInstanceSetupArg) -> anyhow::Result<()>,
	) -> anyhow::Result<()> {
		self.handle_hook::<OnInstanceSetup>(Self::get_hook_arg, f)
	}

	/// Bind to the while_instance_launch hook
	pub fn while_instance_launch(
		&mut self,
		f: impl FnOnce(HookContext<WhileInstanceLaunch>, WhileInstanceLaunchArg) -> anyhow::Result<()>,
	) -> anyhow::Result<()> {
		self.handle_hook::<WhileInstanceLaunch>(Self::get_hook_arg, f)
	}

	/// Handle a hook
	fn handle_hook<H: Hook>(
		&mut self,
		arg: impl FnOnce(&mut Self) -> anyhow::Result<H::Arg>,
		f: impl FnOnce(HookContext<H>, H::Arg) -> anyhow::Result<H::Result>,
	) -> anyhow::Result<()> {
		if self.hook == H::get_name_static() {
			let arg = arg(self)?;
			let ctx = HookContext {
				ctx: &mut self.ctx,
				_h: PhantomData,
			};
			let result = f(ctx, arg)?;
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

/// Stored hook context
struct StoredHookContext {
	custom_config: Option<String>,
	output: PluginOutput,
}

/// Argument passed to every hook
pub struct HookContext<'ctx, H: Hook> {
	ctx: &'ctx mut StoredHookContext,
	_h: PhantomData<H>,
}

impl<'ctx, H: Hook> HookContext<'ctx, H> {
	/// Get the custom configuration for the plugin passed into the hook
	pub fn get_custom_config(&self) -> Option<&str> {
		self.ctx.custom_config.as_deref()
	}

	/// Get the plugin's output stream
	pub fn get_output(&mut self) -> &mut PluginOutput {
		&mut self.ctx.output
	}

	/// Get the mcvm data directory path
	pub fn get_data_dir(&self) -> anyhow::Result<PathBuf> {
		get_env_path("MCVM_DATA_DIR").context("Failed to get directory from environment variable")
	}

	/// Get the mcvm config directory path
	pub fn get_config_dir(&self) -> anyhow::Result<PathBuf> {
		get_env_path("MCVM_CONFIG_DIR").context("Failed to get directory from environment variable")
	}
}

/// Get a path from an environment variable
fn get_env_path(var: &str) -> Option<PathBuf> {
	let var = std::env::var_os(var);
	var.map(PathBuf::from)
}
