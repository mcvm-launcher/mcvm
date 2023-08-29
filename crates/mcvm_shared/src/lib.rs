/// Common addon constructs
pub mod addon;
/// Common instance constructs
pub mod instance;
/// Tools for languages and language detection
pub mod lang;
/// Enums for modifications to the game
pub mod modifications;
/// Common package constructs
pub mod pkg;
/// Tools for dealing with version patterns
pub mod versions;

/// Common utilities
pub mod util {
	use serde::Deserialize;

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

	/// Utility enum for deserialization that lets you do a list that can be one item
	#[derive(Deserialize, Debug, Clone)]
	#[serde(untagged)]
	pub enum DeserListOrSingle<T> {
		Single(T),
		List(Vec<T>),
	}

	impl<T> Default for DeserListOrSingle<T> {
		fn default() -> Self {
			Self::List(Vec::default())
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
			self_vec.extend(other.get_vec());
			*self = Self::List(self_vec);
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

		/// Checks if the Later does contain a value
		pub fn is_full(&self) -> bool {
			matches!(self, Self::Full(..))
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

		/// Converts to an Option<T>
		pub fn into_option(self) -> Option<T> {
			match self {
				Self::Empty => None,
				Self::Full(val) => Some(val),
			}
		}

		fn fail(&self) -> ! {
			panic!("Value in Later<T> does not exist");
		}
	}

	impl<T: Clone> Later<T> {
		/// Grab the value by cloning and panic if it isn't there
		pub fn get_clone(&self) -> T {
			if let Self::Full(value) = self {
				value.clone()
			} else {
				self.fail();
			}
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
