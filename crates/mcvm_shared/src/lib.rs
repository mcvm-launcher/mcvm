pub mod addon;
pub mod instance;
pub mod lang;
pub mod modifications;
pub mod pkg;
pub mod versions;

pub mod util {
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
	}
}
