use std::{
	io::Write,
	ops::{Deref, DerefMut},
};

use anyhow::bail;
use serde::{Deserialize, Serialize};

use crate::{
	lang::translate::TranslationKey,
	manual_files::{self, ManualFile},
	pkg::{PackageDiff, PkgRequest, ResolutionError},
	util::{OS_STRING, print::ReplPrinter},
};

/// Trait for a type that can output information about Nitrolaunch processes
#[async_trait::async_trait]
pub trait NitroOutput: Send {
	/// Base function for a simple message. Used as a fallback
	fn display_text(&mut self, text: String, level: MessageLevel);

	/// Function to display a message to the user
	fn display_message(&mut self, message: Message) {
		self.display_text(message.contents.default_format(), message.level);
	}

	/// Display a message at the important level
	fn display(&mut self, contents: MessageContents) {
		self.display_message(Message {
			contents,
			level: MessageLevel::Important,
		})
	}

	/// Display a message at the debug level
	fn debug(&mut self, contents: MessageContents) {
		self.display_message(Message {
			contents,
			level: MessageLevel::Debug,
		})
	}

	/// Display a message at the trace level
	fn trace(&mut self, contents: MessageContents) {
		self.display_message(Message {
			contents,
			level: MessageLevel::Trace,
		})
	}

	/// Start a process of multiple messages. Implementations can use this to replace a line
	/// multiple times
	fn start_process(&mut self) {}

	/// End an existing process
	fn end_process(&mut self) {}

	/// Start a new section / level of hierarchy. Implementations can use this to set the indent level
	fn start_section(&mut self) {}

	/// End the current section and go down a level of hierarchy
	fn end_section(&mut self) {}

