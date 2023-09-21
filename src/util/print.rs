use std::fmt::Debug;
use std::io::{Stdout, Write};

// use color_print::{cformat, cstr};

/// String used program-wide for most indentation
pub const INDENT_STR: &str = "  ";

/// Used to print text that is replaced
#[derive(Debug)]
pub struct ReplPrinter {
	stdout: Stdout,
	chars_written: usize,
	finished: bool,
	options: PrintOptions,
}

impl ReplPrinter {
	/// Make a new ReplPrinter with a verbosity option.
	/// If that option is false, then nothing will be printed
	pub fn new(verbose: bool) -> Self {
		Self {
			stdout: std::io::stdout(),
			chars_written: 0,
			finished: false,
			options: PrintOptions::new(verbose, 0),
		}
	}

	/// Make a new ReplPrinter using a set of print options
	pub fn from_options(options: PrintOptions) -> Self {
		Self {
			stdout: std::io::stdout(),
			chars_written: 0,
			finished: false,
			options,
		}
	}

	/// Set the indent level of the printer
	pub fn indent(&mut self, indent: usize) {
		self.options.indent = indent;
		self.options.indent_str = make_indent(self.options.indent);
	}

	/// Replace the current line with spaces
	pub fn clearline(&mut self) {
		if self.chars_written == 0 {
			return;
		}

		let _ = write!(self.stdout, "\r");
		for _ in 0..self.chars_written {
			let _ = write!(self.stdout, " ");
		}
		self.chars_written = 0;
		let _ = self.stdout.flush();
	}

	/// Print text to the output, replacing the current line
	pub fn print(&mut self, text: &str) {
		if !self.options.verbose {
			return;
		}
		self.clearline();
		let _ = write!(self.stdout, "\r{}{text}", self.options.indent_str);
		self.chars_written = text.len() + (self.options.indent_str.len() * 8);
		let _ = self.stdout.flush();
	}

	/// Print text on a new line
	pub fn println(&mut self, text: &str) {
		self.chars_written = 0;
		let _ = writeln!(self.stdout);
		self.print(text);
	}

	/// Finish printing and make a newline
	pub fn finish(&mut self) {
		if self.finished {
			return;
		}
		if self.chars_written != 0 {
			self.newline();
		}
		self.finished = true;
	}

	/// Make a line break
	pub fn newline(&mut self) {
		self.println("");
	}
}

impl Drop for ReplPrinter {
	fn drop(&mut self) {
		self.finish();
	}
}

/// Create the characters for an indent count
pub fn make_indent(indent: usize) -> String {
	INDENT_STR.repeat(indent)
}

/// Set of options for printing output
#[derive(Debug, Clone)]
pub struct PrintOptions {
	/// Whether to print at all
	pub verbose: bool,
	/// Indent level
	pub indent: usize,
	/// Indent as a string
	pub indent_str: String,
}

impl PrintOptions {
	/// Create a new PrintOptions with verbosity and indent level settings
	pub fn new(verbose: bool, indent: usize) -> Self {
		Self {
			verbose,
			indent,
			indent_str: make_indent(indent),
		}
	}

	/// Increase the indent of the PrintOptions
	pub fn increase_indent(opt: &Self) -> Self {
		let mut out = opt.clone();
		out.indent += 1;
		out.indent_str = make_indent(out.indent);
		out
	}
}
