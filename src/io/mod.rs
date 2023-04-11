pub mod files;
pub mod java;
pub mod launch;
pub mod lock;
pub mod options;

/// An enum very similar to `Option<T>` that lets us access it with an easier assertion.
/// It is meant for data that we know should already be full at some point.
#[derive(Clone, Default)]
pub enum Later<T> {
	#[default]
	Empty,
	Full(T),
}

impl <T> Later<T> {
	/// Construct an empty Later
	pub fn new() -> Self {
		Self::Empty
	}

	/// Fill the Later with a value
	pub fn fill(&mut self, value: T) {
		*self = Self::Full(value);
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
