use std::collections::HashMap;

use anyhow::{anyhow, bail};

/// Constant var for the Minecraft version
pub const CONSTANT_VAR_MC_VERSION: &str = "MINECRAFT_VERSION";
/// Constant variables that are reserved by mcvm
pub const RESERVED_CONSTANT_VARS: [&str; 1] = [CONSTANT_VAR_MC_VERSION];

/// Check if a variable identifier is a reserved constant variable
pub fn is_reserved_constant_var(var: &str) -> bool {
	RESERVED_CONSTANT_VARS.contains(&var)
}

/// Struct for reserved constant variables
pub struct ReservedConstantVariables<'a> {
	pub mc_version: &'a str,
}

/// Argument to a command that could be a literal or a variable
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
	None,
	Constant(String),
	Var(String),
}

/// An trait that can be used to get and set variables used in script evaluation
pub trait VariableStore {
	/// Set the value of a variable in the store
	fn get_var(&self, var: &str) -> Option<&str>;

	/// Get the value of a variable in the store
	fn set_var(&mut self, var: String, val: String);

	/// Tries to set a variable, but checks if the variable is a constant variable.
	/// This should be used for your 'set' instruction evaluation
	fn try_set_var(&mut self, var: String, val: String) -> anyhow::Result<()> {
		if is_reserved_constant_var(&var) {
			bail!("Tried to set the value of a reserved constant variable");
		}

		self.set_var(var, val);

		Ok(())
	}

	/// Set the values of the reserved constants. Should be run before evaluation
	fn set_reserved_constants(&mut self, constants: ReservedConstantVariables) {
		self.set_var(
			CONSTANT_VAR_MC_VERSION.to_string(),
			constants.mc_version.to_string(),
		);
	}

	/// Check if the store contains a value
	fn var_exists(&self, var: &str) -> bool {
		self.get_var(var).is_some()
	}
}

/// HashMap implementation of a VariableStore
#[derive(Debug, Default, Clone)]
pub struct HashMapVariableStore(HashMap<String, String>);

impl HashMapVariableStore {
	pub fn new() -> Self {
		Self(HashMap::new())
	}
}

impl VariableStore for HashMapVariableStore {
	fn get_var(&self, var: &str) -> Option<&str> {
		self.0.get(var).map(String::as_str)
	}

	fn set_var(&mut self, var: String, val: String) {
		self.0.insert(var, val);
	}

	fn var_exists(&self, var: &str) -> bool {
		self.0.contains_key(var)
	}
}

impl Value {
	/// Replace substitution tokens in a string with variable values
	pub fn substitute_tokens(string: &str, vars: &impl VariableStore) -> String {
		/// What the parser is looking for
		enum State {
			Escaped,
			Dollar,
			OpenBracket,
			Name,
		}
		let mut state = State::Dollar;
		let mut out = String::new();
		let mut name = String::new();
		for c in string.chars() {
			state = match state {
				State::Escaped => {
					out.push(c);
					State::Dollar
				}
				State::Dollar => match c {
					'$' => State::OpenBracket,
					'\\' => State::Escaped,
					_ => {
						out.push(c);
						state
					}
				},
				State::OpenBracket => {
					if c == '{' {
						State::Name
					} else {
						out.push(c);
						State::Dollar
					}
				}
				State::Name => {
					if c == '}' {
						if let Some(var) = vars.get_var(&name) {
							out.push_str(var);
						}
						name.clear();
						State::Dollar
					} else {
						name.push(c);
						state
					}
				}
			}
		}

		out
	}

	/// Obtain the current String value of this Value.
	/// Will fail if it is none or the variable is uninitialized.
	pub fn get(&self, vars: &impl VariableStore) -> anyhow::Result<String> {
		match self {
			Self::None => bail!("Empty value"),
			Self::Constant(val) => Ok(Self::substitute_tokens(val, vars)),
			Self::Var(name) => vars
				.get_var(name)
				.map(str::to_string)
				.ok_or(anyhow!("Variable {name} is not defined")),
		}
	}

	/// Returns whether this value is not none
	pub fn is_some(&self) -> bool {
		!matches!(self, Self::None)
	}

	/// Gets the current string value and converts to an option.
	pub fn get_as_option(&self, vars: &impl VariableStore) -> anyhow::Result<Option<String>> {
		match self {
			Self::None => Ok(None),
			_ => Ok(Some(self.get(vars)?)),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_value_substitution() {
		let vars = {
			let mut vars = HashMapVariableStore::new();
			vars.set_var(String::from("bar"), String::from("foo"));
			vars.set_var(String::from("hello"), String::from("who"));
			vars
		};

		let string = "One ${bar} skip a ${hello}";
		let string = Value::substitute_tokens(string, &vars);
		assert_eq!(string, "One foo skip a who");
	}
}
