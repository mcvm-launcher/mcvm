use std::borrow::Cow;
use std::io::Write;
use std::time::Duration;

use anyhow::{Context, bail};
use color_print::{cformat, cstr};
use crossterm::event::{KeyCode, KeyEvent};
use inquire::{Confirm, Password};
use itertools::Itertools;
use nitrolaunch::io::logging::Logger;
use nitrolaunch::io::paths::Paths;
use nitrolaunch::pkg_crate::{PkgRequest, PkgRequestSource};
use nitrolaunch::shared::io::config::IO_CONFIG;
use nitrolaunch::shared::lang::translate::{TranslationKey, TranslationMap};
use nitrolaunch::shared::manual_files::{self, ManualFile};
use nitrolaunch::shared::output::{Message, MessageContents, MessageLevel, NitroOutput};
use nitrolaunch::shared::util::print::ReplPrinter;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;

use crate::prompt::get_key;

/// A nice colored bullet point for terminal output
pub const HYPHEN_POINT: &str = cstr!("<k!> - </k!>");

/// A star icon
pub const STAR: &str = "\u{2605}";
/// A package icon
pub const PACKAGE: &str = "\u{1F4E6}";
/// An instance icon
pub const INSTANCE: &str = "\u{1F4C0}";
/// A version icon
pub const VERSION: &str = "\u{1F4C5}";
/// A loader icon
pub const LOADER: &str = "\u{1F4E5}";
/// A check icon
pub const CHECK: &str = "\u{2713}";

const SPINNER_INTERVAL_MS: u64 = 300;

/// Terminal NitroOutput
pub struct TerminalOutput {
	tx: UnboundedSender<Event>,
	rx: broadcast::Receiver<ResponseEvent>,
	/// Held to prevent the output task from being dropped before the process exits
	_output_task: Option<JoinHandle<()>>,
	level: MessageLevel,
	translation_map: Option<TranslationMap>,
}

#[async_trait::async_trait]
impl NitroOutput for TerminalOutput {
	fn display_text(&mut self, text: String, level: MessageLevel) {
		let _ = self.tx.send(Event::Print(text, level));
	}

	fn display_message(&mut self, message: Message) {
		let _ = self.tx.send(Event::Message(message));
	}

	fn start_process(&mut self) {
		let _ = self.tx.send(Event::StartProcess);
	}

	fn end_process(&mut self) {
		let _ = self.tx.send(Event::EndProcess);
	}

	fn start_section(&mut self) {
		let _ = self.tx.send(Event::StartSection);
	}

	fn end_section(&mut self) {
		let _ = self.tx.send(Event::EndSection);
	}

	async fn prompt_yes_no(
		&mut self,
		default: bool,
		message: MessageContents,
	) -> anyhow::Result<bool> {
		self.tx
			.send(Event::YesNo {
				message: message.clone(),
				default,
			})
			.map_err(|_| anyhow::anyhow!("Failed to send yes/no prompt event"))?;

		while let Ok(response) = self.rx.recv().await {
			if let ResponseEvent::YesNo(answer) = response {
				return Ok(answer);
			}
		}

		bail!("Failed to receive yes/no prompt response");
	}

	async fn prompt_password(&mut self, message: MessageContents) -> anyhow::Result<String> {
		self.prompt_a_password(message, false).await
	}

	async fn prompt_new_password(&mut self, message: MessageContents) -> anyhow::Result<String> {
		self.prompt_a_password(message, true).await
	}

	async fn prompt_special_manual_files(&mut self, files: Vec<ManualFile>) -> anyhow::Result<()> {
		self.tx
			.send(Event::ManualDownload(files.clone()))
			.map_err(|_| anyhow::anyhow!("Failed to send manual download prompt event"))?;

		while let Ok(response) = self.rx.recv().await {
			if let ResponseEvent::YesNo(success) = response {
				if success {
					return Ok(());
				} else {
					bail!("Cancelled");
				}
			}
		}

		bail!("Failed to receive manual download prompt response");
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
		let _ = nitrolaunch::shared::util::open_link(url);
		self.end_process();
		self.display(MessageContents::Property(
			"Open this link in your web browser if it has not opened already".into(),
			Box::new(MessageContents::Hyperlink(url.into())),
		));
		self.display(MessageContents::Property(
			"and enter the code".into(),
			Box::new(MessageContents::Copyable(code.into())),
		));
	}

