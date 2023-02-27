use std::collections::HashMap;

pub mod lex;
pub mod parse;
pub mod instruction;
pub mod conditions;

// Argument to a command that could be constant or a variable
#[derive(Debug, Clone)]
pub enum Value {
	None,
	Constant(String),
	Var(String)
}

impl Value {
	pub fn get(&self, vars: &HashMap<String, String>) -> Option<String> {
		match self {
			Self::None => None,
			Self::Constant(val) => Some(val.clone()),
			Self::Var(name) => vars.get(name).cloned()
		}
	}
}
