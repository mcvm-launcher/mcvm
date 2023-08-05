use std::{collections::HashMap, fmt::Display};

pub mod conditions;
pub mod instruction;
pub mod lex;
pub mod metadata;
pub mod parse;
pub mod properties;
pub mod routine;

use anyhow::{anyhow, bail, Context};

/// Argument to a command that could be constant or a variable
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
	None,
	Constant(String),
	Var(String),
}

impl Value {
	/// Replace substitution tokens in a string with variable values
	pub fn substitute_tokens(string: &str, vars: &HashMap<String, String>) -> String {
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
						if let Some(var) = vars.get(&name) {
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
	pub fn get(&self, vars: &HashMap<String, String>) -> anyhow::Result<String> {
		match self {
			Self::None => bail!("Empty value"),
			Self::Constant(val) => Ok(Self::substitute_tokens(val, vars)),
			Self::Var(name) => vars
				.get(name)
				.cloned()
				.ok_or(anyhow!("Variable {name} is not defined")),
		}
	}

	/// Returns whether this value is not none
	pub fn is_some(&self) -> bool {
		!matches!(self, Self::None)
	}

	/// Gets the current string value and converts to an option.
	pub fn get_as_option(&self, vars: &HashMap<String, String>) -> anyhow::Result<Option<String>> {
		match self {
			Self::None => Ok(None),
			_ => Ok(Some(self.get(vars)?)),
		}
	}
}

/// Reason why the package reported a failure
#[derive(Debug, Clone)]
pub enum FailReason {
	None,
	UnsupportedVersion,
	UnsupportedModloader,
	UnsupportedPluginLoader,
}

impl FailReason {
	pub fn from_string(string: &str) -> Option<Self> {
		match string {
			"unsupported_version" => Some(Self::UnsupportedVersion),
			"unsupported_modloader" => Some(Self::UnsupportedModloader),
			"unsupported_plugin_loader" => Some(Self::UnsupportedPluginLoader),
			_ => None,
		}
	}
}

impl Display for FailReason {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::None => "",
				Self::UnsupportedVersion => "Unsupported Minecraft version",
				Self::UnsupportedModloader => "Unsupported modloader",
				Self::UnsupportedPluginLoader => "Unsupported plugin loader",
			}
		)
	}
}

/// Parses and validates a package
pub fn parse_and_validate(contents: &str) -> anyhow::Result<()> {
	let parsed = parse::lex_and_parse(contents).context("Parsing failed")?;
	metadata::eval_metadata(&parsed).context("Metadata evaluation failed")?;
	properties::eval_properties(&parsed).context("Properties evaluation failed")?;

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_value_substitution() {
		let vars = {
			let mut vars = HashMap::new();
			vars.insert(String::from("bar"), String::from("foo"));
			vars.insert(String::from("hello"), String::from("who"));
			vars
		};

		let string = "One ${bar} skip a ${hello}";
		let string = Value::substitute_tokens(string, &vars);
		assert_eq!(string, "One foo skip a who");
	}
}
