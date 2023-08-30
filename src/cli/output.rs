use color_print::cformat;
use mcvm::util::print::ReplPrinter;
use mcvm_shared::output::{MCVMOutput, Message, MessageContents, MessageLevel};

/// Terminal MCVMOutput
pub struct TerminalOutput {
	printer: ReplPrinter,
	pub level: MessageLevel,
	in_process: bool,
	indent_level: u8,
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
			MessageContents::Header(text) => {
				self.display_text(cformat!("<s>{}", text), message.level)
			}
			MessageContents::StartProcess(text) => {
				self.display_text(format!("<i>{text}..."), message.level)
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

	fn start_section(&mut self) {
		self.indent_level += 1;
		self.printer.indent(self.indent_level.into());
	}

	fn end_section(&mut self) {
		if self.indent_level != 0 {
			self.indent_level -= 1;
			self.printer.indent(self.indent_level.into());
		}
	}
}

impl TerminalOutput {
	pub fn new() -> Self {
		Self {
			printer: ReplPrinter::new(true),
			level: MessageLevel::Important,
			in_process: false,
			indent_level: 0,
		}
	}
}
