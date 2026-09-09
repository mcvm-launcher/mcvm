use std::{
	io::{Stdin, Stdout, Write},
	time::Duration,
};

use anyhow::Context;
use nitro_shared::{
	manual_files::ManualFile,
	output::{Message, MessageLevel, NitroOutput},
};

use crate::{
	input_output::{InputAction, OutputAction},
	plugin::NEWEST_PROTOCOL_VERSION,
};

/// Struct that implements the NitroOutput trait for printing serialized messages
/// to stdout for the plugin runner to read
pub struct ExecutablePluginOutput {
	use_base64: bool,
	protocol_version: u16,
	stdout: Stdout,
}

impl ExecutablePluginOutput {
	/// Create a new ExecutablePluginOutput
	pub fn new(use_base64: bool, protocol_version: u16) -> Self {
		Self {
			use_base64,
			protocol_version,
			stdout: std::io::stdout(),
		}
	}

	fn send_action(&mut self, action: OutputAction) {
		if let Ok(text) = action.serialize(self.use_base64, self.protocol_version) {
			let _ = writeln!(&mut self.stdout, "{text}");
			let _ = self.stdout.flush();
		}
	}
}

impl Default for ExecutablePluginOutput {
	fn default() -> Self {
		Self::new(true, NEWEST_PROTOCOL_VERSION)
	}
}

#[async_trait::async_trait]
impl NitroOutput for ExecutablePluginOutput {
	fn display_text(&mut self, text: String, level: MessageLevel) {
		self.send_action(OutputAction::Text(text, level));
	}

	fn display_message(&mut self, message: Message) {
		self.send_action(OutputAction::Message(message));
	}

	fn start_process(&mut self) {
		self.send_action(OutputAction::StartProcess);
	}

	fn end_process(&mut self) {
		self.send_action(OutputAction::EndProcess);
	}

	fn start_section(&mut self) {
		self.send_action(OutputAction::StartSection);
	}

	fn end_section(&mut self) {
		self.send_action(OutputAction::EndSection);
	}

	async fn prompt_special_manual_files(&mut self, files: Vec<ManualFile>) -> anyhow::Result<()> {
		self.send_action(OutputAction::StartManualFilesPrompt(files));

		let stdin = std::io::stdin();

		loop {
			if let Some(InputAction::PromptResult(success)) =
				poll_input_action(&stdin, self.protocol_version)?
			{
				if success {
					return Ok(());
				} else {
					return Err(anyhow::anyhow!("Manual files prompt failed"));
				}
			}

			tokio::time::sleep(Duration::from_millis(150)).await;
		}
	}
}

pub(crate) fn poll_input_action(
	stdin: &Stdin,
	protocol_version: u16,
) -> anyhow::Result<Option<InputAction>> {
	let mut buf = String::new();
	let result_len = stdin
		.read_line(&mut buf)
		.context("Failed to read from stdin")?;
	if result_len == 0 {
		return Ok(None);
	}
	let line = buf.trim_end_matches("\r\n").trim_end_matches('\n');

	let action = InputAction::deserialize(line, protocol_version)
		.context("Failed to deserialize input action")?;

	Ok(Some(action))
}
