/// Utilities for working with hashes and checksums
pub mod hash;
/// Utilities for working with serde_json values
pub mod json;
/// Utilities for certain mojang formats
pub mod mojang;
/// Printing and output utilities
pub mod print;
/// Utilities for game versions
pub mod versions;

use std::{
	process::{Command, Stdio},
	time::{SystemTime, UNIX_EPOCH},
};

use cfg_match::cfg_match;
use rand::Rng;

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
pub fn select_random_n_items_from_list<T>(list: &[T], n: usize) -> Vec<&T> {
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
		.stdout(Stdio::null())
		.spawn()?;

	Ok(())
}
