use crate::pkg::PkgRequest;

/// Trait for a type that can output information about MCVM processes
pub trait MCVMOutput {
	/// Base function for a simple message. Used as a fallback
	fn display_text(&mut self, text: String, level: MessageLevel);

	/// Function to display a message to the user
	fn display_message(&mut self, message: Message) {
		self.display_text(default_format_message(message.contents), message.level);
	}

	/// Convenience function to remove the need to construct a message
	fn display(&mut self, contents: MessageContents, level: MessageLevel) {
		self.display_message(Message { contents, level })
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

	/// Offer a confirmation / yes no prompt to the user.
	/// The default is the default value of the prompt.
	fn prompt_yes_no(&mut self, default: bool, message: MessageContents) -> anyhow::Result<bool> {
		let _message = message;
		Ok(default)
	}
}

/// Message formatting for the default implementation
fn default_format_message(contents: MessageContents) -> String {
	match contents {
		MessageContents::Simple(text)
		| MessageContents::Success(text)
		| MessageContents::Hyperlink(text)
		| MessageContents::Copyable(text) => text,
		MessageContents::Notice(text) => format!("Notice: {text}"),
		MessageContents::Warning(text) => format!("Warning: {text}"),
		MessageContents::Error(text) => format!("Error: {text}"),
		MessageContents::Property(key, value) => {
			format!("{key}: {}", default_format_message(*value))
		}
		MessageContents::Header(text) => text.to_uppercase(),
		MessageContents::StartProcess(text) => format!("{text}..."),
		MessageContents::Associated(item, message) => {
			format!("[{item}] {}", default_format_message(*message))
		}
		MessageContents::Package(pkg, message) => {
			format!("[{pkg}] {}", default_format_message(*message))
		}
		MessageContents::ListItem(item) => format!(" - {}", default_format_message(*item)),
	}
}

/// A message supplied to the output
#[derive(Clone, Debug)]
pub struct Message {
	/// The contents of the message
	pub contents: MessageContents,
	/// The printing level of the message
	pub level: MessageLevel,
}

/// Contents of a message. Different types represent different formatting
#[derive(Clone, Debug)]
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
	Associated(String, Box<MessageContents>),
	/// Message with an associated package
	Package(PkgRequest, Box<MessageContents>),
	/// A hyperlink
	Hyperlink(String),
	/// An item in an unordered list
	ListItem(Box<MessageContents>),
	/// Text that can be copied, such as a verification code
	Copyable(String),
}

/// The level of logging that a message has
#[derive(Copy, Clone, Debug)]
pub enum MessageLevel {
	/// Messages that should always be displayed
	Important,
	/// Messages that can be displayed but are not required
	Extra,
	/// Debug-level messages. Good for logging but should not be displayed to the user
	Debug,
}

impl MessageLevel {
	/// Checks if this level is at least another level
	pub fn at_least(&self, other: &Self) -> bool {
		match &self {
			Self::Important => matches!(other, Self::Important | Self::Extra | Self::Debug),
			Self::Extra => matches!(other, Self::Extra | Self::Debug),
			Self::Debug => matches!(other, Self::Debug),
		}
	}
}

/// Dummy MCVMOutput that doesn't print anything
pub struct NoOp;

impl MCVMOutput for NoOp {
	fn display_text(&mut self, _text: String, _level: MessageLevel) {}
}

/// MCVMOutput with simple terminal printing
pub struct Simple(pub MessageLevel);

impl MCVMOutput for Simple {
	fn display_text(&mut self, text: String, level: MessageLevel) {
		if !level.at_least(&self.0) {
			return;
		}

		println!("{text}");
	}
}

/// RAII struct that opens and closes an output process
pub struct OutputProcess<'a, O: MCVMOutput>(pub &'a mut O);

impl<'a, O> OutputProcess<'a, O>
where
	O: MCVMOutput,
{
	pub fn new(o: &'a mut O) -> Self {
		o.start_process();
		Self(o)
	}
}

impl<'a, O> Drop for OutputProcess<'a, O>
where
	O: MCVMOutput,
{
	fn drop(&mut self) {
		self.0.end_process();
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_level_is_at_least() {
		assert!(MessageLevel::Extra.at_least(&MessageLevel::Debug));
		assert!(MessageLevel::Debug.at_least(&MessageLevel::Debug));
		assert!(!MessageLevel::Debug.at_least(&MessageLevel::Extra));
	}
}
