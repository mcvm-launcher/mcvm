use super::Value;
use super::eval::FailReason;
use super::lex::{Token, TextPos, Side};
use super::parse::{BlockId, ParseError};
use super::conditions::Condition;
use crate::data::addon::AddonKind;

#[derive(Debug, Clone)]
pub enum InstrKind {
	If(Condition, BlockId),
	Name(Value),
	Version(Value),
	DefaultFeatures(Vec<Value>),
	Addon {
		name: Value,
		kind: Option<AddonKind>,
		url: Value,
		force: bool
	},
	Set(Option<String>, Value),
	Rely(Vec<Vec<Value>>, Option<Vec<Value>>),
	Finish(),
	Fail(Option<FailReason>)
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
			"fail" => Ok(InstrKind::Fail(None)),
			"rely" => Ok(InstrKind::Rely(Vec::new(), None)),
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
				InstrKind::Name(val)
				| InstrKind::Version(val) => *val = parse_arg(tok, pos)?,
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
				InstrKind::Fail(reason) => match tok {
					Token::Ident(name) => *reason = match FailReason::from_string(name) {
						Some(reason) => Some(reason),
						None => return Err(ParseError::UnknownReason(name.clone(), pos.clone()))
					},
					_ => return Err(ParseError::UnexpectedToken(tok.as_string(), pos.clone()))
				}
				InstrKind::Rely(deps, dep) => match tok {
					Token::Paren(Side::Left) => match dep {
						Some(..) => return Err(ParseError::UnexpectedToken(tok.as_string(), pos.clone())),
						None => *dep = Some(Vec::new())
					}
					Token::Paren(Side::Right) => match dep {
						Some(..) => deps.push(dep.take().expect("Dependency in option missing when pushing")),
						None => return Err(ParseError::UnexpectedToken(tok.as_string(), pos.clone()))
					}
					_ => {
						let val = parse_arg(tok, pos)?;
						match dep {
							Some(dep) => dep.push(val),
							None => deps.push(vec![val])
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
		Token::Num(num) => Ok(Value::Constant(num.to_string())),
		_ => Err(ParseError::UnexpectedToken(tok.as_string(), pos.clone()))
	}
}
