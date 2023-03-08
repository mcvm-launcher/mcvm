use std::collections::HashMap;

pub mod conditions;
pub mod eval;
pub mod instruction;
pub mod lex;
pub mod parse;

use eval::EvalError;

// Argument to a command that could be constant or a variable
#[derive(Debug, Clone)]
pub enum Value {
	None,
	Constant(String),
	Var(String),
}

impl Value {
	pub fn get(&self, vars: &HashMap<String, String>) -> Result<String, EvalError> {
		match self {
			Self::None => Err(EvalError::VarNotDefined(String::from(""))),
			Self::Constant(val) => Ok(val.clone()),
			Self::Var(name) => vars
				.get(name)
				.cloned()
				.ok_or(EvalError::VarNotDefined(name.clone())),
		}
	}
}
