use color_print::cformat;
use mcvm::util::print::ReplPrinter;
use mcvm_shared::output::{MCVMOutput, Message, MessageContents, MessageLevel};

/// Terminal MCVMOutput
pub struct TerminalOutput {
	printer: ReplPrinter,
	in_process: bool,
	pub level: MessageLevel,
}

impl MCVMOutput for TerminalOutput {
	fn display_text(&mut self, text: String, level: MessageLevel) {
		if !level.at_least(&self.level) {
			return;
		}

		if self.in_process {
			self.printer.print(&text);
		} else {
			self.printer.print(&text);
			self.printer.println("");
		}
	}

	fn display_message(&mut self, message: Message) {
		match message.contents {
			MessageContents::Simple(text) => self.display_text(text, message.level),
			MessageContents::Warning(text) => {
				self.display_text(cformat!("<y>Warning: {}", text), message.level)
			}
			MessageContents::Error(text) => {
				self.display_text(cformat!("<r>Error: {}", text), message.level)
			}
			MessageContents::Success(text) => {
				self.display_text(cformat!("<g>{}", text), message.level)
			}
			MessageContents::Property(key, value) => {
				self.display_text(cformat!("<s>{}:</> <b>{}", key, value), message.level)
			}
		}
	}

	fn start_process(&mut self) {
		if self.in_process {
			self.printer.println("");
		} else {
			self.in_process = true;
		}
	}

	fn end_process(&mut self) {
		self.in_process = false;
	}
}

impl TerminalOutput {
	pub fn new() -> Self {
		Self {
			printer: ReplPrinter::new(true),
			in_process: false,
			level: MessageLevel::Important,
		}
	}
}
