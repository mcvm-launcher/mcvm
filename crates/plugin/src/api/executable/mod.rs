/// Output back to the main Nitrolaunch process
pub mod output;

use std::env::Args;
use std::io::{Stdin, Write};
use std::marker::PhantomData;
use std::path::PathBuf;

use anyhow::{Context, bail};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::api::executable::output::poll_input_action;
use crate::hook::Hook;
use crate::hook::{
	CONFIG_DIR_ENV, CUSTOM_CONFIG_ENV, DATA_DIR_ENV, HOOK_VERSION_ENV, PLUGIN_LIST_ENV,
	PLUGIN_STATE_ENV,
};
use crate::input_output::{InputAction, OutputAction};
use crate::plugin::{NEWEST_PROTOCOL_VERSION, PluginManifest};

use self::output::ExecutablePluginOutput;

/// A custom executable plugin definition
pub struct ExecutablePlugin {
	id: String,
	settings: PluginSettings,
	args: Args,
	/// The hook that is being run
	hook: String,
	stored_ctx: StoredHookContext,
	stdin: Stdin,
}

impl ExecutablePlugin {
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
			output: ExecutablePluginOutput::new(settings.use_base64, settings.protocol_version),
		};
		Ok(Self {
			id: id.into(),
			settings,
			args,
			hook,
			stored_ctx,
			stdin: std::io::stdin(),
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

	/// Handle a hook
	pub(crate) fn handle_hook<H: Hook>(
		&mut self,
		arg: impl FnOnce(&mut Self) -> anyhow::Result<H::Arg>,
		f: impl FnOnce(HookContext<H>, H::Arg) -> anyhow::Result<H::Result>,
	) -> anyhow::Result<()> {
		// Check if we are running the given hook
		if self.hook == H::get_name_static() {
			// Check that the hook version of Nitrolaunch matches our hook version
			let expected_version = std::env::var(HOOK_VERSION_ENV);
			if let Ok(expected_version) = expected_version
				&& expected_version != H::get_version().to_string()
			{
				bail!("Hook version does not match. Try updating the plugin or Nitrolaunch.");
			}

			let arg = arg(self)?;
			let mut state = None;
			let mut state_has_changed = false;
			let ctx = HookContext {
				stored_ctx: &mut self.stored_ctx,
				state: &mut state,
				state_has_changed: &mut state_has_changed,
				stdin: &mut self.stdin,
				protocol_version: self.settings.protocol_version,
				_h: PhantomData,
			};

			let mut stdout = std::io::stdout();

			let result = f(ctx, arg);
			let result = match result {
				Ok(result) => result,
				Err(e) => {
					if H::get_takes_over() {
						eprintln!("Error in hook: {e:?}");
					} else {
						let output = OutputAction::SetError(format!("{e:?}"))
							.serialize(self.settings.use_base64, self.settings.protocol_version)?;
						let _ = writeln!(&mut stdout, "{output}");
					}
					return Ok(());
				}
			};

			if !H::get_takes_over() {
				// Output state
				if state_has_changed && let Some(state) = state {
					let action = OutputAction::SetState(state);
					let _ = writeln!(
						&mut stdout,
						"{}",
						action
							.serialize(self.settings.use_base64, self.settings.protocol_version)
							.context("Failed to serialize new hook state")?
					);
				}

				// Output result last as it will make the plugin runner stop listening
				let serialized = if self.settings.protocol_version < 3 {
					serde_json::Value::String(serde_json::to_string(&result)?)
				} else {
					serde_json::to_value(result)?
				};
				let action = OutputAction::SetResult(serialized);
				let _ = writeln!(
					&mut stdout,
					"{}",
					action
						.serialize(self.settings.use_base64, self.settings.protocol_version)
						.context("Failed to serialize hook result")?
				);
			}
			Ok(())
		} else {
			Ok(())
		}
	}

	/// Get the first argument as the hook input
	pub(crate) fn get_hook_arg<Arg: DeserializeOwned>(&mut self) -> anyhow::Result<Arg> {
		let arg = self.args.nth(0).context("Hook argument missing")?;
		serde_json::from_str(&arg).context("Failed to deserialize hook argument")
	}
}

/// Stored hook context in the ExecutablePlugin, shared with the HookContext
struct StoredHookContext {
	custom_config: Option<String>,
	output: ExecutablePluginOutput,
}

/// Argument passed to every hook
pub struct HookContext<'ctx, H: Hook> {
	stored_ctx: &'ctx mut StoredHookContext,
	state: &'ctx mut Option<serde_json::Value>,
	state_has_changed: &'ctx mut bool,
	stdin: &'ctx mut Stdin,
	protocol_version: u16,
	_h: PhantomData<H>,
}

impl<H: Hook> HookContext<'_, H> {
	/// Get the custom configuration for the plugin passed into the hook
	pub fn get_custom_config(&self) -> Option<&str> {
		self.stored_ctx.custom_config.as_deref()
	}

	/// Get the plugin's output stream
	pub fn get_output(&mut self) -> &mut ExecutablePluginOutput {
		&mut self.stored_ctx.output
	}

	/// Get the Nitrolaunch data directory path
	pub fn get_data_dir(&self) -> anyhow::Result<PathBuf> {
		get_env_path(DATA_DIR_ENV).context("Failed to get directory from environment variable")
	}

	/// Get the Nitrolaunch config directory path
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

	/// Gets the latest input action
	pub fn poll(&mut self) -> anyhow::Result<Option<InputAction>> {
		poll_input_action(self.stdin, self.protocol_version)
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
