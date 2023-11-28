/// Printing and output utilities
pub mod print;

use std::{
	process::{Command, Stdio},
	time::{SystemTime, UNIX_EPOCH},
};

use cfg_match::cfg_match;

cfg_match! {
	target_os = "linux" => {
		/// String representing the current operating system
		pub const OS_STRING: &str = "linux";
	}
	target_os = "windows" => {
		/// String representing the current operating system
		pub const OS_STRING: &str = "windows";
	}
	target_os = "macos" => {
		/// String representing the current operating system
		pub const OS_STRING: &str = "macos";
	}
	_ => {
		compile_error!("Target operating system is unsupported")
		pub const OS_STRING: &str = "";
	}
}

cfg_match! {
	target_arch = "x86" => {
		/// String representing the current architecture
		pub const ARCH_STRING: &str = "x86";
	}
	target_arch = "x86_64" => {
		/// String representing the current architecture
		pub const ARCH_STRING: &str = "x64";
	}
	target_arch = "arm" => {
		/// String representing the current architecture
		pub const ARCH_STRING: &str = "arm";
	}
	target_arch = "aarch64" => {
		/// String representing the current architecture
		pub const ARCH_STRING: &str = "aarch64";
	}
	target_arch = "riscv32" => {
		/// String representing the current architecture
		pub const ARCH_STRING: &str = "riscv32";
	}
	target_arch = "riscv64" => {
		/// String representing the current architecture
		pub const ARCH_STRING: &str = "riscv64";
	}
	target_arch = "mips" => {
		/// String representing the current architecture
		pub const ARCH_STRING: &str = "mips";
	}
	target_arch = "mips64" => {
		/// String representing the current architecture
		pub const ARCH_STRING: &str = "mips64";
	}
	target_arch = "powerpc" => {
		/// String representing the current architecture
		pub const ARCH_STRING: &str = "powerpc";
	}
	target_arch = "powerpc64" => {
		/// String representing the current architecture
		pub const ARCH_STRING: &str = "powerpc64";
	}
	_ => {
		pub const ARCH_STRING: &str = "";
		compile_error!("Target architecture is unsupported")
	}
}

cfg_match! {
	target_os = "linux" => {
		/// String of the preferred archive file extension
		pub const PREFERRED_ARCHIVE: &str = "tar.gz";
	}
	_ => {
		/// String of the preferred archive file extension
		pub const PREFERRED_ARCHIVE: &str = "zip";
	}
}

/// Adds a dot to the preferred archive name
pub fn preferred_archive_extension() -> String {
	format!(".{PREFERRED_ARCHIVE}")
}

cfg_match! {
	target_pointer_width = "64" => {
		/// String representing the current pointer width
		pub const TARGET_BITS_STR: &str = "64";
	}
	_ => {
		/// String representing the current pointer width
		pub const TARGET_BITS_STR: &str = "32";
	}
}

/// Skip in a loop if a result fails
#[macro_export]
macro_rules! skip_fail {
	($res:expr) => {
		match $res {
			Ok(val) => val,
			Err(..) => continue,
		}
	};
}

/// Skip in a loop if an option is none
#[macro_export]
macro_rules! skip_none {
	($res:expr) => {
		match $res {
			Some(val) => val,
			None => continue,
		}
	};
}

/// Capitalizes the first character of a string
pub fn cap_first_letter(string: &str) -> String {
	let mut c = string.chars();
	match c.next() {
		None => String::new(),
		Some(f) => f.to_uppercase().chain(c).collect(),
	}
}

/// Merges two options together with the right one taking precedence
///
/// Right takes precedence when they are both some
/// ```
/// use mcvm_shared::util::merge_options;
///
/// let x = Some(7);
/// let y = Some(8);
/// assert_eq!(merge_options(x, y), Some(8));
/// ```
/// Right is some so it overwrites none
/// ```
/// use mcvm_shared::util::merge_options;
///
/// let x = None;
/// let y = Some(12);
/// assert_eq!(merge_options(x, y), Some(12));
/// ```
/// Uses left because right is none:
/// ```
/// use mcvm_shared::util::merge_options;
///
/// let x = Some(5);
/// let y = None;
/// assert_eq!(merge_options(x, y), Some(5));
/// ```
pub fn merge_options<T>(left: Option<T>, right: Option<T>) -> Option<T> {
	if right.is_some() {
		right
	} else {
		left
	}
}