	/// Gets an OutputProcess object for this output
	fn get_process(&'_ mut self) -> OutputProcess<'_, Self>
	where
		Self: Sized,
	{
		OutputProcess::new(self)
	}

	/// Gets an OutputSection object for this output
	fn get_section(&'_ mut self) -> OutputSection<'_, Self>
	where
		Self: Sized,
	{
		OutputSection::new(self)
	}

	/// Offer a confirmation / yes no prompt to the user.
	/// The default is the default value of the prompt.
	async fn prompt_yes_no(
		&mut self,
		default: bool,
		message: MessageContents,
	) -> anyhow::Result<bool> {
		let _message = message;
		Ok(default)
	}

	/// Offer a password / secret prompt
	async fn prompt_password(&mut self, message: MessageContents) -> anyhow::Result<String> {
		let _ = message;
		bail!("No password prompt available")
	}

	/// Offer a new password / secret prompt
	async fn prompt_new_password(&mut self, message: MessageContents) -> anyhow::Result<String> {
		self.prompt_password(message).await
	}

	/// Get the translation for the specified key
	fn translate(&self, key: TranslationKey) -> &str {
		key.get_default()
	}

	// Specialized implementations for certain combinations of messages.
	// These all have default impls which you can override for more specific behavior

	/// Specialized implementation for details given to the user for Microsoft authentication
	fn display_special_ms_auth(&mut self, url: &str, code: &str) {
		default_special_ms_auth(self, url, code);
	}

	/// Specialized implementation for displaying a package resolution error to the user
	fn display_special_resolution_error(&mut self, error: ResolutionError, instance_id: &str) {
		self.display(MessageContents::Error(format!(
			"Failed to resolve packages for instance {instance_id}: {error}"
		)))
	}

	/// Specialized implementation for prompting an account passkey
	async fn prompt_special_account_passkey(
		&mut self,
		message: MessageContents,
		account_id: &str,
	) -> anyhow::Result<String> {
		let _ = account_id;
		self.prompt_password(message).await
	}

	/// Specialized implementation for showing changes to installed packages and asking if the user
	/// wants to proceed
	async fn prompt_special_package_diffs(
		&mut self,
		diffs: Vec<PackageDiff>,
	) -> anyhow::Result<bool> {
		self.display(MessageContents::Header("Package changes:".into()));

		for diff in diffs {
			match &diff {
				PackageDiff::Added(pkg)
				| PackageDiff::Removed(pkg)
				| PackageDiff::VersionChanged(pkg, ..) => {
					let pkg = PkgRequest::clone(pkg);

					let message = match &diff {
						PackageDiff::Added(..) => "Added".to_string(),
						PackageDiff::Removed(..) => "Removed".to_string(),
						PackageDiff::VersionChanged(_, old_version, new_version) => {
							format!("{old_version} -> {new_version}")
						}
						_ => unreachable!(),
					};
					self.display(MessageContents::Package(
						pkg,
						Box::new(MessageContents::Simple(message)),
					));
				}
				PackageDiff::ManyAdded(count) => {
					self.display(MessageContents::Simple(format!("Added {count} packages")))
				}
				PackageDiff::ManyRemoved(count) => {
					self.display(MessageContents::Simple(format!("Removed {count} packages")))
				}
			}
		}

		self.prompt_yes_no(
			false,
			MessageContents::Simple("Would you like to proceed with these changes?".into()),
		)
		.await
	}

	/// Prompts the user to install manually downloaded files.
	/// Should scan the downloads directory and wait until all files are downloaded before proceeding.
	async fn prompt_special_manual_files(&mut self, files: Vec<ManualFile>) -> anyhow::Result<()> {
		let dir = manual_files::get_scan_dir_from_os(OS_STRING);
		while files.len() > manual_files::scan(&dir, &files).len() {
			std::thread::sleep(std::time::Duration::from_millis(100));
		}
		Ok(())
	}

	/// Gets a copy of this output that may technically be used in asynchronous tasks,
	/// but will most likely be used for something synchronous like the output of a plugin command
	fn get_greater_copy(&self) -> Box<dyn NitroOutput + Sync> {
		self.get_lesser_copy()
	}

	/// Gets a copy of this output that can be used for asynchronous tasks.
	/// Note that this does not have to be an exact copy.
	/// If multiple sources writing to your output at the same time would mess up the formatting (like for a terminal),
	/// this copy can be a lesser version that just saves to logging, for example.
	fn get_lesser_copy(&self) -> Box<dyn NitroOutput + Sync> {
		Box::new(NoOp)
	}
}

#[async_trait::async_trait]
impl<T: NitroOutput + Sync + ?Sized> NitroOutput for Box<T> {
	fn display_text(&mut self, text: String, level: MessageLevel) {
		self.deref_mut().display_text(text, level)
	}

	fn display_message(&mut self, message: Message) {
		self.deref_mut().display_message(message)
	}

	fn start_process(&mut self) {
		self.deref_mut().start_process()
	}

	fn end_process(&mut self) {
		self.deref_mut().end_process()
	}

	fn start_section(&mut self) {
		self.deref_mut().start_section()
	}

	fn end_section(&mut self) {
		self.deref_mut().end_section()
	}

	async fn prompt_yes_no(
		&mut self,
		default: bool,
		message: MessageContents,
	) -> anyhow::Result<bool> {
		self.deref_mut().prompt_yes_no(default, message).await
	}

	async fn prompt_password(&mut self, message: MessageContents) -> anyhow::Result<String> {
		self.deref_mut().prompt_password(message).await
	}

	async fn prompt_new_password(&mut self, message: MessageContents) -> anyhow::Result<String> {
		self.deref_mut().prompt_new_password(message).await
	}

	fn translate(&self, key: TranslationKey) -> &str {
		self.deref().translate(key)
	}

	fn display_special_ms_auth(&mut self, url: &str, code: &str) {
		self.deref_mut().display_special_ms_auth(url, code)
	}

	fn display_special_resolution_error(&mut self, error: ResolutionError, instance_id: &str) {
		self.deref_mut()
			.display_special_resolution_error(error, instance_id)
	}

	async fn prompt_special_account_passkey(
		&mut self,
		message: MessageContents,
		account_id: &str,
	) -> anyhow::Result<String> {
		self.deref_mut()
			.prompt_special_account_passkey(message, account_id)
			.await
	}

	async fn prompt_special_package_diffs(
		&mut self,
		diffs: Vec<PackageDiff>,
	) -> anyhow::Result<bool> {
		self.deref_mut().prompt_special_package_diffs(diffs).await
	}

	fn get_greater_copy(&self) -> Box<dyn NitroOutput + Sync> {
		self.deref().get_lesser_copy()
	}

	fn get_lesser_copy(&self) -> Box<dyn NitroOutput + Sync> {
		self.deref().get_lesser_copy()
	}
}

/// Displays the default Microsoft authentication messages
pub fn default_special_ms_auth(o: &mut (impl NitroOutput + ?Sized), url: &str, code: &str) {
	o.display(MessageContents::Property(
		"Open this link in your web browser if it has not opened already".into(),
		Box::new(MessageContents::Hyperlink(url.into())),
	));
	o.display(MessageContents::Property(
		"and enter the code".into(),
		Box::new(MessageContents::Copyable(code.into())),
	));
}

/// A message supplied to the output
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
	/// The contents of the message
	pub contents: MessageContents,
	/// The printing level of the message
	pub level: MessageLevel,
}

/// Contents of a message. Different types represent different formatting
#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MessageContents {
	/// Simple message with no formatting
	Simple(String),
	/// An important notice to the user
	Notice(String),
	/// A warning to the user
	Warning(String),
	/// An error
	Error(String),
	/// A success / finish message
	Success(String),
	/// A key-value property
	Property(String, Box<MessageContents>),
	/// A header / big message
	Header(String),
	/// An start of some long running process. Usually ends with ...
	StartProcess(String),
	/// A message with an associated value displayed along with it.
	Associated(Box<MessageContents>, Box<MessageContents>),
	/// Message with an associated package
	Package(PkgRequest, Box<MessageContents>),
	/// A hyperlink
	Hyperlink(String),
	/// An item in an unordered list
	ListItem(Box<MessageContents>),
	/// Text that can be copied, such as a verification code
	Copyable(String),
	/// A progress indicator
	Progress {
		/// The current amount completed
		current: u32,
		/// The total amount that needs to be completed
		total: u32,
	},
}

