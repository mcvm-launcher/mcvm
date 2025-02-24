use std::io::{BufRead, BufReader};
use std::ops::Deref;
use std::path::Path;
use std::process::{Child, ChildStdout, Command};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, bail, Context};
use mcvm_core::net::minecraft::MinecraftUserProfile;
use mcvm_core::util::versions::MinecraftVersionDeser;
use mcvm_core::{net::game_files::version_manifest::VersionEntry, Paths};
use mcvm_pkg::script_eval::AddonInstructionData;
use mcvm_pkg::{RecommendedPackage, RequiredPackage};
use mcvm_shared::lang::translate::LanguageMap;
use mcvm_shared::modifications::{ClientType, ServerType};
use mcvm_shared::pkg::PackageID;
use mcvm_shared::{output::MCVMOutput, versions::VersionInfo, Side};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::output::OutputAction;

/// The substitution token for the plugin directory in the command
pub static PLUGIN_DIR_TOKEN: &str = "${PLUGIN_DIR}";
/// The environment variable for custom config passed to a hook
pub static CUSTOM_CONFIG_ENV: &str = "MCVM_CUSTOM_CONFIG";
/// The environment variable for the data directory passed to a hook
pub static DATA_DIR_ENV: &str = "MCVM_DATA_DIR";
/// The environment variable for the config directory passed to a hook
pub static CONFIG_DIR_ENV: &str = "MCVM_CONFIG_DIR";
/// The environment variable for the plugin state passed to a hook
pub static PLUGIN_STATE_ENV: &str = "MCVM_PLUGIN_STATE";
/// The environment variable for the version of MCVM
pub static MCVM_VERSION_ENV: &str = "MCVM_VERSION";
/// The environment variable that tells the executable it is running as a plugin
pub static MCVM_PLUGIN_ENV: &str = "MCVM_PLUGIN";

/// Trait for a hook that can be called
pub trait Hook {
	/// The type for the argument that goes into the hook
	type Arg: Serialize + DeserializeOwned;
	/// The type for the result from the hook
	type Result: DeserializeOwned + Serialize + Default;

	/// Get the name of the hook
	fn get_name(&self) -> &'static str {
		Self::get_name_static()
	}

	/// Get the name of the hook statically
	fn get_name_static() -> &'static str;

	/// Get whether the hook should forward all output to the terminal
	fn get_takes_over() -> bool {
		false
	}

	/// Call the hook using the specified program
	#[allow(clippy::too_many_arguments)]
	fn call(
		&self,
		cmd: &str,
		arg: &Self::Arg,
		additional_args: &[String],
		working_dir: Option<&Path>,
		use_base64: bool,
		custom_config: Option<String>,
		state: Arc<Mutex<serde_json::Value>>,
		paths: &Paths,
		mcvm_version: Option<&str>,
		plugin_id: &str,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<HookHandle<Self>>
	where
		Self: Sized,
	{
		let _ = o;
		let arg = serde_json::to_string(arg).context("Failed to serialize hook argument")?;

		let cmd = cmd.replace(
			PLUGIN_DIR_TOKEN,
			&working_dir
				.map(|x| x.to_string_lossy().to_string())
				.unwrap_or_default(),
		);
		let mut cmd = Command::new(cmd);

		cmd.args(additional_args);
		cmd.arg(self.get_name());
		cmd.arg(arg);

		// Set up environment
		if let Some(custom_config) = custom_config {
			cmd.env(CUSTOM_CONFIG_ENV, custom_config);
		}
		cmd.env(DATA_DIR_ENV, &paths.data);
		cmd.env(CONFIG_DIR_ENV, paths.project.config_dir());
		if let Some(mcvm_version) = mcvm_version {
			cmd.env(MCVM_VERSION_ENV, mcvm_version);
		}
		cmd.env(MCVM_PLUGIN_ENV, "1");
		if let Some(working_dir) = working_dir {
			cmd.current_dir(working_dir);
		}
		{
			let lock = state.lock().map_err(|x| anyhow!("{x}"))?;
			// Don't send null state to improve performance
			if !lock.is_null() {
				let state = serde_json::to_string(lock.deref())
					.context("Failed to serialize plugin state")?;
				cmd.env(PLUGIN_STATE_ENV, state);
			}
		}

		if Self::get_takes_over() {
			cmd.spawn()?.wait()?;

			Ok(HookHandle::constant(
				Self::Result::default(),
				plugin_id.to_string(),
			))
		} else {
			cmd.stdout(std::process::Stdio::piped());

			let mut child = cmd.spawn()?;

			let stdout = child.stdout.take().unwrap();
			let stdout_reader = BufReader::new(stdout);

			let handle = HookHandle {
				inner: HookHandleInner::Process {
					child,
					stdout: stdout_reader,
					line_buf: String::new(),
					result: None,
				},
				plugin_state: Some(state),
				use_base64,
				plugin_id: plugin_id.to_string(),
			};

			Ok(handle)
		}
	}
}

