use std::path::Path;

use cfg_match::cfg_match;

cfg_match! {
	target_os = "linux" => {
		pub static CLASSPATH_SEP: char = ':';
	}
	target_os = "windows" => {
		pub static CLASSPATH_SEP: char = ';';
	}
}

/// A utility for working with Java classpaths
#[derive(Debug, Clone, PartialEq)]
pub struct Classpath {
	string: String,
}

impl Classpath {
	pub fn new() -> Self {
		Self {
			string: String::new(),
		}
	}

	/// Append the classpath separator to the end of the string
	pub fn add_sep(&mut self) {
		self.string.push(CLASSPATH_SEP);
	}

	/// Appends a string to the end of the classpath
	pub fn add(&mut self, string: &str) {
		if !self.string.is_empty() {
			self.add_sep();
		}

		if let Some(c) = self.string.chars().last() {
			if c == CLASSPATH_SEP {
				return;
			}
		}

		self.string.push_str(string);
	}

	/// Converts a path to a string and appends it to the classpath
	pub fn add_path(&mut self, path: &Path) {
		self.add(path.to_str().expect("Failed to convert path to a string"))
	}

	/// Extends the classpath with another classpath
	pub fn extend(&mut self, other: Classpath) {
		self.add(&other.string)
	}

	/// Obtain the classpath as a string
	pub fn get_str(&self) -> String {
		self.string.clone()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_classpath() {
		let mut classpath = Classpath::new();
		assert_eq!(classpath.get_str(), String::new());
		classpath.add("foo");
		assert_eq!(classpath.get_str(), String::from("foo"));
		classpath.add("bar");
		assert_eq!(classpath.get_str(), String::from("foo") + &String::from(CLASSPATH_SEP) + "bar");
	}
}