impl MessageContents {
	/// Constructs a property message
	pub fn property(property: impl ToString, value: MessageContents) -> Self {
		Self::Property(property.to_string(), Box::new(value))
	}

	/// Constructs an associated message
	pub fn associated(property: MessageContents, value: MessageContents) -> Self {
		Self::Associated(Box::new(property), Box::new(value))
	}

	/// Constructs a list item message
	pub fn list_item(item: MessageContents) -> Self {
		Self::ListItem(Box::new(item))
	}

	/// Message formatting for the default implementation
	pub fn default_format(self) -> String {
		match self {
			MessageContents::Simple(text)
			| MessageContents::Success(text)
			| MessageContents::Hyperlink(text)
			| MessageContents::Copyable(text) => text,
			MessageContents::Notice(text) => format!("Notice: {text}"),
			MessageContents::Warning(text) => format!("Warning: {text}"),
			MessageContents::Error(text) => format!("Error: {text}"),
			MessageContents::Property(key, value) => {
				format!("{key}: {}", value.default_format())
			}
			MessageContents::Header(text) => format!("# {text}"),
			MessageContents::StartProcess(text) => format!("{text}..."),
			MessageContents::Associated(item, message) => {
				format!("[{}] {}", item.default_format(), message.default_format())
			}
			MessageContents::Package(pkg, message) => {
				format!("[{pkg}] {}", message.default_format())
			}
			MessageContents::ListItem(item) => format!(" - {}", item.default_format()),
			MessageContents::Progress { current, total } => format!("{current}/{total}"),
		}
	}
}

impl From<&str> for MessageContents {
	fn from(value: &str) -> Self {
		Self::Simple(value.to_string())
	}
}

impl From<String> for MessageContents {
	fn from(value: String) -> Self {
		Self::Simple(value)
	}
}

/// The level of logging that a message has
#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum MessageLevel {
	/// Very Debug-level messages. Should only be used for logging
	Trace,
	/// Debug-level messages. Good for logging but should not be displayed to
	/// the user unless they ask
	Debug,
	/// Messages that should always be displayed
	Important,
}

/// Dummy NitroOutput that doesn't print anything
#[derive(Clone, Copy)]
pub struct NoOp;

impl NitroOutput for NoOp {
	fn display_text(&mut self, _text: String, _level: MessageLevel) {}
}

