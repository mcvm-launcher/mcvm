use std::{
	io::{BufRead, BufReader, Lines},
	path::Path,
	process::{Child, ChildStdout, Command},
};

use anyhow::{bail, Context};
use mcvm_core::{net::game_files::version_manifest::VersionEntry, Paths};
use mcvm_shared::{output::MCVMOutput, versions::VersionInfo, Side};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::output::OutputAction;

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
	fn call(
		&self,
		cmd: &str,
		arg: &Self::Arg,
		additional_args: &[String],
		working_dir: Option<&Path>,
		custom_config: Option<String>,
		paths: &Paths,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<HookHandle<Self>>
	where
		Self: Sized,
	{
		let _ = o;
		let arg = serde_json::to_string(arg).context("Failed to serialize hook argument")?;
		let mut cmd = Command::new(cmd);
		cmd.args(additional_args);
		cmd.arg(self.get_name());
		cmd.arg(arg);

		// Set up environment
		if let Some(custom_config) = custom_config {
			cmd.env("MCVM_CUSTOM_CONFIG", custom_config);
		}
		cmd.env("MCVM_DATA_DIR", &paths.data);
		cmd.env("MCVM_CONFIG_DIR", &paths.project.config_dir());
		if let Some(working_dir) = working_dir {
			cmd.current_dir(working_dir);
		}

		if Self::get_takes_over() {
			cmd.spawn()?.wait()?;

			Ok(HookHandle::constant(Self::Result::default()))
		} else {
			cmd.stdout(std::process::Stdio::piped());

			let mut child = cmd.spawn()?;

			let stdout = child.stdout.take().unwrap();
			let stdout_reader = BufReader::new(stdout);
			let stdout_lines = stdout_reader.lines();

			let handle = HookHandle {
				inner: HookHandleInner::Process(child, stdout_lines, None),
			};

			Ok(handle)
		}
	}
}

/// Handle returned by running a hook. Make sure to await it if you need to.
#[must_use]
pub struct HookHandle<H: Hook> {
	inner: HookHandleInner<H>,
}

impl<H: Hook> HookHandle<H> {
	/// Create a new constant handle
	pub fn constant(result: H::Result) -> Self {
		Self {
			inner: HookHandleInner::Constant(result),
		}
	}

	/// Poll the handle, returning true if the handle is ready
	pub fn poll(&mut self, o: &mut impl MCVMOutput) -> anyhow::Result<bool> {
		match &mut self.inner {
			HookHandleInner::Process(_, lines, result) => {
				// TODO: Make this actually poll instead of just reading all the lines
				for line in lines {
					let line = line?;
					let action = OutputAction::deserialize(&line)
						.context("Failed to deserialize plugin action")?;
					match action {
						OutputAction::SetResult(new_result) => {
							*result = Some(
								serde_json::from_str(&new_result)
									.context("Failed to deserialize hook result")?,
							);
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
				}

				Ok(true)
			}
			HookHandleInner::Constant(..) => Ok(true),
		}
	}

	/// Get the result of the hook by waiting for it
	pub fn result(mut self, o: &mut impl MCVMOutput) -> anyhow::Result<H::Result> {
		if let HookHandleInner::Process(..) = &self.inner {
			loop {
				let result = self.poll(o)?;
				if result {
					break;
				}
			}
		}

		match self.inner {
			HookHandleInner::Constant(result) => Ok(result),
			HookHandleInner::Process(mut child, _, result) => {
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
			HookHandleInner::Process(mut child, _, result) => {
				child.kill()?;

				Ok(result)
			}
		}
	}
}

/// The inner value for a HookHandle
enum HookHandleInner<H: Hook> {
	/// A process hook
	Process(Child, Lines<BufReader<ChildStdout>>, Option<H::Result>),
	/// A constant result hook
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
	WhileInstanceLaunch,
	"while_instance_launch",
	"Hook for running sibling processes with an instance when it is launched",
	WhileInstanceLaunchArg,
	(),
);

/// Argument for the WhileInstanceLaunch hook
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct WhileInstanceLaunchArg {
	/// The side of the instance
	pub side: Option<Side>,
	/// Path to the instance's game dir
	pub game_dir: String,
	/// Version info for the instance
	pub version_info: VersionInfo,
	/// Custom config on the instance
	pub custom_config: serde_json::Map<String, serde_json::Value>,
	/// The PID of the instance process
	pub pid: Option<u32>,
}
