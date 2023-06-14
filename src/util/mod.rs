pub mod json;
pub mod mojang;
pub mod print;
pub mod versions;

use std::time::{SystemTime, UNIX_EPOCH};

use cfg_match::cfg_match;
use rand::Rng;

cfg_match! {
	target_os = "linux" => {
		pub static OS_STRING: &str = "linux";
	}
	target_os = "windows" => {
		pub static OS_STRING: &str = "windows";
	}
	_ => {
		pub static OS_STRING: &str = "";
		compile_error!("Target operating system is unsupported")
	}
}

cfg_match! {
	target_arch = "x86" => {
		pub static ARCH_STRING: &str = "x86";
	}
	target_arch = "x86_64" => {
		pub static ARCH_STRING: &str = "x64";
	}
	target_arch = "arm" => {
		pub static ARCH_STRING: &str = "arm";
	}
	_ => {
		pub static ARCH_STRING: &str = "";
		compile_error!("Target architecture is unsupported")
	}
}

cfg_match! {
	target_os = "linux" => {
		pub static PREFERRED_ARCHIVE: &str = "tar.gz";
	}
	_ => {
		pub static PREFERRED_ARCHIVE: &str = "zip";
	}
}

/// Adds a dot to the preferred archive name
pub fn preferred_archive_extension() -> String {
	format!(".{PREFERRED_ARCHIVE}")
}

cfg_match! {
	target_pointer_width = "64" => {
		pub static TARGET_BITS_STR: &str = "64";
	}
	_ => {
		pub static TARGET_BITS_STR: &str = "32";
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

/// Validates a simple string identifier
pub fn validate_identifier(id: &str) -> bool {
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
/// use mcvm::util::merge_options;
///
/// let x = Some(7);
/// let y = Some(8);
/// assert_eq!(merge_options(x, y), Some(8));
/// ```
/// Right is some so it overwrites none
/// ```
/// use mcvm::util::merge_options;
///
/// let x = None;
/// let y = Some(12);
/// assert_eq!(merge_options(x, y), Some(12));
/// ```
/// Uses left because right is none:
/// ```
/// use mcvm::util::merge_options;
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

/// Selects a random set of n elements from a list. The return slice will not necessarily be of n length
pub fn select_random_n_items_from_list<'a, T>(list: &'a [T], n: usize) -> Vec<&'a T> {
	let mut indices: Vec<usize> = (0..list.len()).collect();
	let mut rng = rand::thread_rng();
	let mut chosen = Vec::new();
	for _ in 0..n {
		if indices.is_empty() {
			break;
		}

		let index = rng.gen_range(0..indices.len());
		let index = indices.remove(index);
		chosen.push(&list[index]);
	}

	chosen
}

/// Gets the current UTC timestamp
pub fn utc_timestamp() -> anyhow::Result<u64> {
	Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

pub trait ToInt {
	fn to_int(&self) -> i32;
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_id_validation() {
		assert!(validate_identifier("hello"));
		assert!(validate_identifier("Hello"));
		assert!(validate_identifier("H3110"));
		assert!(validate_identifier("hello-world"));
		assert!(validate_identifier("hello_world"));
		assert!(validate_identifier("hello.world"));
		assert!(!validate_identifier("hello*world"));
		assert!(!validate_identifier("hello\nworld"));
		assert!(!validate_identifier("hello world"));
	}
}
