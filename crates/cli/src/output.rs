use std::collections::HashMap;
use std::io::Write;
use std::{fs::File, path::PathBuf};

use anyhow::Context;
use color_print::{cformat, cstr};
use inquire::{Confirm, Password};
use mcvm::io::files::paths::Paths;
use mcvm::pkg_crate::{PkgRequest, PkgRequestSource};
use mcvm::shared::lang::translate::TranslationKey;
use mcvm::shared::output::{
	default_special_ms_auth, MCVMOutput, Message, MessageContents, MessageLevel,
};
use mcvm::shared::util::print::ReplPrinter;
use mcvm::shared::util::utc_timestamp;

/// A nice colored bullet point for terminal output
pub const HYPHEN_POINT: &str = cstr!("<k!> - </k!>");

/// Terminal MCVMOutput
pub struct TerminalOutput {
	printer: ReplPrinter,
	level: MessageLevel,
	in_process: bool,
	indent_level: u8,
	log_file: File,
	latest_log_file: File,
	translation_map: Option<HashMap<TranslationKey, String>>,
}

impl MCVMOutput for TerminalOutput {
	fn display_text(&mut self, text: String, level: MessageLevel) {
		let _ = self.log_message(&text, level);
		self.display_text_impl(text, level);
	}

	fn display_message(&mut self, message: Message) {
		let _ = self.log_message(
			&Self::format_message_log(message.contents.clone()),
			message.level,
		);
		self.display_text_impl(Self::format_message(message.contents), message.level);
	}

	fn start_process(&mut self) {
		if self.in_process {
			self.printer.newline();
		} else {
			self.in_process = true;
		}
	}

