use std::collections::HashMap;

pub mod conditions;
pub mod eval;
pub mod instruction;
pub mod lex;
pub mod parse;

use anyhow::{anyhow, bail};

// Argument to a command that could be constant or a variable
#[derive(Debug, Clone)]
pub enum Value {
	None,
	Constant(String),
	Var(String),
}

impl Value {
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
