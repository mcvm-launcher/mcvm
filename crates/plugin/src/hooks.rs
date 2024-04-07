use std::process::Command;

use anyhow::{bail, Context};
use serde::{de::DeserializeOwned, Serialize};

/// Trait for a hook that can be called
pub trait Hook {
	/// The type for the argument that goes into the hook
	type Arg: Serialize + DeserializeOwned;
	/// The type for the result from the hook
	type Result: DeserializeOwned + Serialize;

	/// Get the name of the hook
	fn get_name(&self) -> &'static str {
		Self::get_name_static()
	}

	/// Get the name of the hook statically
	fn get_name_static() -> &'static str;

	/// Call the hook using the specified program
	fn call(
		&self,
		cmd: &str,
		arg: &Self::Arg,
		custom_config: Option<String>,
	) -> anyhow::Result<Self::Result> {
		let arg = serde_json::to_string(arg).context("Failed to serialize hook argument")?;
		let mut cmd = Command::new(cmd);
		cmd.arg(self.get_name());
		cmd.arg(arg);
		if let Some(custom_config) = custom_config {
			cmd.env("MCVM_CUSTOM_CONFIG", custom_config);
		}
		cmd.stdout(std::process::Stdio::piped());

		let result = cmd
			.spawn()?
			.wait_with_output()
			.context("Failed to wait for hook child process")?;

		if !result.status.success() {
			if let Some(exit_code) = result.status.code() {
				bail!("Hook returned a non-zero exit code of {}", exit_code);
			} else {
				bail!("Hook returned a non-zero exit code");
			}
		}

		let result =
			serde_json::from_slice(&result.stdout).context("Failed to deserialize hook result")?;

		Ok(result)
	}
}

macro_rules! def_hook {
	($struct:ident, $name:literal, $desc:literal, $arg:ty, $res:ty) => {
		#[doc = $desc]
		pub struct $struct;

		impl Hook for $struct {
			type Arg = $arg;
			type Result = $res;

			fn get_name_static() -> &'static str {
				$name
			}
		}
	};
}

def_hook!(
	OnLoad,
	"on_load",
	"Hook for when a plugin is loaded",
	(),
	()
);