	fn get_greater_copy(&self) -> Box<dyn NitroOutput + Sync> {
		Box::new(Self {
			tx: self.tx.clone(),
			rx: self.rx.resubscribe(),
			level: MessageLevel::Important,
			translation_map: None,
			_output_task: None,
		})
	}
}

impl TerminalOutput {
	pub fn new(paths: &Paths) -> anyhow::Result<Self> {
		let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
		let (response_tx, response_rx) = broadcast::channel(20);
		let output_task = OutputTask::new(rx, response_tx, paths)?;

		let task = tokio::spawn(output_task.run());

		Ok(Self {
			tx,
			rx: response_rx,
			level: MessageLevel::Important,
			translation_map: None,
			_output_task: Some(task),
		})
	}

	/// Set the log level of the output
	pub fn set_log_level(&mut self, level: MessageLevel) {
		self.level = level;
		let _ = self.tx.send(Event::SetLevel(level));
	}

	/// Set the translation map of the output
	pub fn set_translation_map(&mut self, map: TranslationMap) {
		self.translation_map = Some(map);
	}

	/// Shuts down the output task and waits until it is finished
	pub async fn shutdown(self) {
		std::mem::drop(self.tx);
		if let Some(task) = self._output_task {
			let _ = task.await;
		}
	}

	async fn prompt_a_password(
		&mut self,
		message: MessageContents,
		is_new: bool,
	) -> anyhow::Result<String> {
		self.tx
			.send(Event::Password {
				message: message.clone(),
				is_new,
			})
			.map_err(|_| anyhow::anyhow!("Failed to send password prompt event"))?;

		while let Ok(response) = self.rx.recv().await {
			if let ResponseEvent::Password { password } = response {
				return Ok(password);
			}
		}

		bail!("Failed to receive password prompt response");
	}
}

