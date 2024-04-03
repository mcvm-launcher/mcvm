use std::{ffi::OsStr, process::Command};

use anyhow::{bail, Context};
use serde::{de::DeserializeOwned, Serialize};

/// Trait for a hook that can be called
pub trait Hook {
	/// The type for the argument that goes into the hook
	type Arg: Serialize + DeserializeOwned;
	/// The type for the result from the hook
	type Result: DeserializeOwned + Serialize;

	/// Get the name of the hook
	fn get_name(&self) -> &'static str;

	/// Call the hook using the specified program
	fn call(&self, cmd: &OsStr, arg: &Self::Arg) -> anyhow::Result<Self::Result> {
		let arg = serde_json::to_string(arg).context("Failed to serialize hook argument")?;
		let mut cmd = Command::new(cmd);
		cmd.stdout(std::process::Stdio::null());
		cmd.arg(self.get_name());
		cmd.arg(arg);

		let result = cmd
			.spawn()
			.context("Failed to spawn hook child")?
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