	fn end_process(&mut self) {
		if self.in_process {
			self.printer.newline();
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

	fn prompt_password(&mut self, message: MessageContents) -> anyhow::Result<String> {
		let ans = Password::new(&Self::format_message(message))
			.without_confirmation()
			.prompt()
			.context("Inquire prompt failed")?;

		Ok(ans)
	}

	fn prompt_new_password(&mut self, message: MessageContents) -> anyhow::Result<String> {
		let ans = Password::new(&Self::format_message(message))
			.prompt()
			.context("Inquire prompt failed")?;

		Ok(ans)
	}

	fn translate(&self, key: TranslationKey) -> &str {
		if let Some(map) = &self.translation_map {
			map.get(&key)
				.map(|x| x.as_str())
				.unwrap_or(key.get_default())
		} else {
			key.get_default()
		}
	}

	fn display_special_ms_auth(&mut self, url: &str, code: &str) {
		let _ = mcvm::shared::util::open_link(url);
		default_special_ms_auth(self, url, code);
	}
}

impl TerminalOutput {
	pub fn new(paths: &Paths) -> anyhow::Result<Self> {
		let path = get_log_file_path(paths).context("Failed to get log file path")?;
		let file = File::create(path).context("Failed to open log file")?;
		let latest_file = File::create(get_latest_log_file_path(paths))
			.context("Failed to open latest.txt log file")?;
		Ok(Self {
			printer: ReplPrinter::new(true),
			level: MessageLevel::Important,
			in_process: false,
			indent_level: 0,
			log_file: file,
			latest_log_file: latest_file,
			translation_map: None,
		})
	}

	/// Display text
	fn display_text_impl(&mut self, text: String, level: MessageLevel) {
		if !level.at_least(&self.level) {
			return;
		}

		if self.in_process {
			self.printer.print(&text);
		} else {
			self.printer.print(&text);
			self.printer.newline();
		}
	}

	/// Formatting for messages
	fn format_message(contents: MessageContents) -> String {
		match contents {
			MessageContents::Simple(text) => text,
			MessageContents::Notice(text) => cformat!("<y>Notice: {}", text),
			MessageContents::Warning(text) => cformat!("<y><s>Warning:</> {}", text),
			MessageContents::Error(text) => cformat!("<r><s,u>Error:</> {}", text),
			MessageContents::Success(text) => cformat!("<g>{}", add_period(text)),
			MessageContents::Property(key, value) => {
				cformat!("<s>{}:</> {}", key, Self::format_message(*value))
			}
			MessageContents::Header(text) => cformat!("<s>{}", text),
			MessageContents::StartProcess(text) => cformat!("{text}..."),
			MessageContents::Associated(item, message) => {
				// Don't parenthesize progress bars
				if let MessageContents::Progress { .. } | MessageContents::Package(..) =
					item.as_ref()
				{
					cformat!(
						"{} {}",
						Self::format_message(*item),
						Self::format_message(*message)
					)
				} else {
					cformat!(
						"[{}] {}",
						Self::format_message(*item),
						Self::format_message(*message)
					)
				}
			}
			MessageContents::Package(pkg, message) => {
				let pkg_disp = disp_pkg_request_with_colors(pkg);
				cformat!("[{}] {}", pkg_disp, Self::format_message(*message))
			}
			MessageContents::Hyperlink(url) => cformat!("<m,u>{}", url),
			MessageContents::ListItem(item) => {
				HYPHEN_POINT.to_string() + &Self::format_message(*item)
			}
			MessageContents::Copyable(text) => cformat!("<u>{}", text),
			MessageContents::Progress { current, total } => {
				let (full, empty) = progress_bar_parts(
					current,
					total,
					ProgressBarSettings {
						len: 25,
						full: "=",
						empty: "-",
						end: ">",
					},
				);
				cformat!("<s>[</><g>{}</g><k!>{}</><s>]</>", full, empty)
			}
			contents => contents.default_format(),
		}
	}

	/// Formatting for messages in the log file
	fn format_message_log(contents: MessageContents) -> String {
		match contents {
			MessageContents::Simple(text) => text,
			MessageContents::Notice(text) => format!("[NOTICE] {}", text),
			MessageContents::Warning(text) => format!("[WARN] {}", text),
			MessageContents::Error(text) => format!("[ERR] {}", text),
			MessageContents::Success(text) => format!("[SUCCESS] {}", add_period(text)),
			MessageContents::Property(key, value) => {
				format!("{}: {}", key, Self::format_message_log(*value))
			}
			MessageContents::Header(text) => format!("### {} ###", text),
			MessageContents::StartProcess(text) => format!("{text}..."),
			MessageContents::Associated(item, message) => {
				format!(
					"({}) {}",
					Self::format_message_log(*item),
					Self::format_message_log(*message)
				)
			}
			MessageContents::Package(pkg, message) => {
				let pkg_disp = pkg.debug_sources();
				format!("[{}] {}", pkg_disp, Self::format_message_log(*message))
			}
			MessageContents::Hyperlink(url) => url,
			MessageContents::ListItem(item) => " - ".to_string() + &Self::format_message_log(*item),
			MessageContents::Copyable(text) => text,
			MessageContents::Progress { current, total } => format!("{current}/{total}"),
			contents => contents.default_format(),
		}
	}

	/// Log a message to the log file
	pub fn log_message(&mut self, text: &str, level: MessageLevel) -> anyhow::Result<()> {
		let level_indicator = match level {
			MessageLevel::Important => "I",
			MessageLevel::Extra => "E",
			MessageLevel::Debug => "D",
			MessageLevel::Trace => "T",
		};
		writeln!(self.log_file, "[{level_indicator}] {text}")?;
		writeln!(self.latest_log_file, "[{level_indicator}] {text}")?;

		Ok(())
	}

	/// Set the log level of the output
	pub fn set_log_level(&mut self, level: MessageLevel) {
		self.level = level;
	}

	/// Set the translation map of the output
	pub fn set_translation_map(&mut self, map: HashMap<TranslationKey, String>) {
		self.translation_map = Some(map);
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

/// Get the path to the latest log file
fn get_latest_log_file_path(paths: &Paths) -> PathBuf {
	paths.logs.join("latest.txt")
}

/// Settings for progress bar formatting
struct ProgressBarSettings {
	/// The length of the bar
	len: u8,
	/// The string to use for full
	full: &'static str,
	/// The string to use for empty
	empty: &'static str,
	/// The character to use for the end of the filled section of the bar
	end: &'static str,
}

/// Creates a nice looking progress bar and returns the full and empty parts
fn progress_bar_parts(current: u32, total: u32, settings: ProgressBarSettings) -> (String, String) {
	let progress = (current as f32) / (total as f32);
	let full_count = (progress * (settings.len as f32)) as u8;
	let empty_count = settings.len - full_count;
	let mut full_bar = settings.full.repeat(full_count.into());
	if full_count > 0 {
		full_bar.replace_range(
			full_bar.len() - settings.end.len()..full_bar.len(),
			settings.end,
		);
	}
	let empty_bar = settings.empty.repeat(empty_count.into());
	(full_bar, empty_bar)
}

/// Adds a period to the end of a string if it isn't punctuated already
fn add_period(string: String) -> String {
	if string.ends_with(['.', ',', ';', ':', '!', '?']) {
		string
	} else {
		string + "."
	}
}
