/// Output back to the main MCVM process
pub mod output;

use std::env::Args;
use std::marker::PhantomData;
use std::path::PathBuf;

use anyhow::{bail, Context};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::hook_call::{
	CONFIG_DIR_ENV, CUSTOM_CONFIG_ENV, DATA_DIR_ENV, HOOK_VERSION_ENV, PLUGIN_LIST_ENV,
	PLUGIN_STATE_ENV,
};
use crate::hooks::Hook;
use crate::output::OutputAction;
use crate::plugin::{PluginManifest, NEWEST_PROTOCOL_VERSION};

use self::output::PluginOutput;

/// A plugin definition
pub struct CustomPlugin {
	id: String,
	settings: PluginSettings,
	args: Args,
	/// The hook that is being run
	hook: String,
	stored_ctx: StoredHookContext,
}

macro_rules! hook_interface {
	($name:ident, $name2:literal, $hook:ident, $arg:expr) => {
		/// Bind to the subcommand hook
		#[doc = concat!("Bind to the ", $name2, " hook")]
		pub fn $name(
			&mut self,
			f: impl FnOnce(
				HookContext<$crate::hooks::$hook>,
				<$crate::hooks::$hook as Hook>::Arg,
			) -> anyhow::Result<<$crate::hooks::$hook as Hook>::Result>,
		) -> anyhow::Result<()> {
			self.handle_hook::<$crate::hooks::$hook>($arg, f)
		}
	};

	($name:ident, $name2:literal, $hook:ident) => {
		hook_interface!($name, $name2, $hook, Self::get_hook_arg);
	};
}

impl CustomPlugin {
	/// Create a new plugin definition
	pub fn new(id: &str) -> anyhow::Result<Self> {
		Self::with_settings(id, PluginSettings::default())
	}

	/// Create a new plugin definition with more advanced settings
	pub fn with_settings(id: &str, settings: PluginSettings) -> anyhow::Result<Self> {
		let mut args = std::env::args();
		args.next();
		let hook = args.next().context("Missing hook to run")?;
		let custom_config = std::env::var(CUSTOM_CONFIG_ENV).ok();
		let stored_ctx = StoredHookContext {
			custom_config,
			output: PluginOutput::new(settings.use_base64, settings.protocol_version),
		};
		Ok(Self {
			id: id.into(),
			settings,
			args,
			hook,
			stored_ctx,
		})
	}

	/// Create a new plugin definition from manifest file contents
	pub fn from_manifest_file(id: &str, manifest: &str) -> anyhow::Result<Self> {
		let manifest =
			serde_json::from_str(manifest).context("Failed to deserialize plugin manifest")?;
		Self::from_manifest(id, &manifest)
	}

	/// Create a new plugin definition from a manifest
	pub fn from_manifest(id: &str, manifest: &PluginManifest) -> anyhow::Result<Self> {
		let settings = PluginSettings {
			use_base64: !manifest.raw_transfer,
			protocol_version: manifest.protocol_version.unwrap_or(NEWEST_PROTOCOL_VERSION),
		};

		Self::with_settings(id, settings)
	}

	/// Get the ID of the plugin
	pub fn get_id(&self) -> &str {
		&self.id
	}

	hook_interface!(on_load, "on_load", OnLoad, |_| Ok(()));
	hook_interface!(subcommand, "subcommand", Subcommand);
	hook_interface!(
		modify_instance_config,
		"modify_instance_config",
		ModifyInstanceConfig
	);
	hook_interface!(add_versions, "add_versions", AddVersions);
	hook_interface!(on_instance_setup, "on_instance_setup", OnInstanceSetup);
	hook_interface!(on_instance_launch, "on_instance_launch", OnInstanceLaunch);
	hook_interface!(
		while_instance_launch,
		"while_instance_launch",
		WhileInstanceLaunch
	);
	hook_interface!(on_instance_stop, "on_instance_stop", OnInstanceStop);
	hook_interface!(
		custom_package_instruction,
		"custom_package_instruction",
		CustomPackageInstruction
	);
	hook_interface!(handle_auth, "handle_auth", HandleAuth);
	hook_interface!(add_translations, "add_translations", AddTranslations);
	hook_interface!(
		add_instance_transfer_formats,
		"add_instance_transfer_formats",
		AddInstanceTransferFormats
	);
	hook_interface!(export_instance, "export_instance", ExportInstance);
	hook_interface!(import_instance, "import_instance", ImportInstance);
	hook_interface!(
		add_supported_game_modifications,
		"add_supported_game_modifications",
		AddSupportedGameModifications
	);
	hook_interface!(add_instances, "add_instances", AddInstances);

