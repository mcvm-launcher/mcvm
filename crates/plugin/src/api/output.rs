use mcvm_shared::output::{MCVMOutput, Message, MessageLevel};

use crate::{input_output::OutputAction, plugin::NEWEST_PROTOCOL_VERSION};

/// Struct that implements the MCVMOutput trait for printing serialized messages
/// to stdout for the plugin runner to read
pub struct PluginOutput {
	use_base64: bool,
	protocol_version: u16,
}

impl PluginOutput {
	/// Create a new PluginOutput
	pub fn new(use_base64: bool, protocol_version: u16) -> Self {
		Self {
			use_base64,
			protocol_version,
		}
	}
}

impl Default for PluginOutput {
	fn default() -> Self {
		Self::new(true, NEWEST_PROTOCOL_VERSION)
	}
}

impl MCVMOutput for PluginOutput {
	fn display_text(&mut self, text: String, level: MessageLevel) {
		let action = OutputAction::Text(text, level);
		if let Ok(text) = action.serialize(self.use_base64, self.protocol_version) {
			println!("{text}");
		}
	}

	fn display_message(&mut self, message: Message) {
		let action = OutputAction::Message(message);
		if let Ok(text) = action.serialize(self.use_base64, self.protocol_version) {
			println!("{text}");
		}
	}

	fn start_process(&mut self) {
		let action = OutputAction::StartProcess;
		if let Ok(text) = action.serialize(self.use_base64, self.protocol_version) {
			println!("{text}");
		}
	}

	fn end_process(&mut self) {
		let action = OutputAction::EndProcess;
		if let Ok(text) = action.serialize(self.use_base64, self.protocol_version) {
			println!("{text}");
		}
	}

	fn start_section(&mut self) {
		let action = OutputAction::StartSection;
		if let Ok(text) = action.serialize(self.use_base64, self.protocol_version) {
			println!("{text}");
		}
	}

	fn end_section(&mut self) {
		let action = OutputAction::EndSection;
		if let Ok(text) = action.serialize(self.use_base64, self.protocol_version) {
			println!("{text}");
		}
	}
}
