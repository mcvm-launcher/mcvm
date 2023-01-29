use std::io::{Stdout, Write};

// Used to print text that is replaced
pub struct ReplPrinter {
	stdout: Stdout,
	chars_written: usize,
	finished: bool,
	verbose: bool,
	indent: usize
}

impl ReplPrinter {
	pub fn new(verbose: bool) -> Self {
		ReplPrinter {
			stdout: std::io::stdout(),
			chars_written: 0,
			finished: false,
			verbose,
			indent: 0
		}
	}

	pub fn indent(&mut self, indent: usize) {
		self.indent = indent;
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
		if !self.verbose {
			return;
		}
		self.clearline();
		let indent_str = std::iter::repeat("\t").take(self.indent).collect::<String>();
		print!("\r{indent_str}{text}");
		self.chars_written = text.len();
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
}

impl Drop for ReplPrinter {
	fn drop(&mut self) {
		self.finish();
	}
}
