use std::{
	collections::VecDeque,
	io::{BufRead, BufReader, Write},
	path::Path,
	process::{Child, ChildStdin, ChildStdout, Command},
	sync::{Arc, Mutex},
	time::Instant,
};

use anyhow::{anyhow, bail, Context};
use mcvm_core::Paths;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel, NoOp};

use crate::{
	hooks::Hook,
	input_output::{CommandResult, InputAction, OutputAction},
	plugin::{PluginPersistence, DEFAULT_PROTOCOL_VERSION},
	plugin_debug_enabled,
};

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
/// The environment variable that tells what version of the hook this is
pub static HOOK_VERSION_ENV: &str = "MCVM_HOOK_VERSION";
/// The environment variable with the list of plugins
pub static PLUGIN_LIST_ENV: &str = "MCVM_PLUGIN_LIST";

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
	/// Persistent data for the plugin
	pub persistence: Arc<Mutex<PluginPersistence>>,
	/// Paths
	pub paths: &'a Paths,
	/// The version of MCVM
	pub mcvm_version: Option<&'a str>,
	/// The ID of the plugin
	pub plugin_id: &'a str,
	/// The list of all enabled plugins and their versions
	pub plugin_list: &'a [String],
	/// The protocol version
	pub protocol_version: u16,
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

	let plugin_dir = arg
		.working_dir
		.map(|x| x.to_string_lossy().to_string())
		.unwrap_or_default();
	let cmd = arg.cmd.replace(PLUGIN_DIR_TOKEN, &plugin_dir);
	let mut cmd = Command::new(cmd);

	for arg in arg.additional_args {
		cmd.arg(arg.replace(PLUGIN_DIR_TOKEN, &plugin_dir));
	}
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
	cmd.env(HOOK_VERSION_ENV, H::get_version().to_string());
	{
		let lock = arg.persistence.lock().map_err(|x| anyhow!("{x}"))?;
		// Don't send null state to improve performance
		if !lock.state.is_null() {
			let state =
				serde_json::to_string(&lock.state).context("Failed to serialize plugin state")?;
			cmd.env(PLUGIN_STATE_ENV, state);
		}
	}
	let plugin_list = arg.plugin_list.join(",");
	cmd.env(PLUGIN_LIST_ENV, plugin_list);

	if plugin_debug_enabled() {
		o.display(
			MessageContents::Simple(format!("{cmd:?}")),
			MessageLevel::Debug,
		);
	}

	if H::get_takes_over() {
		cmd.spawn().context("Failed to run hook command")?.wait()?;

		Ok(HookHandle::constant(
			H::Result::default(),
			arg.plugin_id.to_string(),
		))
	} else {
		cmd.stdout(std::process::Stdio::piped());
		cmd.stdin(std::process::Stdio::piped());

		let mut child = cmd.spawn()?;

		let stdout = child.stdout.take().unwrap();
		let stdout_reader = BufReader::new(stdout);

		let stdin = child.stdin.take().unwrap();

		let start_time = if std::env::var("MCVM_PLUGIN_PROFILE").is_ok_and(|x| x == "1") {
			Some(Instant::now())
		} else {
			None
		};

		let handle = HookHandle {
			inner: HookHandleInner::Process {
				child,
				stdout: stdout_reader,
				stdin: stdin,
				line_buf: String::new(),
				result: None,
			},
			plugin_persistence: Some(arg.persistence),
			use_base64: arg.use_base64,
			protocol_version: arg.protocol_version,
			plugin_id: arg.plugin_id.to_string(),
			command_results: VecDeque::new(),
			start_time,
		};

		Ok(handle)
	}
}

/// Handle returned by running a hook. Make sure to await it if you need to.
#[must_use]
pub struct HookHandle<H: Hook> {
	inner: HookHandleInner<H>,
	plugin_persistence: Option<Arc<Mutex<PluginPersistence>>>,
	use_base64: bool,
	protocol_version: u16,
	plugin_id: String,
	command_results: VecDeque<CommandResult>,
	start_time: Option<Instant>,
}

