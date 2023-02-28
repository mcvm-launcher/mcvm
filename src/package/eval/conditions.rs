use crate::data::asset::Modloader;
use crate::data::instance::InstKind;
use crate::util::versions::MinecraftVersion;

use super::Value;
use super::eval::{EvalConstants, EvalResult, EvalError, EvalData};
use super::lex::{Token, TextPos};
use super::parse::ParseError;
use super::instruction::{parse_arg, ParseArgResult};

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ConditionKind {
	Not(Option<Box<ConditionKind>>),
	Version(Value),
	Side(Option<InstKind>),
	Modloader(Option<Modloader>)
}

impl ConditionKind {
	pub fn from_str(string: &str) -> Option<Self> {
		match string {
			"not" => Some(Self::Not(None)),
			"version" => Some(Self::Version(Value::None)),
			"side" => Some(Self::Side(None)),
			"modloader" => Some(Self::Modloader(None)),
			_ => None
		}
	}
	
	pub fn parse(&mut self, tok: &Token, pos: &TextPos, parse_var: bool) -> Result<bool, ParseError> {
		match self {
			ConditionKind::Not(condition) => {
				match condition {
					Some(condition) => {
						return condition.parse(tok, pos, parse_var);
					}
					None => match tok {
						Token::Ident(name) => match ConditionKind::from_str(name) {
							Some(nested_cond) => *condition = Some(Box::new(nested_cond)),
							None => {}
						},
						_ => return Err(ParseError::UnexpectedToken(tok.as_string(), pos.clone()))
					}
				}

				Ok(false)
			}
			ConditionKind::Version(val) => {
				match parse_arg(tok, pos, parse_var)? {
					ParseArgResult::ParseVar => Ok(true),
					ParseArgResult::Value(new_val) => {
						*val = new_val;
						Ok(false)
					}
				}
			}
			ConditionKind::Side(side) => {
				match tok {
					Token::Ident(name) => match InstKind::from_str(name) {
						Some(kind) => *side = Some(kind),
						None => {}
					}
					_ => return Err(ParseError::UnexpectedToken(tok.as_string(), pos.clone()))
				}
				Ok(false)
			}
			ConditionKind::Modloader(loader) => {
				match tok {
					Token::Ident(name) => match Modloader::from_str(name) {
						Some(kind) => *loader = Some(kind),
						None => {}
					}
					_ => return Err(ParseError::UnexpectedToken(tok.as_string(), pos.clone()))
				}
				Ok(false)
			}
		}
	}

	pub fn eval(&self, constants: &EvalConstants, eval: &EvalData)
	-> Result<bool, EvalError> {
		match self {
			Self::Not(condition) => {
				condition.as_ref().expect("Not condition is missing").eval(constants, eval)
			}
			Self::Version(version) => {
				let version = version.get(&eval.vars)?;
				match &constants.version {
					MinecraftVersion::Unknown(other) => Ok(version == *other)
				}
			}
			Self::Side(side) => {
				Ok(constants.side == *side.as_ref().expect("If side is missing"))
			}
			Self::Modloader(modloader) => {
				Ok(constants.modloader == *modloader.as_ref().expect("If modloader is missing"))
			}
		}
	}
}

#[derive(Debug, Clone)]
pub struct Condition {
	pub kind: ConditionKind,
	parse_var: bool
}

impl Condition {
	pub fn new(kind: ConditionKind) -> Self {
		Self {
			kind,
			parse_var: false
		}
	}
	
	pub fn parse(&mut self, tok: &Token, pos: &TextPos) -> Result<(), ParseError> {
		self.parse_var = self.kind.parse(tok, pos, self.parse_var)?;
		Ok(())
	}
}
