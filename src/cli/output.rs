use std::io::Write;
use std::{fs::File, path::PathBuf};

use anyhow::Context;
use color_print::cformat;
use inquire::Confirm;
use mcvm::{
	io::files::paths::Paths,
	util::{
		print::{ReplPrinter, HYPHEN_POINT},
		utc_timestamp,
	},
};
use mcvm_pkg::{PkgRequest, PkgRequestSource};
use mcvm_shared::output::{MCVMOutput, Message, MessageContents, MessageLevel, default_special_ms_auth};

/// Terminal MCVMOutput
pub struct TerminalOutput {
	printer: ReplPrinter,
	pub level: MessageLevel,
	in_process: bool,
	indent_level: u8,
	log_file: File,
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

		let _ = self.log_message(&text);
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

	fn prompt_yes_no(&mut self, default: bool, message: MessageContents) -> anyhow::Result<bool> {
		let ans = Confirm::new(&Self::format_message(message))
			.with_default(default)
			.prompt()
			.context("Inquire prompt failed")?;

		Ok(ans)
	}

	fn display_special_ms_auth(&mut self, url: &str, code: &str) {
		let _ = mcvm::util::open_link(url);
		default_special_ms_auth(self, url, code);
	}
}

impl TerminalOutput {
	pub fn new(paths: &Paths) -> anyhow::Result<Self> {
		let path = get_log_file_path(paths).context("Failed to get log file path")?;
		let file = File::create(path).context("Failed to open log file")?;
		Ok(Self {
			printer: ReplPrinter::new(true),
			level: MessageLevel::Important,
			in_process: false,
			indent_level: 0,
			log_file: file,
		})
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

	/// Log a message to the log file
	fn log_message(&mut self, text: &str) -> anyhow::Result<()> {
		writeln!(self.log_file, "{text}")?;

		Ok(())
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

/// Get the path to a log file
fn get_log_file_path(paths: &Paths) -> anyhow::Result<PathBuf> {
	Ok(paths.logs.join(format!("log-{}.txt", utc_timestamp()?)))
}