/// Handle returned by running a hook. Make sure to await it if you need to.
#[must_use]
pub struct HookHandle<H: Hook> {
	inner: HookHandleInner<H>,
	plugin_state: Option<Arc<Mutex<serde_json::Value>>>,
	use_base64: bool,
	plugin_id: String,
}

impl<H: Hook> HookHandle<H> {
	/// Create a new constant handle
	pub fn constant(result: H::Result, plugin_id: String) -> Self {
		Self {
			inner: HookHandleInner::Constant(result),
			plugin_state: None,
			use_base64: true,
			plugin_id,
		}
	}

	/// Get the ID of the plugin that returned this handle
	pub fn get_id(&self) -> &String {
		&self.plugin_id
	}

	/// Poll the handle, returning true if the handle is ready
	pub fn poll(&mut self, o: &mut impl MCVMOutput) -> anyhow::Result<bool> {
		match &mut self.inner {
			HookHandleInner::Process {
				line_buf,
				stdout,
				result,
				..
			} => {
				line_buf.clear();
				let result_len = stdout.read_line(line_buf)?;
				// EoF
				if result_len == 0 {
					return Ok(true);
				}
				let line = line_buf.trim_end_matches("\r\n").trim_end_matches('\n');

				let action = OutputAction::deserialize(line, self.use_base64)
					.context("Failed to deserialize plugin action")?;
				match action {
					OutputAction::SetResult(new_result) => {
						*result = Some(
							serde_json::from_str(&new_result)
								.context("Failed to deserialize hook result")?,
						);
					}
					OutputAction::SetState(new_state) => {
						let state = self
							.plugin_state
							.as_mut()
							.context("Hook handle does not have a reference to persistent state")?;
						let mut lock = state.lock().map_err(|x| anyhow!("{x}"))?;
						*lock = new_state;
					}
					OutputAction::Text(text, level) => {
						o.display_text(text, level);
					}
					OutputAction::Message(message) => {
						o.display_message(message);
					}
					OutputAction::StartProcess => {
						o.start_process();
					}
					OutputAction::EndProcess => {
						o.end_process();
					}
					OutputAction::StartSection => {
						o.start_section();
					}
					OutputAction::EndSection => {
						o.end_section();
					}
				}

				Ok(false)
			}
			HookHandleInner::Constant(..) => Ok(true),
		}
	}

	/// Get the result of the hook by waiting for it
	pub fn result(mut self, o: &mut impl MCVMOutput) -> anyhow::Result<H::Result> {
		if let HookHandleInner::Process { .. } = &self.inner {
			loop {
				let result = self.poll(o)?;
				if result {
					break;
				}
			}
		}

		match self.inner {
			HookHandleInner::Constant(result) => Ok(result),
			HookHandleInner::Process {
				mut child, result, ..
			} => {
				let cmd_result = child.wait()?;

				if !cmd_result.success() {
					if let Some(exit_code) = cmd_result.code() {
						bail!("Hook returned a non-zero exit code of {}", exit_code);
					} else {
						bail!("Hook returned a non-zero exit code");
					}
				}

				let result = result.context("Plugin hook did not return a result")?;

				Ok(result)
			}
		}
	}

	/// Get the result of the hook by killing it
	pub fn kill(self, o: &mut impl MCVMOutput) -> anyhow::Result<Option<H::Result>> {
		let _ = o;
		match self.inner {
			HookHandleInner::Constant(result) => Ok(Some(result)),
			HookHandleInner::Process {
				mut child, result, ..
			} => {
				child.kill()?;

				Ok(result)
			}
		}
	}
}

