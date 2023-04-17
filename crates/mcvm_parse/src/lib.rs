use std::{collections::HashMap, fmt::Display};

pub mod conditions;
pub mod instruction;
pub mod lex;
pub mod parse;

use anyhow::{anyhow, bail};

/// Argument to a command that could be constant or a variable
#[derive(Debug, Clone)]
pub enum Value {
	None,
	Constant(String),
	Var(String),
}

impl Value {
	/// Obtain the current String value of this Value.
	/// Will fail if it is none or the variable is uninitialized.
	pub fn get(&self, vars: &HashMap<String, String>) -> anyhow::Result<String> {
		match self {
			Self::None => bail!("Empty value"),
			Self::Constant(val) => Ok(val.clone()),
			Self::Var(name) => vars
				.get(name)
				.cloned()
				.ok_or(anyhow!("Variable {name} is not defined")),
		}
	}
}

/// Reason why the package reported a failure
#[derive(Debug, Clone)]
pub enum FailReason {
	None,
	UnsupportedVersion,
	UnsupportedModloader,
}

impl FailReason {
	pub fn from_string(string: &str) -> Option<Self> {
		match string {
			"unsupported_version" => Some(Self::UnsupportedVersion),
			"unsupported_modloader" => Some(Self::UnsupportedModloader),
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
			}
		)
	}
}
