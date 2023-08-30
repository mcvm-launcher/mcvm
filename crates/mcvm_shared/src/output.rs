/// Trait for a type that can output information about MCVM processes
pub trait MCVMOutput {
	/// Base function for a simple message. Used as a fallback
	fn display_text(&mut self, text: String, level: MessageLevel);

	/// Function to display a message to the user
	fn display_message(&mut self, message: Message) {
		match message.contents {
			MessageContents::Simple(text) | MessageContents::Success(text) => {
				self.display_text(text, message.level)
			}
			MessageContents::Warning(text) => {
				self.display_text(format!("Warning: {text}"), message.level)
			}
			MessageContents::Error(text) => {
				self.display_text(format!("Error: {text}"), message.level)
			}
			MessageContents::Property(key, value) => {
				self.display_text(format!("{key}: {value}"), message.level)
			}
			MessageContents::Header(text) => self.display_text(text.to_uppercase(), message.level),
			MessageContents::StartProcess(text) => {
				self.display_text(format!("{text}..."), message.level)
			}
		}
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
	/// A warning to the user
	Warning(String),
	/// An error
	Error(String),
	/// A success / finish message
	Success(String),
	/// A key-value property
	Property(String, String),
	/// A header / big message
	Header(String),
	/// An start of some long running process. Usually ends with ...
	StartProcess(String),
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
