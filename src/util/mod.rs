pub mod versions;
pub mod json;
pub mod mojang;
pub mod print;

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

// Combo of option and result
pub enum OptionResult<T, E> {
	Some(T),
	None,
	Err(E)
}
