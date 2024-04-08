use anyhow::Context;
use base64::prelude::*;
use mcvm_shared::output::{Message, MessageLevel};
use serde::{Deserialize, Serialize};

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
	SetResult(String),
}

impl OutputAction {
	/// Serialize the action to be sent to the plugin runner
	pub fn serialize(&self) -> anyhow::Result<String> {
		let json = serde_json::to_string(&self).context("Failed to serialize output action")?;
		// We have to base64 encode it to prevent newlines from messing up the output format
		let base64 = BASE64_STANDARD.encode(json);
		Ok(base64)
	}

	/// Deserialize an action sent from the plugin
	pub fn deserialize(action: &str) -> anyhow::Result<Self> {
		let json = BASE64_STANDARD
			.decode(action)
			.context("Failed to decode action base64")?;
		let action =
			serde_json::from_slice(&json).context("Failed to deserialize output action")?;
		Ok(action)
	}
}