/// Gets the current UTC timestamp
pub fn utc_timestamp() -> anyhow::Result<u64> {
	Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

/// Trait for a value that can be converted to an integer
pub trait ToInt {
	/// Get this value as an i32
	fn to_int(&self) -> i32;
}

// Command for opening links
cfg_match! {
	target_os = "linux" => {
		const URL_OPEN_CMD: &str = "xdg-open";
	}
	target_os = "windows" => {
		const URL_OPEN_CMD: &str = "start";
	}
	target_os = "macos" => {
		const URL_OPEN_CMD: &str = "open";
	}
	_ => {
		compile_error!("Target operating system is unsupported")
	}
}

/// Attempt to open a link on the user's computer
pub fn open_link(link: &str) -> anyhow::Result<()> {
	Command::new(URL_OPEN_CMD)
		.arg(link)
		.stderr(Stdio::null())
		.stdout(Stdio::null())
		.spawn()?;

	Ok(())
}

#[cfg(feature = "schema")]
	use schemars::JsonSchema;
	use serde::{Deserialize, Serialize};

	/// Converts "yes" or "no" to a boolean
	pub fn yes_no(string: &str) -> Option<bool> {
		match string {
			"yes" => Some(true),
			"no" => Some(false),
			_ => None,
		}
	}

	/// Checks if a string is a valid identifier
	pub fn is_valid_identifier(id: &str) -> bool {
		for c in id.chars() {
			if !c.is_ascii() {
				return false;
			}

			if c.is_ascii_punctuation() {
				match c {
					'_' | '-' | '.' => {}
					_ => return false,
				}
			}

			if c.is_ascii_whitespace() {
				return false;
			}
		}

		true
	}

	/// Utility enum for deserialization that lets you do a list that can be one item
	/// without the braces
	#[derive(Deserialize, Debug, Clone)]
	#[cfg_attr(feature = "schema", derive(JsonSchema))]
	#[serde(untagged)]
	pub enum DeserListOrSingle<T> {
		/// Only one item, specified without braces
		Single(T),
		/// A list of items, specified with braces
		List(Vec<T>),
	}

	impl<T> Default for DeserListOrSingle<T> {
		fn default() -> Self {
			Self::List(Vec::default())
		}
	}

	impl<T: Serialize> Serialize for DeserListOrSingle<T> {
		fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
		where
			S: serde::Serializer,
		{
			match self {
				Self::List(list) => {
					if list.len() == 1 {
						list[0].serialize(serializer)
					} else {
						list.serialize(serializer)
					}
				}
				Self::Single(val) => val.serialize(serializer),
			}
		}
	}

	impl<T> DeserListOrSingle<T> {
		/// Checks if this value is empty
		pub fn is_empty(&self) -> bool {
			matches!(self, Self::List(list) if list.is_empty())
		}

		/// Iterates over this DeserListOrSingle
		pub fn iter(&self) -> DeserListOrSingleIter<'_, T> {
			match &self {
				Self::Single(val) => {
					DeserListOrSingleIter(DeserListOrSingleIterState::Single(Some(val)))
				}
				Self::List(list) => {
					DeserListOrSingleIter(DeserListOrSingleIterState::List(list.iter()))
				}
			}
		}
	}

	impl<T: Clone> DeserListOrSingle<T> {
		/// Get the contained value as a Vec
		pub fn get_vec(&self) -> Vec<T> {
			match &self {
				Self::Single(val) => vec![val.clone()],
				Self::List(list) => list.clone(),
			}
		}

		/// Merges this enum with another
		pub fn merge(&mut self, other: Self) {
			let mut self_vec = self.get_vec();
			self_vec.extend(other.iter().cloned());
			*self = Self::List(self_vec);
		}
	}

	/// Iterator over DeserListOrSingle
	pub struct DeserListOrSingleIter<'a, T>(DeserListOrSingleIterState<'a, T>);

	/// State for a DeserListOrSingleIter
	enum DeserListOrSingleIterState<'a, T> {
		Single(Option<&'a T>),
		List(std::slice::Iter<'a, T>),
	}

	impl<'a, T> Iterator for DeserListOrSingleIter<'a, T> {
		type Item = &'a T;

		fn next(&mut self) -> Option<Self::Item> {
			match &mut self.0 {
				DeserListOrSingleIterState::Single(val) => val.take(),
				DeserListOrSingleIterState::List(slice_iter) => slice_iter.next(),
			}
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[test]
		fn test_id_validation() {
			assert!(is_valid_identifier("hello"));
			assert!(is_valid_identifier("Hello"));
			assert!(is_valid_identifier("H3110"));
			assert!(is_valid_identifier("hello-world"));
			assert!(is_valid_identifier("hello_world"));
			assert!(is_valid_identifier("hello.world"));
			assert!(!is_valid_identifier("hello*world"));
			assert!(!is_valid_identifier("hello\nworld"));
			assert!(!is_valid_identifier("hello world"));
		}

		#[test]
		fn test_deser_list_or_single_iter() {
			let item = DeserListOrSingle::Single(7);
			assert_eq!(item.iter().next(), Some(&7));

			let item = DeserListOrSingle::List(vec![1, 2, 3]);
			let mut iter = item.iter();
			assert_eq!(iter.next(), Some(&1));
			assert_eq!(iter.next(), Some(&2));
			assert_eq!(iter.next(), Some(&3));
		}
	}
