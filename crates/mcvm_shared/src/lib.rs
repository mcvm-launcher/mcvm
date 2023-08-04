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

pub mod later {

	/// An enum very similar to `Option<T>` that lets us access it with an easier assertion.
	/// It is meant for data that we know should already be full at some point.
	#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
	pub enum Later<T> {
		#[default]
		Empty,
		Full(T),
	}

	impl<T> Later<T> {
		/// Construct an empty Later
		pub fn new() -> Self {
			Self::Empty
		}

		/// Fill the Later with a value
		pub fn fill(&mut self, value: T) {
			*self = Self::Full(value);
		}

		/// Checks if the Later does not contain a value
		pub fn is_empty(&self) -> bool {
			matches!(self, Self::Empty)
		}

		/// Grab the value inside and panic if it isn't there
		pub fn get(&self) -> &T {
			if let Self::Full(value) = self {
				value
			} else {
				self.fail();
			}
		}

		/// Grab the value inside mutably and panic if it isn't there
		pub fn get_mut(&mut self) -> &mut T {
			if let Self::Full(value) = self {
				value
			} else {
				self.fail();
			}
		}

		/// Grab the value inside without a reference and panic if it isn't there
		pub fn get_val(self) -> T {
			if let Self::Full(value) = self {
				value
			} else {
				self.fail();
			}
		}

		fn fail(&self) -> ! {
			panic!("Value in Later<T> does not exist");
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[test]
		fn test_later_fill() {
			let mut later = Later::new();
			later.fill(7);
			later.get();
		}

		#[test]
		#[should_panic(expected = "Value in Later<T> does not exist")]
		fn test_later_fail() {
			let later: Later<i32> = Later::new();
			later.get();
		}
	}
}
