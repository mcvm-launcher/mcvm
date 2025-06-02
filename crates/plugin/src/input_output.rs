use anyhow::Context;
use base64::prelude::*;
use mcvm_shared::output::{Message, MessageLevel};
use serde::{Deserialize, Serialize};

/// The delimiter which starts every output line after protocol version 2
pub static STARTING_DELIMITER: &str = "%_";

/// An action to be sent between the plugin and plugin runner
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputAction {
	/// Display raw text
	Text(String, MessageLevel),
	/// Display a message
	Message(Message),
	/// Start an output process
	StartProcess,
	/// Stop an output process
	EndProcess,
	/// Start an output section
	StartSection,
	/// End an output section
	EndSection,
	/// Set the result of the hook
	SetResult(serde_json::Value),
	/// Set the persistent state of the plugin
	SetState(serde_json::Value),
	/// Return a result from a command
	SetCommandResult(CommandResult),
	/// Run a command on the plugin's worker
	RunWorkerCommand {
		/// The command to run
		command: String,
		/// The argument/input to the command
		payload: serde_json::Value,
	},
}

impl OutputAction {
	/// Serialize the action to be sent to the plugin runner
	pub fn serialize(&self, use_base64: bool, protocol_version: u16) -> anyhow::Result<String> {
		let json = serde_json::to_string(&self).context("Failed to serialize output action")?;
		let out = if use_base64 {
			// We have to base64 encode it to prevent newlines from messing up the output format
			let base64 = BASE64_STANDARD.encode(json);
			base64
		} else {
			json
		};

		if protocol_version >= 2 {
			Ok(format!("{STARTING_DELIMITER}{out}"))
		} else {
			Ok(out)
		}
	}

	/// Deserialize an action sent from the plugin
	pub fn deserialize(
		action: &str,
		use_base64: bool,
		protocol_version: u16,
	) -> anyhow::Result<Option<Self>> {
		// Remove the starting delimiter
		let action = if protocol_version >= 2 {
			if action.starts_with(STARTING_DELIMITER) {
				&action[STARTING_DELIMITER.len()..]
			} else {
				return Ok(None);
			}
		} else {
			action
		};
		let json = if use_base64 {
			BASE64_STANDARD
				.decode(action)
				.context("Failed to decode action base64")?
		} else {
			action.bytes().collect()
		};
		let action =
			serde_json::from_slice(&json).context("Failed to deserialize output action")?;
		Ok(Some(action))
	}
}

/// An action to be sent to the plugin from the plugin runner
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputAction {
	/// Run a custom command
	Command {
		/// The command to run
		command: String,
		/// The argument/input to the command
		payload: serde_json::Value,
	},
	/// The result of a custom command
	CommandResult(CommandResult),
}

impl InputAction {
	/// Serialize the action to be sent to the plugin
	pub fn serialize(&self, protocol_version: u16) -> anyhow::Result<String> {
		let _ = protocol_version;
		serde_json::to_string(&self).context("Failed to serialize input action")
	}

	/// Deserialize an action sent from the plugin runner
	pub fn deserialize(action: &str, protocol_version: u16) -> anyhow::Result<Self> {
		let _ = protocol_version;
		serde_json::from_str(action).context("Failed to deserialize input action")
	}
}

/// The result of a custom command
#[derive(Serialize, Deserialize)]
pub struct CommandResult {
	/// The command that was run
	command: String,
	/// The result from the command
	result: serde_json::Value,
}