/// Format a PkgRequest with colors
fn disp_pkg_request_with_colors(req: PkgRequest) -> String {
	match req.source {
		PkgRequestSource::UserRequire => cformat!("<y>{req}"),
		PkgRequestSource::Bundled(..) => cformat!("<b>{req}"),
		PkgRequestSource::Refused(..) => cformat!("<r>{req}"),
		PkgRequestSource::Dependency(..) | PkgRequestSource::Repository => {
			cformat!("<c>{req}")
		}
	}
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

struct OutputTask {
	rx: UnboundedReceiver<Event>,
	tx: broadcast::Sender<ResponseEvent>,
	printer: ReplPrinter,
	level: MessageLevel,
	in_process: bool,
	loading_spinner_message: Option<MessageContents>,
	loading_spinner_stage: u8,
	indent_level: u8,
	logger: Logger,
	wrapping_enabled: bool,
}

impl OutputTask {
	fn new(
		rx: UnboundedReceiver<Event>,
		response_tx: broadcast::Sender<ResponseEvent>,
		paths: &Paths,
	) -> anyhow::Result<Self> {
		let mut logger = Logger::new(paths, "cli").context("Failed to create logger")?;

		// Log the command
		let args = std::env::args().join(" ");
		let _ = logger.log_message(MessageContents::Simple(args), MessageLevel::Important);

		Ok(Self {
			rx,
			tx: response_tx,
			printer: ReplPrinter::new(true),
			level: MessageLevel::Important,
			in_process: false,
			loading_spinner_message: None,
			loading_spinner_stage: 0,
			indent_level: 0,
			logger,
			wrapping_enabled: IO_CONFIG.get_bool("cli_wrap").unwrap_or(false),
		})
	}

	async fn run(mut self) {
		let mut spinner_timer = tokio::time::interval(Duration::from_millis(SPINNER_INTERVAL_MS));

		loop {
			tokio::select! {
				ev = self.rx.recv() => {
					let Some(ev) = ev else {
						break;
					};

					match ev {
						Event::Print(text, level) => {
							if level >= self.level {
								self.display_text_impl(&text);
							}
							let _ = self.logger.log_message(MessageContents::Simple(text), level);
						}
						Event::Message(message) => {
							self.display(message);
						}
						Event::StartProcess => self.start_process(),
						Event::EndProcess => self.end_process(),
						Event::StartSection => self.start_section(),
						Event::EndSection => self.end_section(),
						Event::YesNo { message, default } => {
							let ans = Confirm::new(&format_message(message))
								.with_default(default)
								.prompt();
							if let Ok(ans) = ans {
								let _ = self.tx.send(ResponseEvent::YesNo(ans));
							}
						}
						Event::Password { message, is_new } => {
							let ans = if is_new {
								Password::new(&format_message(message))
									.prompt()
							} else {
								Password::new(&format_message(message))
									.without_confirmation()
									.prompt()
							};
							if let Ok(ans) = ans {
								let _ = self.tx.send(ResponseEvent::Password { password: ans });
							}
						}
						Event::ManualDownload(files) => {
							self.display(Message {
								contents: MessageContents::Notice("Some files must be downloaded manually".into()),
								level: MessageLevel::Important,
							});
							self.display(Message {
								contents: "Download these files into your downloads folder".into(),
								level: MessageLevel::Important,
							});
							self.display(Message {
								contents: "Press [a] to open all links in your browser and [c] to exit".into(),
								level: MessageLevel::Important,
							});
							let mut remaining_files = files.iter().map(|x| x.filename.clone()).collect_vec();
							let scan_dir = manual_files::get_scan_dir_from_os(std::env::consts::OS);

							for file in &files {
								self.display(Message {
									contents: MessageContents::list_item(MessageContents::property(
										file.filename.clone(),
										MessageContents::Hyperlink(file.url.clone()),
									)),
									level: MessageLevel::Important,
								});
							}

							self.start_process();
							let mut success = true;
							let _ = crossterm::terminal::enable_raw_mode();
							while remaining_files.len() > 0 {
								let scanned_files = manual_files::scan(&scan_dir, &files);
								remaining_files.retain(|x| !scanned_files.contains(x));
								if remaining_files.is_empty() {
									break;
								}
								let remaining_str = if remaining_files.len() > 3 {
									remaining_files.iter().take(3).join(", ") + &format!(" and {} more", remaining_files.len() - 3)
								} else {
									remaining_files.iter().join(", ")
								};

								self.display(Message {
									contents: MessageContents::associated(
										MessageContents::Progress { current: scanned_files.len() as u32, total: files.len() as u32 },
										MessageContents::Simple(format!("Waiting for {} to be downloaded", remaining_str)),
									),
									level: MessageLevel::Important,
								});
								self.update_spinner();

								if let Ok(Some(KeyEvent { code, .. })) = get_key() {
									match code {
										KeyCode::Char('a') => {
											manual_files::open_all(&files);
										}
										KeyCode::Char('c') => {
											success = false;
											break;
										}
										_ => {}
									}
								}

								tokio::time::sleep(Duration::from_millis(SPINNER_INTERVAL_MS)).await;
							}

							if success {
								self.display(Message {
									contents: MessageContents::Success("All files downloaded".into()),
									level: MessageLevel::Important,
								});
							}
							self.end_process();
							let _ = crossterm::terminal::disable_raw_mode();

							let _ = self.tx.send(ResponseEvent::YesNo(success));
						}
						Event::SetLevel(level) => self.level = level,
					}
				}
				_ = spinner_timer.tick() => {
					self.update_spinner();
				}
			};
		}
	}

	fn start_process(&mut self) {
		self.end_process();
		self.in_process = true;
	}

	fn end_process(&mut self) {
		if self.in_process {
			self.printer.newline();
		}
		self.in_process = false;
		self.loading_spinner_message = None;
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

	fn display(&mut self, message: Message) {
		let _ = self
			.logger
			.log_message(message.contents.clone(), message.level);

		if message.level >= self.level {
			let is_error = matches!(&message.contents, MessageContents::Error(..));

			/*
				If the message is an error it will span multiple lines and break the ReplPrinter,
				plus the process is aborted anyway
			*/
			if is_error {
				self.end_process();
			}

			let is_success = matches!(message.contents, MessageContents::Success(..));
			if self.in_process {
				self.loading_spinner_message = Some(message.contents);
				if is_success {
					self.update_spinner();
				}
			} else {
				let message_contents = if is_success {
					format!(
						"{} {}",
						format_loading_spinner(4),
						format_message(message.contents)
					)
				} else {
					format_message(message.contents)
				};
				let message_contents = if !self.in_process && self.wrapping_enabled {
					wrap_message(&message_contents).to_string()
				} else {
					message_contents
				};
				self.display_text_impl(&message_contents);
			}
		}
	}

	fn display_text_impl(&mut self, text: &str) {
		if self.in_process {
			self.printer.print(text);
		} else {
			self.printer.print(text);
			self.printer.newline();
		}
	}

	fn update_spinner(&mut self) {
		let Some(message) = &self.loading_spinner_message else {
			return;
		};

		self.loading_spinner_stage += 1;
		if self.loading_spinner_stage > 3 {
			self.loading_spinner_stage = 0;
		}

		let spinner = if let MessageContents::Success(..) = &message {
			format_loading_spinner(4)
		} else {
			format_loading_spinner(self.loading_spinner_stage)
		};

		let message = format!("{spinner} {}", format_message(message.clone()));
		self.display_text_impl(&message);
	}
}

enum Event {
	Print(String, MessageLevel),
	Message(Message),
	StartProcess,
	EndProcess,
	StartSection,
	EndSection,
	YesNo {
		message: MessageContents,
		default: bool,
	},
	Password {
		message: MessageContents,
		is_new: bool,
	},
	ManualDownload(Vec<ManualFile>),
	SetLevel(MessageLevel),
}

#[derive(Clone)]
enum ResponseEvent {
	YesNo(bool),
	Password { password: String },
}

/// Formatting for messages
fn format_message(contents: MessageContents) -> String {
	match contents {
		MessageContents::Simple(text) => text,
		MessageContents::Notice(text) => {
			cformat!("<y>Notice: {}", text)
		}
		MessageContents::Warning(text) => cformat!("<y><s>Warning:</> {}", text),
		MessageContents::Error(text) => cformat!("<r><s,u>Error:</> {}", text),
		MessageContents::Success(text) => {
			cformat!("<g>{}", add_period(text))
		}
		MessageContents::Property(key, value) => {
			cformat!("<s>{}:</> {}", key, format_message(*value))
		}
		MessageContents::Header(text) => cformat!("<s>{}", text),
		MessageContents::StartProcess(text) => cformat!("{text}..."),
		MessageContents::Associated(item, message) => {
			// Don't parenthesize progress bars
			if let MessageContents::Progress { .. } | MessageContents::Package(..) = item.as_ref() {
				cformat!("{} {}", format_message(*item), format_message(*message))
			} else {
				cformat!("[{}] {}", format_message(*item), format_message(*message))
			}
		}
		MessageContents::Package(pkg, message) => {
			let pkg_disp = disp_pkg_request_with_colors(pkg);
			cformat!("[{}] {}", pkg_disp, format_message(*message))
		}
		MessageContents::Hyperlink(url) => cformat!("<m,u>{}", url),
		MessageContents::ListItem(item) => HYPHEN_POINT.to_string() + &format_message(*item),
		MessageContents::Copyable(text) => cformat!("<u>{}", text),
		MessageContents::Progress { current, total } => {
			let (full, empty) = progress_bar_parts(
				current,
				total,
				ProgressBarSettings {
					len: 25,
					full: "■",
					empty: "□",
					end: "⬢",
				},
			);
			cformat!("<s>[</><g>{}</g><k!>{}</><s>]</>", full, empty)
		}
		contents => contents.default_format(),
	}
}

/// Formats the loading spinner with a stage from 0-3, or 4 for a checkmark
fn format_loading_spinner(stage: u8) -> String {
	let icon = match stage {
		0 => "⡈",
		1 => "⠔",
		2 => "⠢",
		3 => "⢁",
		4 => &cformat!("<g>✓"),
		_ => ".",
	};

	cformat!("<s>[</><y>{icon}</><s>]</>")
}

/// Function for outputting an instance stdout line formatted
pub fn instance_stdout_line(line: &str) {
	let line = format_instance_stdout_line(line);

	let _ = writeln!(std::io::stdout(), "{line}");
}

/// Formats an output line from an instance
pub fn format_instance_stdout_line(line: &str) -> Cow<'_, str> {
	if line.starts_with('\u{001b}') {
		Cow::Borrowed(line)
	} else {
		// Category colors
		let line = line.replacen("/INFO", &cformat!("/<k!,s>INFO"), 1);
		let line = line.replacen("/WARN", &cformat!("/<y,s>WARN"), 1);
		let line = line.replacen("/ERROR", &cformat!("/<r,s>ERROR"), 1);

		// Timestamp
		let line = if line.starts_with('[') {
			if let Some(end) = line.find(']') {
				cformat!("<k!>{}</>{}", &line[0..end + 1], &line[end + 1..])
			} else {
				line
			}
		} else {
			line
		};

		Cow::Owned(line)
	}
}

/// Wraps a message based on the terminal width
fn wrap_message(message: &'_ str) -> Cow<'_, str> {
	let Ok((width, ..)) = crossterm::terminal::size() else {
		return Cow::Borrowed(message);
	};

	wrap_message_width(message, width as usize)
}

/// Wraps a message to a max size
fn wrap_message_width(message: &'_ str, width: usize) -> Cow<'_, str> {
	if width == 0 {
		return Cow::Borrowed(message);
	}

	let char_len = message.char_indices().count();
	let wrap_count = char_len / width;

	let mut out = String::with_capacity(message.len() + wrap_count);
	// +1 is to ensure we get the extra text at the end that is not wrapped
	for i in 0..(wrap_count + 1) {
		let start_char = i * width;
		// Bound the end to the end of the message
		let char_count = if start_char + width > char_len {
			char_len - start_char
		} else {
			width
		};
		if char_count == 0 {
			continue;
		}

		let mut chars = message.char_indices();
		let start = chars.nth(start_char).expect("Should be in bounds").0;
		let (end_start, end_char) = chars.nth(char_count - 2).expect("Should be in bounds");
		let end = end_start + end_char.len_utf8();

		out.push_str(&message[start..end]);
		// Prevent trailing newlines
		if end != message.len() {
			out.push('\n');
		}
	}

	Cow::Owned(out)
}

/// Cuts or pads a message to exactly `width` characters
pub fn fit_message_width(message: &str, width: usize) -> Cow<'_, str> {
	if width == 0 {
		return Cow::Borrowed("");
	}

	let mut char_count = 0;
	let mut end_byte = message.len();

	// Find where to cut (if needed) and count chars
	for (i, _) in message.char_indices() {
		if char_count == width {
			end_byte = i;
			break;
		}
		char_count += 1;
	}

	// Truncate if string is longer
	if char_count == width && end_byte < message.len() {
		return Cow::Owned(message[..end_byte].to_string());
	}

	// Return if correct width
	if char_count == width {
		return Cow::Borrowed(message);
	}

	// Pad if string is shorter
	let mut out = String::with_capacity(message.len() + (width - char_count));
	out.push_str(message);

	for _ in 0..(width - char_count) {
		out.push(' ');
	}

	Cow::Owned(out)
}

/// Get whether icons are enabled
pub fn icons_enabled() -> bool {
	IO_CONFIG.get_bool("cli_icons").unwrap_or_default()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_wrap_empty() {
		assert_eq!(wrap_message_width("", 5), "");
	}

	#[test]
	fn test_wrap_zero_width() {
		assert_eq!(wrap_message_width("foo", 0), "foo");
	}

	#[test]
	fn test_wrap_equal_width() {
		assert_eq!(wrap_message_width("foo", 3), "foo");
	}

	#[test]
	fn test_wrap_multiple_of_width() {
		assert_eq!(wrap_message_width("foobar", 3), "foo\nbar");
	}

	#[test]
	fn test_wrap_standard_value() {
		assert_eq!(wrap_message_width("foobarba", 3), "foo\nbar\nba");
	}

	#[test]
	fn test_wrap_inside_codepoint() {
		assert_eq!(wrap_message_width("fo⬢bar", 3), "fo⬢\nbar");
	}
}
