use std::{
	io::{BufRead, BufReader},
	process::Command,
};

use anyhow::{bail, Context};
use mcvm_shared::output::MCVMOutput;
use serde::{de::DeserializeOwned, Serialize};

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
		custom_config: Option<String>,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Self::Result> {
		let arg = serde_json::to_string(arg).context("Failed to serialize hook argument")?;
		let mut cmd = Command::new(cmd);
		cmd.arg(self.get_name());
		cmd.arg(arg);
		if let Some(custom_config) = custom_config {
			cmd.env("MCVM_CUSTOM_CONFIG", custom_config);
		}

		if Self::get_takes_over() {
			cmd.spawn()?.wait()?;

			Ok(Self::Result::default())
		} else {
			cmd.stdout(std::process::Stdio::piped());

			let mut child = cmd.spawn()?;

			let stdout = child.stdout.as_mut().unwrap();
			let stdout_reader = BufReader::new(stdout);
			let stdout_lines = stdout_reader.lines();

			let mut result = None;
			for line in stdout_lines {
				let line = line?;
				let action = OutputAction::deserialize(&line)
					.context("Failed to deserialize plugin action")?;
				match action {
					OutputAction::SetResult(new_result) => {
						result = Some(
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
