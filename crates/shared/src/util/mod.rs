/// Printing and output utilities
pub mod print;

use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use cfg_match::cfg_match;

macro_rules! def_matched_item {
	($cfg:ident, $doc:literal, $name:ident, $err:literal, $($k:literal: $v: literal);* $(;)?) => {
		cfg_match! {
			$(
				$cfg = $k => {
					#[doc = $doc]
					pub const $name: &str = $v;
				}
			)*
			_ => {
				compile_error!($err)
				pub const $name: &str = "";
			}
		}
	};
}

def_matched_item! {
	target_os,
	"String representing the current operating system",
	OS_STRING,
	"Target operating system is unsupported",
	"linux": "linux";
	"windows": "windows";
	"macos": "macos";
	"ios": "ios";
	"android": "android";
	"freebsd": "freebsd";
	"dragonfly": "dragonfly";
	"bitrig": "bitrig";
	"netbsd": "netbsd";
	"openbsd": "openbsd";
}

def_matched_item! {
	target_arch,
	"String representing the current architecture",
	ARCH_STRING,
	"Target architecture is unsupported",
	"x86": "x86";
	"x86_64": "x86_64";
	"arm": "arm";
	"aarch64": "aarch64";
	"riscv32": "riscv32";
	"riscv64": "riscv64";
	"mips": "mips";
	"mips64": "mips64";
	"powerpc": "powerpc";
	"powerpc64": "powerpc64";
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

/// Gets the current UTC timestamp in seconds
pub fn utc_timestamp() -> anyhow::Result<u64> {
	Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

/// Trait for a value that can be converted to an integer
pub trait ToInt {
	/// Get this value as an i32
	fn to_int(&self) -> i32;
}

impl ToInt for bool {
	fn to_int(&self) -> i32 {
		*self as i32
	}
}

// Command for opening links
cfg_match! {
	target_os = "linux" => {
		const URL_OPEN_CMD: Option<&str> = Some("xdg-open");
	}
	target_os = "windows" => {
		const URL_OPEN_CMD: Option<&str> = Some("start");
	}
	target_os = "macos" => {
		const URL_OPEN_CMD: Option<&str> = Some("open");
	}
	_ => {
		const URL_OPEN_CMD: Option<&str> = None;
	}
}

/// Attempt to open a link on the user's computer
pub fn open_link(link: &str) -> anyhow::Result<()> {
	let Some(cmd) = URL_OPEN_CMD else {
		return Ok(());
	};

	Command::new(cmd)
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
#[derive(Deserialize, Debug, Clone, Eq)]
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

	/// Checks if an option of this struct is empty
	pub fn is_option_empty(val: &Option<Self>) -> bool {
		val.is_none() || matches!(val, Some(val) if val.is_empty())
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

impl<T: PartialEq> PartialEq for DeserListOrSingle<T> {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(DeserListOrSingle::Single(l), DeserListOrSingle::Single(r)) => l == r,
			(DeserListOrSingle::List(l), DeserListOrSingle::List(r)) => l == r,
			(DeserListOrSingle::List(l), DeserListOrSingle::Single(r)) => {
				l.len() == 1 && l.first().expect("Length is 1") == r
			}
			(DeserListOrSingle::Single(l), DeserListOrSingle::List(r)) => {
				r.len() == 1 && r.first().expect("Length is 1") == l
			}
		}
	}
}

impl<T: Clone> Extend<T> for DeserListOrSingle<T> {
	fn extend<U: IntoIterator<Item = T>>(&mut self, iter: U) {
		// Convert single to list
		if let Self::Single(item) = self {
			*self = Self::List(vec![item.clone()]);
		}
		// Extend the list
		if let Self::List(list) = self {
			list.extend(iter);
			// Convert back to single if there is only one item
			if list.len() == 1 {
				*self = Self::Single(list.first().expect("Length is 1").clone());
			}
		}
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

/// Extension trait for Default
pub trait DefaultExt {
	/// Check if the value is equal to it's default value
	fn is_default(&self) -> bool;
}

impl<T: Default + PartialEq> DefaultExt for T {
	fn is_default(&self) -> bool {
		self == &Self::default()
	}
}

/// Macro to try a fallible operation multiple times before giving up and returning an error
#[macro_export]
macro_rules! try_3 {
	($op:block) => {
		if let Ok(out) = $op {
			Ok(out)
		} else {
			if let Ok(out) = $op {
				Ok(out)
			} else {
				$op
			}
		}
	};
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