	/// Handle a hook
	fn handle_hook<H: Hook>(
		&mut self,
		arg: impl FnOnce(&mut Self) -> anyhow::Result<H::Arg>,
		f: impl FnOnce(HookContext<H>, H::Arg) -> anyhow::Result<H::Result>,
	) -> anyhow::Result<()> {
		// Check if we are running the given hook
		if self.hook == H::get_name_static() {
			// Check that the hook version of MCVM matches our hook version
			let expected_version = std::env::var(HOOK_VERSION_ENV);
			if let Ok(expected_version) = expected_version {
				if expected_version != H::get_version().to_string() {
					bail!("Hook version does not match. Try updating the plugin or MCVM.");
				}
			}

			let arg = arg(self)?;
			let mut state = None;
			let mut state_has_changed = false;
			let ctx = HookContext {
				stored_ctx: &mut self.stored_ctx,
				state: &mut state,
				state_has_changed: &mut state_has_changed,
				_h: PhantomData,
			};
			let result = f(ctx, arg)?;
			if !H::get_takes_over() {
				// Output result
				let serialized = serde_json::to_string(&result)?;
				let action = OutputAction::SetResult(serialized);
				println!(
					"{}",
					action
						.serialize(self.settings.use_base64, self.settings.protocol_version)
						.context("Failed to serialize hook result")?
				);

				// Output state
				if state_has_changed {
					if let Some(state) = state {
						let action = OutputAction::SetState(state);
						println!(
							"{}",
							action
								.serialize(self.settings.use_base64, self.settings.protocol_version)
								.context("Failed to serialize new hook state")?
						);
					}
				}
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

/// Stored hook context in the CustomPlugin, shared with the HookContext
struct StoredHookContext {
	custom_config: Option<String>,
	output: PluginOutput,
}

/// Argument passed to every hook
pub struct HookContext<'ctx, H: Hook> {
	stored_ctx: &'ctx mut StoredHookContext,
	state: &'ctx mut Option<serde_json::Value>,
	state_has_changed: &'ctx mut bool,
	_h: PhantomData<H>,
}

impl<'ctx, H: Hook> HookContext<'ctx, H> {
	/// Get the custom configuration for the plugin passed into the hook
	pub fn get_custom_config(&self) -> Option<&str> {
		self.stored_ctx.custom_config.as_deref()
	}

	/// Get the plugin's output stream
	pub fn get_output(&mut self) -> &mut PluginOutput {
		&mut self.stored_ctx.output
	}

	/// Get the mcvm data directory path
	pub fn get_data_dir(&self) -> anyhow::Result<PathBuf> {
		get_env_path(DATA_DIR_ENV).context("Failed to get directory from environment variable")
	}

	/// Get the mcvm config directory path
	pub fn get_config_dir(&self) -> anyhow::Result<PathBuf> {
		get_env_path(CONFIG_DIR_ENV).context("Failed to get directory from environment variable")
	}

	/// Get the list of enabled plugins
	pub fn get_plugin_list(&self) -> Vec<PluginListEntry> {
		let Ok(var) = std::env::var(PLUGIN_LIST_ENV) else {
			return Vec::new();
		};

		var.split(",")
			.map(|x| PluginListEntry { id: x.to_string() })
			.collect()
	}

	/// Get the persistent plugin state, kept the same for this entire hook handler,
	/// along with a default state
	pub fn get_persistent_state(
		&mut self,
		default: impl Serialize,
	) -> anyhow::Result<&mut serde_json::Value> {
		match &mut self.state {
			Some(val) => Ok(val),
			self_state @ None => {
				if let Ok(state) = std::env::var(PLUGIN_STATE_ENV) {
					**self_state = Some(serde_json::from_str(&state)?);
				} else {
					**self_state = Some(serde_json::to_value(default)?);
				};
				Ok(self_state.as_mut().expect("We just set it man"))
			}
		}
	}

	/// Set the persistent plugin state
	pub fn set_persistent_state(&mut self, state: impl Serialize) -> anyhow::Result<()> {
		let state = serde_json::to_value(state)?;
		*self.state = Some(state);
		*self.state_has_changed = true;

		Ok(())
	}
}

/// Settings for a plugin using the API
pub struct PluginSettings {
	/// Whether to use base64 encoding in the plugin protocol.
	/// If this is set to false, raw_transfer must be true in the manifest
	pub use_base64: bool,
	/// The protocol version to use
	pub protocol_version: u16,
}

impl Default for PluginSettings {
	fn default() -> Self {
		Self {
			use_base64: true,
			protocol_version: NEWEST_PROTOCOL_VERSION,
		}
	}
}

/// An entry in the list of enabled plugins
pub struct PluginListEntry {
	/// The ID of the entry
	pub id: String,
}

/// Get a path from an environment variable
fn get_env_path(var: &str) -> Option<PathBuf> {
	let var = std::env::var_os(var);
	var.map(PathBuf::from)
}