/// NitroOutput with simple terminal printing
#[derive(Clone, Copy)]
pub struct Simple(pub MessageLevel);

impl NitroOutput for Simple {
	fn display_text(&mut self, text: String, level: MessageLevel) {
		if level < self.0 {
			return;
		}

		println!("{text}");
	}

	fn get_lesser_copy(&self) -> Box<dyn NitroOutput + Sync> {
		Box::new(Self(self.0))
	}
}

/// NitroOutput with advanced terminal printing, with process and section support
#[derive(Clone)]
pub struct Advanced {
	printer: ReplPrinter,
	in_process: bool,
	indent: usize,
}

impl Advanced {
	/// Creates a new advanced output
	pub fn new() -> Self {
		Self {
			printer: ReplPrinter::new(true),
			in_process: false,
			indent: 0,
		}
	}
}

impl NitroOutput for Advanced {
	fn display_text(&mut self, text: String, level: MessageLevel) {
		if level < MessageLevel::Important {
			return;
		}

		if self.in_process {
			self.printer.print(&text);
		} else {
			self.printer.println(&text);
		}
	}

	fn start_process(&mut self) {
		self.in_process = true;
	}

	fn end_process(&mut self) {
		self.in_process = false;
		self.printer.newline();
	}

	fn start_section(&mut self) {
		self.indent += 1;
		self.printer.indent(self.indent);
	}

	fn end_section(&mut self) {
		if self.indent > 0 {
			self.indent -= 1;
		}
		self.printer.indent(self.indent);
	}

	fn get_lesser_copy(&self) -> Box<dyn NitroOutput + Sync> {
		Box::new(Self {
			printer: self.printer.clone(),
			in_process: self.in_process,
			indent: self.indent,
		})
	}
}

/// Dummy NitroOutput that records messages for testing
#[derive(Clone)]
pub struct TestOutput(pub Vec<Message>);

impl NitroOutput for TestOutput {
	fn display_text(&mut self, _text: String, _level: MessageLevel) {}

	fn display_message(&mut self, message: Message) {
		self.0.push(message);
	}
}

/// NitroOutput that outputs to a writer
#[derive(Clone)]
pub struct WriterOutput<T: Write + Send>(pub T);

impl<T: Write + Send> NitroOutput for WriterOutput<T> {
	fn display_text(&mut self, text: String, _level: MessageLevel) {
		let _ = write!(self.0, "{}", text);
	}
}

/// RAII struct that opens and closes an output process
pub struct OutputProcess<'a, O: NitroOutput>(&'a mut O);

impl<'a, O> OutputProcess<'a, O>
where
	O: NitroOutput,
{
	/// Create a new OutputProcess from an NitroOutput
	pub fn new(o: &'a mut O) -> Self {
		o.start_process();
		Self(o)
	}

	/// Finish the proces early
	pub fn finish(self) {}
}

impl<O> Drop for OutputProcess<'_, O>
where
	O: NitroOutput,
{
	fn drop(&mut self) {
		self.0.end_process();
	}
}

impl<O> Deref for OutputProcess<'_, O>
where
	O: NitroOutput,
{
	type Target = O;

	fn deref(&self) -> &Self::Target {
		self.0
	}
}

impl<O> DerefMut for OutputProcess<'_, O>
where
	O: NitroOutput,
{
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.0
	}
}

/// RAII struct that opens and closes an output section
pub struct OutputSection<'a, O: NitroOutput>(&'a mut O);

impl<'a, O> OutputSection<'a, O>
where
	O: NitroOutput,
{
	/// Create a new OutputProcess from an NitroOutput
	pub fn new(o: &'a mut O) -> Self {
		o.start_section();
		Self(o)
	}

	/// Finish the section early
	pub fn finish(self) {}
}

impl<O> Drop for OutputSection<'_, O>
where
	O: NitroOutput,
{
	fn drop(&mut self) {
		self.0.end_section();
	}
}

impl<O> Deref for OutputSection<'_, O>
where
	O: NitroOutput,
{
	type Target = O;

	fn deref(&self) -> &Self::Target {
		self.0
	}
}

impl<O> DerefMut for OutputSection<'_, O>
where
	O: NitroOutput,
{
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.0
	}
}
