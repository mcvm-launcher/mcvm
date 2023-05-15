use anyhow::bail;

use super::conditions::Condition;
use super::lex::{TextPos, Token};
use super::parse::BlockId;
use super::FailReason;
use super::Value;
use crate::unexpected_token;
use mcvm_shared::addon::AddonKind;

/// Type of an instruction
#[derive(Debug, Clone)]
pub enum InstrKind {
	If(Condition, BlockId),
	Name(Value),
	Version(Value),
	DefaultFeatures(Vec<Value>),
	Addon {
		id: Value,
		file_name: Value,
		kind: Option<AddonKind>,
		url: Value,
		force: bool,
		append: Value,
		path: Value,
	},
	Set(Option<String>, Value),
	Require(Vec<Vec<super::parse::require::Package>>),
	Refuse(Value),
	Recommend(Value),
	Bundle(Value),
	Finish(),
	Fail(Option<FailReason>),
}

/// A command / statement run in a package script
#[derive(Debug, Clone)]
pub struct Instruction {
	pub kind: InstrKind,
}

impl Instruction {
	pub fn new(kind: InstrKind) -> Self {
		Self { kind }
	}

	/// Starts an instruction from the provided string
	pub fn from_str(string: &str, pos: &TextPos) -> anyhow::Result<Self> {
		let kind = match string {
			"name" => Ok::<InstrKind, anyhow::Error>(InstrKind::Name(Value::None)),
			"version" => Ok(InstrKind::Version(Value::None)),
			"default_features" => Ok(InstrKind::DefaultFeatures(Vec::new())),
			"set" => Ok(InstrKind::Set(None, Value::None)),
			"finish" => Ok(InstrKind::Finish()),
			"fail" => Ok(InstrKind::Fail(None)),
			"refuse" => Ok(InstrKind::Refuse(Value::None)),
			"recommend" => Ok(InstrKind::Recommend(Value::None)),
			"bundle" => Ok(InstrKind::Bundle(Value::None)),
			string => bail!("Unknown instruction '{string}' {}", pos),
		}?;
		Ok(Instruction::new(kind))
	}

	/// Parses a token and returns true if finished
	pub fn parse(&mut self, tok: &Token, pos: &TextPos) -> anyhow::Result<bool> {
		if let Token::Semicolon = tok {
			Ok(true)
		} else {
			match &mut self.kind {
				InstrKind::Name(val)
				| InstrKind::Version(val)
				| InstrKind::Refuse(val)
				| InstrKind::Recommend(val)
				| InstrKind::Bundle(val) => *val = parse_arg(tok, pos)?,
				InstrKind::DefaultFeatures(features) => features.push(parse_arg(tok, pos)?),
				InstrKind::Set(var, val) => {
					if var.is_some() {
						*val = parse_arg(tok, pos)?;
					} else {
						match tok {
							Token::Ident(name) => *var = Some(name.clone()),
							_ => {
								unexpected_token!(tok, pos)
							}
						}
					}
				}
				InstrKind::Fail(reason) => match tok {
					Token::Ident(name) => {
						*reason = match FailReason::from_string(name) {
							Some(reason) => Some(reason),
							None => {
								bail!("Unknown fail reason '{}' {}", name.clone(), pos.clone());
							}
						}
					}
					_ => unexpected_token!(tok, pos),
				},
				_ => {}
			}

			Ok(false)
		}
	}
}

/// Parses a generic instruction argument with variable support
pub fn parse_arg(tok: &Token, pos: &TextPos) -> anyhow::Result<Value> {
	match tok {
		Token::Variable(name) => Ok(Value::Var(name.to_string())),
		Token::Str(text) => Ok(Value::Constant(text.clone())),
		Token::Num(num) => Ok(Value::Constant(num.to_string())),
		_ => unexpected_token!(tok, pos),
	}
}
