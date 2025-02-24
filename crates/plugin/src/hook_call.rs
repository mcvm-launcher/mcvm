use std::{
	io::{BufRead, BufReader},
	ops::Deref,
	path::Path,
	process::{Child, ChildStdout, Command},
	sync::{Arc, Mutex},
};

use anyhow::{anyhow, bail, Context};
use mcvm_core::Paths;
use mcvm_shared::output::MCVMOutput;

use crate::{hooks::Hook, output::OutputAction};

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

/// Argument struct for the hook call function
pub struct HookCallArg<'a, H: Hook> {
	/// The command to run
	pub cmd: &'a str,
	/// The argument to the hook
	pub arg: &'a H::Arg,
	/// Additional arguments for the executable
	pub additional_args: &'a [String],
	/// The working directory for the executable
	pub working_dir: Option<&'a Path>,
	/// Whether to use base64 encoding
	pub use_base64: bool,
	/// Custom configuration for the plugin
	pub custom_config: Option<String>,
	/// State for the plugin
	pub state: Arc<Mutex<serde_json::Value>>,
	/// Paths
	pub paths: &'a Paths,
	/// The version of MCVM
	pub mcvm_version: Option<&'a str>,
	/// The ID of the plugin
	pub plugin_id: &'a str,
}

pub(crate) fn call<H: Hook>(
	hook: &H,
	arg: HookCallArg<'_, H>,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<HookHandle<H>>
where
	H: Sized,
{
	let _ = o;
	let hook_arg = serde_json::to_string(arg.arg).context("Failed to serialize hook argument")?;

	let cmd = arg.cmd.replace(
		PLUGIN_DIR_TOKEN,
		&arg.working_dir
			.map(|x| x.to_string_lossy().to_string())
			.unwrap_or_default(),
	);
	let mut cmd = Command::new(cmd);

	cmd.args(arg.additional_args);
	cmd.arg(hook.get_name());
	cmd.arg(hook_arg);

	// Set up environment
	if let Some(custom_config) = arg.custom_config {
		cmd.env(CUSTOM_CONFIG_ENV, custom_config);
	}
	cmd.env(DATA_DIR_ENV, &arg.paths.data);
	cmd.env(CONFIG_DIR_ENV, arg.paths.project.config_dir());
	if let Some(mcvm_version) = arg.mcvm_version {
		cmd.env(MCVM_VERSION_ENV, mcvm_version);
	}
	cmd.env(MCVM_PLUGIN_ENV, "1");
	if let Some(working_dir) = arg.working_dir {
		cmd.current_dir(working_dir);
	}
	{
		let lock = arg.state.lock().map_err(|x| anyhow!("{x}"))?;
		// Don't send null state to improve performance
		if !lock.is_null() {
			let state =
				serde_json::to_string(lock.deref()).context("Failed to serialize plugin state")?;
			cmd.env(PLUGIN_STATE_ENV, state);
		}
	}

	if H::get_takes_over() {
		cmd.spawn()?.wait()?;

		Ok(HookHandle::constant(
			H::Result::default(),
			arg.plugin_id.to_string(),
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
			plugin_state: Some(arg.state),
			use_base64: arg.use_base64,
			plugin_id: arg.plugin_id.to_string(),
		};

		Ok(handle)
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