/// The inner value for a HookHandle
enum HookHandleInner<H: Hook> {
	/// Result is coming from a running process
	Process {
		child: Child,
		line_buf: String,
		stdout: BufReader<ChildStdout>,
		result: Option<H::Result>,
	},
	/// Result is a constant, either from a constant hook or a takeover hook
	Constant(H::Result),
}

macro_rules! def_hook {
	($struct:ident, $name:literal, $desc:literal, $arg:ty, $res:ty, $($extra:tt)*) => {
		#[doc = $desc]
		pub struct $struct;

		impl Hook for $struct {
			type Arg = $arg;
			type Result = $res;

			fn get_name_static() -> &'static str {
				$name
			}

			$(
				$extra
			)*
		}
	};
}

def_hook!(
	OnLoad,
	"on_load",
	"Hook for when a plugin is loaded",
	(),
	(),
);

def_hook!(
	Subcommand,
	"subcommand",
	"Hook for when a command's subcommands are run",
	Vec<String>,
	(),
	fn get_takes_over() -> bool {
		true
	}
);

def_hook!(
	ModifyInstanceConfig,
	"modify_instance_config",
	"Hook for modifying an instance's configuration",
	serde_json::Map<String, serde_json::Value>,
	ModifyInstanceConfigResult,
);

/// Result from the ModifyInstanceConfig hook
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ModifyInstanceConfigResult {
	/// Additional JVM args to add to the instance
	pub additional_jvm_args: Vec<String>,
}

def_hook!(
	AddVersions,
	"add_versions",
	"Hook for adding extra versions to the version manifest",
	(),
	Vec<VersionEntry>,
);

def_hook!(
	OnInstanceSetup,
	"on_instance_setup",
	"Hook for doing work when setting up an instance for update or launch",
	OnInstanceSetupArg,
	(),
);

/// Argument for the OnInstanceSetup hook
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct OnInstanceSetupArg {
	/// The ID of the instance
	pub id: String,
	/// The side of the instance
	pub side: Option<Side>,
	/// Path to the instance's game dir
	pub game_dir: String,
	/// Version info for the instance
	pub version_info: VersionInfo,
	/// Custom config on the instance
	pub custom_config: serde_json::Map<String, serde_json::Value>,
}

def_hook!(
	OnInstanceLaunch,
	"on_instance_launch",
	"Hook for doing work before an instance is launched",
	InstanceLaunchArg,
	(),
);

def_hook!(
	WhileInstanceLaunch,
	"while_instance_launch",
	"Hook for running sibling processes with an instance when it is launched",
	InstanceLaunchArg,
	(),
);

def_hook!(
	OnInstanceStop,
	"on_instance_stop",
	"Hook for doing work when an instance is stopped gracefully",
	InstanceLaunchArg,
	(),
);

/// Argument for the OnInstanceLaunch and WhileInstanceLaunch hooks
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct InstanceLaunchArg {
	/// The ID of the instance
	pub id: String,
	/// The side of the instance
	pub side: Option<Side>,
	/// Path to the instance's dir
	pub dir: String,
	/// Path to the instance's game dir
	pub game_dir: String,
	/// Version info for the instance
	pub version_info: VersionInfo,
	/// Custom config on the instance
	pub custom_config: serde_json::Map<String, serde_json::Value>,
	/// The PID of the instance process
	pub pid: Option<u32>,
}

def_hook!(
	CustomPackageInstruction,
	"custom_package_instruction",
	"Hook for handling custom instructions in packages",
	CustomPackageInstructionArg,
	CustomPackageInstructionResult,
);

/// Argument for the CustomPackageInstruction hook
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CustomPackageInstructionArg {
	/// The ID of the package
	pub pkg_id: String,
}

