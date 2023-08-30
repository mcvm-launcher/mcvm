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
		}
	}
}

/// A message supplied to the output
#[derive(Clone, Debug)]
pub struct Message {
	pub contents: MessageContents,
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
