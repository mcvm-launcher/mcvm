use super::{parse::{BlockId, ParseError}, lex::{Token, TextPos}, Value};
use super::conditions::Condition;
use crate::data::asset::AssetKind;

#[derive(Debug, Clone)]
pub enum InstrKind {
	If(Condition, BlockId),
	Name(Value),
	Version(Value),
	DefaultFeatures(Vec<Value>),
	Asset {
		name: Value,
		kind: Option<AssetKind>,
		url: Value
	},
	Set(Option<String>, Value),
	Finish(),
	Fail()
}

#[derive(Debug, Clone)]
pub struct Instruction {
	pub kind: InstrKind
}

impl Instruction {
	pub fn new(kind: InstrKind) -> Self {
		Self {
			kind
		}
	}

	pub fn from_str(string: &str, pos: &TextPos) -> Result<Self, ParseError> {
		let kind = match string {
			"name" => Ok(InstrKind::Name(Value::None)),
			"version" => Ok(InstrKind::Version(Value::None)),
			"default_features" => Ok(InstrKind::DefaultFeatures(Vec::new())),
			"set" => Ok(InstrKind::Set(None, Value::None)),
			"finish" => Ok(InstrKind::Finish()),
			"fail" => Ok(InstrKind::Fail()),
			string => Err(ParseError::UnknownInstr(string.to_owned(), pos.clone()))
		}?;
		Ok(Instruction::new(kind))
	}

	// Parses a token and returns true if finished
	pub fn parse(&mut self, tok: &Token, pos: &TextPos) -> Result<bool, ParseError> {
		if let Token::Semicolon = tok {
			Ok(true)
		} else {
			match &mut self.kind {
				InstrKind::Name(val) |
				InstrKind::Version(val) => *val = parse_arg(tok, pos)?,
				InstrKind::DefaultFeatures(features) => features.push(parse_arg(tok, pos)?),
				InstrKind::Set(var, val) => {
					if var.is_some() {
						*val = parse_arg(tok, pos)?;
					} else {
						match tok {
							Token::Ident(name) => *var = Some(name.clone()),
							_ => return Err(ParseError::UnexpectedToken(tok.as_string(), pos.clone()))
						}
					}
				}
				_ => {}
			}

			Ok(false)
		}
	}
}

// Parses a generic instruction argument
pub fn parse_arg(tok: &Token, pos: &TextPos) -> Result<Value, ParseError> {
	match tok {
		Token::Variable(name) => Ok(Value::Var(name.to_string())),
		Token::Str(text) => Ok(Value::Constant(text.clone())),
		Token::Num(num) => Ok(Value::Constant(num.to_string().clone())),
		_ => Err(ParseError::UnexpectedToken(tok.as_string(), pos.clone()))
	}
}
