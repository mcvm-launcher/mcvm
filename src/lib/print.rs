use std::io::Stdout;
use std::io::Write;

// Used to print text that is replaced
pub struct ReplPrinter {
	stdout: Stdout,
	chars_written: usize,
	finished: bool
}

impl ReplPrinter {
	pub fn new() -> Self {
		ReplPrinter {
			stdout: std::io::stdout(),
			chars_written: 0,
			finished: false
		}
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
		self.clearline();
		print!("\r{text}");
		self.chars_written = text.len();
		self.stdout.flush().unwrap();
	}

	pub fn finish(&mut self) {
		if self.finished {
			return;
		}
		self.chars_written = 0;
		println!();
		self.finished = true;
	}
}

impl Drop for ReplPrinter {
	fn drop(&mut self) {
		self.finish();
	}
}
