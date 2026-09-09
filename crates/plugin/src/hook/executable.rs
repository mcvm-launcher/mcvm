use std::{collections::VecDeque, env::consts::EXE_SUFFIX, sync::Arc, time::Instant};

use anyhow::{Context, anyhow, bail};
use nitro_shared::{
	no_window,
	output::{MessageContents, NitroOutput},
};
use tokio::{
	io::AsyncWriteExt,
	process::{Child, ChildStdin, ChildStdout, Command},
	sync::Mutex,
};

use crate::{
	hook::{
		CONFIG_DIR_ENV, CUSTOM_CONFIG_ENV, DATA_DIR_ENV, EXE_EXTENSION_TOKEN, HOOK_VERSION_ENV,
		Hook, INSTANCE_LIST_ENV, NITRO_PLUGIN_ENV, NITRO_VERSION_ENV, PLUGIN_DIR_TOKEN,
		PLUGIN_LIST_ENV, PLUGIN_STATE_ENV, TEMPLATE_LIST_ENV,
		call::{HookCallArg, HookHandle},
	},
	input_output::{CommandResult, InputAction, OutputAction},
	plugin::{HookSubscription, PluginPersistence},
	plugin_debug_enabled,
	try_read::TryLineReader,
};

const READ_BUF_SIZE: usize = 32768;

/// Calls an executable hook handler
pub(crate) async fn call_executable<H: Hook + Sized>(
	hook: &H,
	arg: HookCallArg<'_, H>,
	o: &mut impl NitroOutput,
) -> anyhow::Result<HookHandle<H>> {
	let _ = o;
	let hook_arg = serde_json::to_string(arg.arg).context("Failed to serialize hook argument")?;

	let plugin_dir = arg
		.working_dir
		.map(|x| x.to_string_lossy().to_string())
		.unwrap_or_default();
	let cmd = arg.cmd.replace(PLUGIN_DIR_TOKEN, &plugin_dir);
	let cmd = cmd.replace(EXE_EXTENSION_TOKEN, EXE_SUFFIX);
	let mut cmd = Command::new(cmd);

	for arg in arg.additional_args {
		cmd.arg(arg.replace(PLUGIN_DIR_TOKEN, &plugin_dir));
	}
	cmd.arg(hook.get_name());
	cmd.arg(hook_arg);

	// Set up environment
	if let Some(custom_config) = arg.ctx.custom_config {
		cmd.env(CUSTOM_CONFIG_ENV, custom_config);
	}
	cmd.env(DATA_DIR_ENV, &arg.paths.data_dir);
	cmd.env(CONFIG_DIR_ENV, &arg.paths.config_dir);
	if let Some(nitro_version) = arg.ctx.nitro_version {
		cmd.env(NITRO_VERSION_ENV, nitro_version);
	}
	cmd.env(NITRO_PLUGIN_ENV, "1");
	cmd.env(HOOK_VERSION_ENV, H::get_version().to_string());
	{
		let lock = arg.persistence.lock().await;
		// Don't send null state to improve performance
		if !lock.state.is_null() {
			let state =
				serde_json::to_string(&lock.state).context("Failed to serialize plugin state")?;
			cmd.env(PLUGIN_STATE_ENV, state);
		}
	}
	let plugin_list = arg.ctx.plugin_list.join(",");
	cmd.env(PLUGIN_LIST_ENV, plugin_list);

	if let Some(context) = arg.ctx.global_context {
		if arg.ctx.subscriptions.contains(&HookSubscription::Instances) {
			cmd.env(
				INSTANCE_LIST_ENV,
				serde_json::to_string(&context.get_instances())?,
			);
		}
		if arg.ctx.subscriptions.contains(&HookSubscription::Templates) {
			cmd.env(
				TEMPLATE_LIST_ENV,
				serde_json::to_string(&context.get_templates())?,
			);
		}
	}

	no_window!(cmd);

	if plugin_debug_enabled() {
		o.display(MessageContents::Simple(format!("{cmd:?}")));
	}

	if H::get_takes_over() {
		cmd.spawn()
			.context("Failed to run hook command")?
			.wait()
			.await?;

		Ok(HookHandle::constant(
			H::Result::default(),
			arg.plugin_id.to_string(),
		))
	} else {
		cmd.stdout(std::process::Stdio::piped());
		cmd.stdin(std::process::Stdio::piped());

		let handle_inner = ExecutableHookHandle {
			inner: ExecutableHookHandleInner::NotStarted(cmd),
			use_base64: arg.use_base64,
			protocol_version: arg.protocol_version,
			plugin_id: arg.plugin_id.to_string(),
		};

		let handle =
			HookHandle::executable(handle_inner, arg.plugin_id.to_string(), arg.persistence);

		Ok(handle)
	}
}

/// Hook handler internals for an executable hook
pub(super) struct ExecutableHookHandle<H: Hook> {
	pub use_base64: bool,
	pub protocol_version: u16,
	pub plugin_id: String,
	inner: ExecutableHookHandleInner<H>,
}