impl<H: Hook> HookHandle<H> {
	/// Create a new constant handle
	pub fn constant(result: H::Result, plugin_id: String) -> Self {
		Self {
			inner: HookHandleInner::Constant(result),
			plugin_persistence: None,
			use_base64: true,
			protocol_version: DEFAULT_PROTOCOL_VERSION,
			plugin_id,
			command_results: VecDeque::new(),
			start_time: None,
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
				stdin,
				..
			} => {
				line_buf.clear();
				let result_len = stdout.read_line(line_buf)?;
				// EoF
				if result_len == 0 {
					return Ok(true);
				}
				let line = line_buf.trim_end_matches("\r\n").trim_end_matches('\n');

				let action =
					OutputAction::deserialize(line, self.use_base64, self.protocol_version)
						.context("Failed to deserialize plugin action")?;

				let Some(action) = action else {
					if let Some(message) = line.strip_prefix("$_") {
						println!("{message}");
					}
					return Ok(false);
				};

				let persistence = self
					.plugin_persistence
					.as_mut()
					.context("Hook handle does not have a reference to persistent plugin data")?;
				let mut persistence_lock = persistence.lock().map_err(|x| anyhow!("{x}"))?;

				// Send command results from the worker to this hook
				if let Some(worker) = &mut persistence_lock.worker {
					while let Some(result) = worker.command_results.pop_front() {
						let action = InputAction::CommandResult(result)
							.serialize(self.protocol_version)
							.context("Failed to serialize input action")?;
						writeln!(stdin, "{action}")
							.context("Failed to write input action to plugin")?;
					}
				}

				match action {
					OutputAction::SetResult(new_result) => {
						// Before version 3, this was just a string
						let new_result = if self.protocol_version < 3 {
							let string: String = serde_json::from_value(new_result)
								.context("Failed to deserialize hook result")?;
							serde_json::from_str(&string)
								.context("Failed to deserialize hook result")?
						} else {
							serde_json::from_value(new_result)
								.context("Failed to deserialize hook result")?
						};
						*result = Some(new_result);

						if let Some(start_time) = &self.start_time {
							let now = Instant::now();
							let delta = now.duration_since(*start_time);
							o.display(
								MessageContents::Simple(format!(
									"Plugin {} took {delta:?} to run hook",
									self.plugin_id
								)),
								MessageLevel::Important,
							);
						}

						// We can stop polling early
						return Ok(true);
					}
					OutputAction::SetState(new_state) => {
						persistence_lock.state = new_state;
					}
					OutputAction::RunWorkerCommand { command, payload } => {
						let worker = persistence_lock.worker.as_mut().context(
							"Command was called on plugin worker, but the worker was not started",
						)?;
						worker
							.send_input_action(InputAction::Command { command, payload })
							.context("Failed to send command to worker")?;
					}
					OutputAction::SetCommandResult(result) => {
						self.command_results.push_back(result)
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

	/// Sends an action to the plugin
	pub fn send_input_action(&mut self, action: InputAction) -> anyhow::Result<()> {
		if let HookHandleInner::Process { stdin, .. } = &mut self.inner {
			let action = action
				.serialize(self.protocol_version)
				.context("Failed to serialize input action")?;
			writeln!(stdin, "{action}").context("Failed to write input action to plugin")?;
		}

		Ok(())
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
						bail!(
							"Hook from plugin '{}' returned a non-zero exit code of {}",
							self.plugin_id,
							exit_code
						);
					} else {
						bail!(
							"Hook from plugin '{}' returned a non-zero exit code",
							self.plugin_id
						);
					}
				}

				let result = result.with_context(|| {
					format!(
						"Plugin hook for plugin '{}' did not return a result",
						self.plugin_id
					)
				})?;

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

	/// Terminate the hook gracefully, without getting the result
	pub fn terminate(mut self) {
		let result = self.send_input_action(InputAction::Terminate);
		if result.is_err() {
			let _ = self.kill(&mut NoOp);
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
		stdin: ChildStdin,
		result: Option<H::Result>,
	},
	/// Result is a constant, either from a constant hook or a takeover hook
	Constant(H::Result),
}
