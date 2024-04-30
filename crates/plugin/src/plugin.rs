use std::collections::HashMap;
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Context;
use mcvm_core::Paths;
use mcvm_shared::{lang::translate::LanguageMap, output::MCVMOutput};
use serde::{Deserialize, Deserializer};

use crate::hooks::{Hook, HookHandle};

/// A plugin
#[derive(Debug)]
pub struct Plugin {
	/// The plugin's ID
	id: String,
	/// The plugin's manifest
	manifest: PluginManifest,
	/// The custom config for the plugin, serialized from JSON
	custom_config: Option<String>,
	/// The working directory for the plugin
	working_dir: Option<PathBuf>,
	/// The persistent state of the plugin
	state: Arc<Mutex<serde_json::Value>>,
}

impl Plugin {
	/// Create a new plugin from an ID and manifest
	pub fn new(id: String, manifest: PluginManifest) -> Self {
		Self {
			id,
			manifest,
			custom_config: None,
			working_dir: None,
			state: Arc::new(Mutex::new(serde_json::Value::Null)),
		}
	}

	/// Get the ID of the plugin
	pub fn get_id(&self) -> &String {
		&self.id
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
		paths: &Paths,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Option<HookHandle<H>>> {
		let Some(handler) = self.manifest.hooks.get(hook.get_name()) else {
			return Ok(None);
		};
		match handler {
			HookHandler::Execute { executable, args } => hook
				.call(
					executable,
					arg,
					args,
					self.working_dir.as_deref(),
					self.custom_config.clone(),
					self.state.clone(),
					paths,
					o,
				)
				.map(Some),
			HookHandler::Constant { constant } => Ok(Some(HookHandle::constant(
				serde_json::from_value(constant.clone())?,
			))),
			HookHandler::Native { function } => {
				let arg = serde_json::to_string(arg)
					.context("Failed to serialize native hook argument")?;
				let result = function(arg).context("Native hook handler failed")?;
				let result = serde_json::from_str(&result)
					.context("Failed to deserialize native hook result")?;
				Ok(Some(HookHandle::constant(result)))
			}
		}
	}

	/// Set the custom config of the plugin
	pub fn set_custom_config(&mut self, config: serde_json::Value) -> anyhow::Result<()> {
		let serialized =
			serde_json::to_string(&config).context("Failed to serialize custom plugin config")?;
		self.custom_config = Some(serialized);
		Ok(())
	}

	/// Set the working dir of the plugin
	pub fn set_working_dir(&mut self, dir: PathBuf) {
		self.working_dir = Some(dir);
	}
}

/// Configuration for a plugin
#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct PluginManifest {
	/// The display name of the plugin
	pub name: Option<String>,
	/// The short description of the plugin
	pub description: Option<String>,
	/// The MCVM version this plugin is for
	pub mcvm_version: Option<String>,
	/// The hook handlers for the plugin
	pub hooks: HashMap<String, HookHandler>,
	/// The language map the plugin provides
	pub language_map: LanguageMap,
	/// The subcommands the plugin provides
	pub subcommands: HashMap<String, String>,
}

impl PluginManifest {
	/// Create a new PluginManifest
	pub fn new() -> Self {
		Self::default()
	}
}

/// A handler for a single hook that a plugin uses
#[derive(Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "snake_case")]
pub enum HookHandler {
	/// Handle this hook by running an executable
	Execute {
		/// The executable to run
		executable: String,
		/// Arguments for the executable
		#[serde(default)]
		args: Vec<String>,
	},
	/// Handle this hook by returning a constant result
	Constant {
		/// The constant result
		constant: serde_json::Value,
	},
	/// Handle this hook with a native function call
	Native {
		/// The function to handle the hook
		#[serde(deserialize_with = "deserialize_native_function")]
		function: Box<dyn Fn(String) -> anyhow::Result<String>>,
	},
}

impl Debug for HookHandler {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "HookHandler")
	}
}

/// Deserialize function for the native hook. No plugin manifests should ever use this,
/// so just return a function that does nothing.
fn deserialize_native_function<'de, D>(
	_: D,
) -> Result<Box<dyn Fn(String) -> anyhow::Result<String>>, D::Error>
where
	D: Deserializer<'de>,
{
	Ok(Box::new(|_| Ok(String::new())))
}