/// Result from the CustomPackageInstruction hook
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CustomPackageInstructionResult {
	/// Whether the instruction was handled by this plugin
	pub handled: bool,
	/// The output of addon requests
	pub addon_reqs: Vec<AddonInstructionData>,
	/// The output dependencies
	pub deps: Vec<Vec<RequiredPackage>>,
	/// The output conflicts
	pub conflicts: Vec<PackageID>,
	/// The output recommendations
	pub recommendations: Vec<RecommendedPackage>,
	/// The output bundled packages
	pub bundled: Vec<PackageID>,
	/// The output compats
	pub compats: Vec<(PackageID, PackageID)>,
	/// The output package extensions
	pub extensions: Vec<PackageID>,
	/// The output notices
	pub notices: Vec<String>,
}

def_hook!(
	HandleAuth,
	"handle_auth",
	"Hook for handling authentication for custom user types",
	HandleAuthArg,
	HandleAuthResult,
);

/// Argument for the HandleAuth hook
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HandleAuthArg {
	/// The ID of the user
	pub user_id: String,
	/// The custom type of the user
	pub user_type: String,
}

/// Result from the HandleAuth hook
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HandleAuthResult {
	/// Whether the auth for this user type was handled by this plugin
	pub handled: bool,
	/// The resulting user profile
	pub profile: Option<MinecraftUserProfile>,
}

def_hook!(
	AddTranslations,
	"add_translations",
	"Hook for adding extra translations to MCVM",
	(),
	LanguageMap,
);

def_hook!(
	AddInstanceTransferFormats,
	"add_instance_transfer_formats",
	"Hook for adding information about instance transfer formats",
	(),
	Vec<InstanceTransferFormat>,
);

/// Information about an instance transfer format
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct InstanceTransferFormat {
	/// The ID for this format
	pub id: String,
	/// Info for the import side of this format
	pub import: Option<InstanceTransferFormatDirection>,
	/// Info for the export side of this format
	pub export: Option<InstanceTransferFormatDirection>,
}

/// Information about a side of an instance transfer format
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct InstanceTransferFormatDirection {
	/// Support status of the modloader
	pub modloader: InstanceTransferFeatureSupport,
	/// Support status of the mods
	pub mods: InstanceTransferFeatureSupport,
	/// Support status of the launch settings
	pub launch_settings: InstanceTransferFeatureSupport,
}

/// Support status of some feature in an instance transfer format
#[derive(Serialize, Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum InstanceTransferFeatureSupport {
	/// This feature is supported by the transfer
	#[default]
	Supported,
	/// This feature is unsupported by the nature of the format
	FormatUnsupported,
	/// This feature is not yet supported by the plugin
	PluginUnsupported,
}

def_hook!(
	ExportInstance,
	"export_instance",
	"Hook for exporting an instance",
	ExportInstanceArg,
	(),
);

/// Argument provided to the export_instance hook
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ExportInstanceArg {
	/// The ID of the transfer format being used
	pub format: String,
	/// The ID of the instance
	pub id: String,
	/// The name of the instance
	pub name: Option<String>,
	/// The side of the instance
	pub side: Option<Side>,
	/// The directory where the instance game files are located
	pub game_dir: String,
	/// The desired path for the resulting instance, as a file path
	pub result_path: String,
	/// The Minecraft version of the instance
	pub minecraft_version: Option<MinecraftVersionDeser>,
	/// The client type of the new instance
	pub client_type: Option<ClientType>,
	/// The server type of the new instance
	pub server_type: Option<ServerType>,
}

def_hook!(
	ImportInstance,
	"import_instance",
	"Hook for importing an instance",
	ImportInstanceArg,
	ImportInstanceResult,
);

/// Argument provided to the import_instance hook
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ImportInstanceArg {
	/// The ID of the transfer format being used
	pub format: String,
	/// The ID of the new instance
	pub id: String,
	/// The path to the instance to import
	pub source_path: String,
	/// The desired directory for the resulting instance
	pub result_path: String,
}

/// Result from the ImportInstance hook giving information about the new instance
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ImportInstanceResult {
	/// The ID of the transfer format being used
	pub format: String,
	/// The name of the instance
	pub name: Option<String>,
	/// The side of the instance
	pub side: Option<Side>,
	/// The Minecraft version of the instance
	pub version: Option<MinecraftVersionDeser>,
	/// The client type of the new instance
	pub client_type: Option<ClientType>,
	/// The server type of the new instance
	pub server_type: Option<ServerType>,
}
