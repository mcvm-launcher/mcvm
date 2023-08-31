use color_print::cformat;
use mcvm::util::print::{ReplPrinter, HYPHEN_POINT};
use mcvm_pkg::{PkgRequest, PkgRequestSource};
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
		self.display_text(Self::format_message(message.contents), message.level);
	}

	fn start_process(&mut self) {
		if self.in_process {
			self.printer.println("");
		} else {
			self.in_process = true;
		}
	}

	fn end_process(&mut self) {
		if self.in_process {
			self.printer.println("");
		}
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

	/// Formatting for messages
	fn format_message(contents: MessageContents) -> String {
		match contents {
			MessageContents::Simple(text) => text,
			MessageContents::Notice(text) => cformat!("<y>Notice: {}", text),
			MessageContents::Warning(text) => cformat!("<y>Warning: {}", text),
			MessageContents::Error(text) => cformat!("<r>Error: {}", text),
			MessageContents::Success(text) => cformat!("<g>{}", text),
			MessageContents::Property(key, value) => {
				cformat!("<s>{}:</> {}", key, Self::format_message(*value))
			}
			MessageContents::Header(text) => cformat!("<s>{}", text),
			MessageContents::StartProcess(text) => cformat!("<i>{text}..."),
			MessageContents::Associated(item, message) => {
				cformat!("(<b!>{}</b!>) {}", item, Self::format_message(*message))
			}
			MessageContents::Package(pkg, message) => {
				let pkg_disp = disp_pkg_request_with_colors(pkg);
				cformat!("[{}] {}", pkg_disp, Self::format_message(*message))
			}
			MessageContents::Hyperlink(url) => cformat!("<m>{}", url),
			MessageContents::ListItem(item) => {
				HYPHEN_POINT.to_string() + &Self::format_message(*item)
			}
			MessageContents::Copyable(text) => cformat!("<u>{}", text),
		}
	}
}

/// Format a PkgRequest with colors
fn disp_pkg_request_with_colors(req: PkgRequest) -> String {
	match req.source {
		PkgRequestSource::UserRequire => cformat!("<y>{}", req.id),
		PkgRequestSource::Bundled(..) => cformat!("<b>{}", req.id),
		PkgRequestSource::Refused(..) => cformat!("<r>{}", req.id),
		PkgRequestSource::Dependency(..) | PkgRequestSource::Repository => {
			cformat!("<c>{}", req.id)
		}
	}
}
