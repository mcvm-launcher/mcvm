pub mod addon;
pub mod modifications;
pub mod versions;
pub mod pkg;
pub mod instance;

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
