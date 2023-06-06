pub mod addon;
pub mod instance;
pub mod modifications;
pub mod pkg;
pub mod versions;
pub mod lang;

pub mod util {
	/// Converts "yes" or "no" to a boolean
	pub fn yes_no(string: &str) -> Option<bool> {
		match string {
			"yes" => Some(true),
			"no" => Some(false),
			_ => None,
		}
	}
}
