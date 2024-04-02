#![warn(missing_docs)]

//! This crate contains shared data for the other MCVM crates
//!
//! # Features:
//!
//! - `schema`: Enable generation of JSON schemas using the `schemars` crate

/// Common addon constructs
pub mod addon;
/// Tools for languages and language detection
pub mod lang;
/// Enums for modifications to the game
pub mod modifications;
/// MCVM output
pub mod output;
/// Common package constructs
pub mod pkg;
/// Other utilities
pub mod util;
/// Tools for dealing with version patterns
pub mod versions;

use std::{fmt::Display, str::FromStr};

use anyhow::anyhow;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The Later<T> enum
pub mod later {
	/// An enum very similar to `Option<T>` that lets us access it with an easier assertion.
	/// It is meant for data that we know should already be full at some point.
	#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
	pub enum Later<T> {
		/// The Later does not contain a value
		#[default]
		Empty,
		/// The later does contain a value
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

		/// Fill the Later with a function if it is not full already
		pub fn ensure_full(&mut self, f: impl Fn() -> T) {
			if self.is_empty() {
				self.fill(f());
			}
		}

		/// Clear the Later
		pub fn clear(&mut self) {
			*self = Self::Empty;
		}

		/// Checks if the Later does not contain a value
		#[must_use]
		pub fn is_empty(&self) -> bool {
			matches!(self, Self::Empty)
		}

		/// Checks if the Later does contain a value
		#[must_use]
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

		/// Converts to an `Option<T>`
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

/// Minecraft game side, client or server
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Side {
	/// The default game
	Client,
	/// A dedicated server
	Server,
}

impl Side {
	/// Parse a Side from a string
	pub fn parse_from_str(string: &str) -> Option<Self> {
		match string {
			"client" => Some(Self::Client),
			"server" => Some(Self::Server),
			_ => None,
		}
	}
}

impl FromStr for Side {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::parse_from_str(s).ok_or(anyhow!("Not a valid side"))
	}
}

impl Display for Side {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Client => "client",
				Self::Server => "server",
			}
		)
	}
}