/// Used to differentiate between an executable hook handle that has been started or not
enum ExecutableHookHandleInner<H: Hook> {
	NotStarted(Command),
	Started {
		child: Child,
		stdout: TryLineReader<ChildStdout>,
		stdin: ChildStdin,
		result: Option<H::Result>,
	},
}

impl<H: Hook> ExecutableHookHandle<H> {
	/// Ensures that this hook is started
	pub async fn ensure_started(
		&mut self,
		plugin_persistence: &mut Option<Arc<Mutex<PluginPersistence>>>,
		command_results: &mut VecDeque<CommandResult>,
		start_time: &mut Option<Instant>,
		o: &mut impl NitroOutput,
	) -> anyhow::Result<()> {
		if let ExecutableHookHandleInner::NotStarted(..) = &self.inner {
			self.poll(plugin_persistence, command_results, start_time, o)
				.await?;
		}

		Ok(())
	}

	/// Polls this hook, returning true if the polling is done and a result is available
	pub async fn poll(
		&mut self,
		plugin_persistence: &mut Option<Arc<Mutex<PluginPersistence>>>,
		command_results: &mut VecDeque<CommandResult>,
		start_time: &mut Option<Instant>,
		o: &mut impl NitroOutput,
	) -> anyhow::Result<bool> {
		match &mut self.inner {
			ExecutableHookHandleInner::NotStarted(command) => {
				let mut child = command.spawn().context("Failed to spawn command")?;

				let stdout = child.stdout.take().unwrap();
				let stdout = TryLineReader::new(stdout, READ_BUF_SIZE);

				let stdin = child.stdin.take().unwrap();

				*start_time = if std::env::var("NITRO_PLUGIN_PROFILE").is_ok_and(|x| x == "1") {
					Some(Instant::now())
				} else {
					None
				};

				self.inner = ExecutableHookHandleInner::Started {
					child,
					stdout,
					stdin,
					result: None,
				};

				Ok(false)
			}
			ExecutableHookHandleInner::Started {
				stdout,
				stdin,
				result,
				..
			} => {
				let lines = stdout.lines().await?;
				// EoF
				let Some(lines) = lines else {
					return Ok(true);
				};

				let persistence = plugin_persistence
					.as_mut()
					.context("Hook handle does not have a reference to persistent plugin data")?;
				let mut persistence_lock = persistence.lock().await;

				// Send command results from the worker to this hook
				if let Some(worker) = &mut persistence_lock.worker {
					while let Some(result) = worker.pop_command_result() {
						let action = InputAction::CommandResult(result)
							.serialize(self.protocol_version)
							.context("Failed to serialize input action")?;
						stdin
							.write_all(action.as_bytes())
							.await
							.context("Failed to write input action to plugin")?;
						stdin
							.write_all(b"\n")
							.await
							.context("Failed to write input action delimiter to plugin")?;
					}
				}

				let mut inputs_to_send = Vec::new();

				for line in lines {
					let action =
						OutputAction::deserialize(&line, self.use_base64, self.protocol_version)
							.context("Failed to deserialize plugin action")?;

					let Some(action) = action else {
						if let Some(message) = line.strip_prefix("$_") {
							println!("{message}");
						}
						continue;
					};

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

							// We can stop polling early
							return Ok(true);
						}
						OutputAction::SetError(error) => {
							return Err(anyhow!(
								"Plugin '{}' returned an error: {error}",
								self.plugin_id
							));
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
								.await
								.context("Failed to send command to worker")?;
						}
						OutputAction::SetCommandResult(result) => {
							command_results.push_back(result);
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
						OutputAction::StartManualFilesPrompt(files) => {
							let result = o.prompt_special_manual_files(files).await;
							inputs_to_send.push(InputAction::PromptResult(result.is_ok()));
						}
					}
				}

				for input in inputs_to_send {
					self.send_input_action(input).await?;
				}

				Ok(false)
			}
		}
	}

	pub async fn kill(self) -> anyhow::Result<Option<H::Result>> {
		if let ExecutableHookHandleInner::Started {
			mut child, result, ..
		} = self.inner
		{
			child.kill().await?;

			Ok(result)
		} else {
			Ok(None)
		}
	}

	/// Gets the result from this hook. self.poll() must have already returned true for this to not throw an error.
	pub async fn result(self) -> anyhow::Result<H::Result> {
		let ExecutableHookHandleInner::Started {
			mut child, result, ..
		} = self.inner
		else {
			bail!("Result method called before executable hook was polled or started");
		};

		let cmd_result = child.wait().await?;

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

	/// Sends an action to the plugin
	pub async fn send_input_action(&mut self, action: InputAction) -> anyhow::Result<()> {
		if let ExecutableHookHandleInner::Started { stdin, .. } = &mut self.inner {
			let action = action
				.serialize(self.protocol_version)
				.context("Failed to serialize input action")?;

			stdin
				.write_all(action.as_bytes())
				.await
				.context("Failed to write input action to plugin")?;
			stdin
				.write_all(b"\n")
				.await
				.context("Failed to write input action delimiter to plugin")?;
		}

		Ok(())
	}
}
