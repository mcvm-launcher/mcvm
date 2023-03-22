use std::io::{Stdout, Write};

use color_print::cstr;

pub static HYPHEN_POINT: &str = cstr!("<k!> - </k!>");
pub static INDENT_CHAR: &str = "\t";

// Used to print text that is replaced
#[derive(Debug)]
pub struct ReplPrinter {
	stdout: Stdout,
	chars_written: usize,
	finished: bool,
	options: PrintOptions,
}

impl ReplPrinter {
	pub fn new(verbose: bool) -> Self {
		Self {
			stdout: std::io::stdout(),
			chars_written: 0,
			finished: false,
			options: PrintOptions::new(verbose, 0),
		}
	}

	pub fn from_options(options: PrintOptions) -> Self {
		Self {
			stdout: std::io::stdout(),
			chars_written: 0,
			finished: false,
			options
		}
	}

	pub fn indent(&mut self, indent: usize) {
		self.options.indent += indent;
		self.options.indent_str = make_indent(self.options.indent);
	}

	pub fn clearline(&mut self) {
		if self.chars_written == 0 {
			return;
		}

		print!("\r");
		for _ in 0..self.chars_written {
			print!(" ");
		}
		self.chars_written = 0;
		self.stdout.flush().unwrap();
	}

	pub fn print(&mut self, text: &str) {
		if !self.options.verbose {
			return;
		}
		self.clearline();
		print!("\r{}{text}", self.options.indent_str);
		self.chars_written = text.len() + (self.options.indent_str.len() * 8);
		self.stdout.flush().unwrap();
	}

	pub fn finish(&mut self) {
		if self.finished {
			return;
		}
		if self.chars_written != 0 {
			println!();
			self.chars_written = 0;
		}
		self.finished = true;
	}

	pub fn newline(&self) {
		println!();
	}
}

impl Drop for ReplPrinter {
	fn drop(&mut self) {
		self.finish();
	}
}

/// Create the characters for an indent count
pub fn make_indent(indent: usize) -> String {
	INDENT_CHAR.repeat(indent)
}

/// Set of options for printing output
#[derive(Debug, Clone)]
pub struct PrintOptions {
	/// Whether to print at all
	pub verbose: bool,
	/// Indent level
	pub indent: usize,
	/// Indent as a string
	pub indent_str: String
}

impl PrintOptions {
	pub fn new(verbose: bool, indent: usize) -> Self {
		Self {
			verbose,
			indent,
			indent_str: make_indent(indent),
		}
	}

	pub fn increase_indent(opt: &Self) -> Self {
		let mut out = opt.clone();
		out.indent += 1;
		out.indent_str = make_indent(out.indent);
		out
	}
}

