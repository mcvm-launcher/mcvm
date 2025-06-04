use std::path::{Path, PathBuf};

use anyhow::Context;

/// The separator for entries in the classpath
#[cfg(target_os = "linux")]
pub const CLASSPATH_SEP: char = ':';
#[cfg(target_os = "macos")]
/// The separator for entries in the classpath
pub const CLASSPATH_SEP: char = ':';
#[cfg(target_os = "windows")]
/// The separator for entries in the classpath
pub const CLASSPATH_SEP: char = ';';

/// A utility for working with Java classpaths
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Classpath {
	entries: Vec<String>,
}

impl Classpath {
	/// Create a new empty classpath
	pub fn new() -> Self {
		Self {
			entries: Vec::new(),
		}
	}

	/// Appends a string to the end of the classpath
	pub fn add(&mut self, string: &str) {
		self.entries.push(string.to_string());
	}

	/// Converts a path to a string and appends it to the classpath
	pub fn add_path(&mut self, path: &Path) -> anyhow::Result<()> {
		self.add(
			path.to_str()
				.context("Failed to convert path to a string")?,
		);

		Ok(())
	}

	/// Adds multiple path strings to the classpath
	pub fn add_multiple<I: AsRef<str>>(&mut self, strings: impl Iterator<Item = I>) {
		for string in strings {
			self.add(string.as_ref());
		}
	}

	/// Extends the classpath with another classpath
	pub fn extend(&mut self, other: Classpath) {
		self.entries.extend(other.entries);
	}

	/// Obtain the classpath as a string
	pub fn get_str(&self) -> String {
		let mut buf = [0; 4];
		self.entries.join(CLASSPATH_SEP.encode_utf8(&mut buf))
	}

	/// Get the classpath as a list of entries
	pub fn get_entries(&self) -> &[String] {
		&self.entries
	}

	/// Split the classpath into a vector of paths
	pub fn get_paths(&self) -> Vec<PathBuf> {
		self.entries.iter().map(PathBuf::from).collect()
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
		assert_eq!(classpath.get_str(), "foo".to_string());
		classpath.add("bar");
		assert_eq!(
			classpath.get_str(),
			"foo".to_string() + &CLASSPATH_SEP.to_string() + "bar"
		);
	}

	#[test]
	fn test_classpath_extension() {
		let mut classpath = Classpath::new();
		classpath.add("foo");
		classpath.add("bar");
		classpath.add("baz");
		let mut classpath2 = Classpath::new();
		classpath2.add("hello");
		classpath2.add("world");
		classpath.extend(classpath2);
		assert_eq!(
			classpath.get_str(),
			format!("foo{0}bar{0}baz{0}hello{0}world", CLASSPATH_SEP)
		);
	}
}
