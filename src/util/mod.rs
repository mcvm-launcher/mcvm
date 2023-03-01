pub mod versions;
pub mod json;
pub mod mojang;
pub mod print;

pub fn yes_no(string: &str) -> Option<bool> {
	match string {
		"yes" => Some(true),
		"no" => Some(false),
		_ => None
	}
}

// Skip in a loop if a result fails
#[macro_export]
macro_rules! skip_fail {
	($res:expr) => {
		match $res {
			Ok(val) => val,
			Err(..) => continue
		}
	};
}

// Skip in a loop if an option is none
#[macro_export]
macro_rules! skip_none {
	($res:expr) => {
		match $res {
			Some(val) => val,
			None => continue
		}
	};
}