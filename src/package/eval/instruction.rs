use anyhow::bail;

use super::conditions::Condition;
use super::eval::FailReason;
use super::lex::{Side, TextPos, Token};
use super::parse::BlockId;
use super::Value;
use crate::data::addon::AddonKind;
use crate::unexpected_token;

/// Type of an instruction
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
		force: bool,
		append: Value,
		path: Value,
	},
	Set(Option<String>, Value),
	Rely(Vec<Vec<Value>>, Option<Vec<Value>>),
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
			"rely" => Ok(InstrKind::Rely(Vec::new(), None)),
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
				InstrKind::Name(val) | InstrKind::Version(val) => *val = parse_arg(tok, pos)?,
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
				InstrKind::Rely(deps, dep) => match tok {
					Token::Paren(Side::Left) => match dep {
						Some(..) => {
							unexpected_token!(tok, pos);
						}
						None => *dep = Some(Vec::new()),
					},
					Token::Paren(Side::Right) => match dep {
						Some(..) => deps.push(
							dep.take()
								.expect("Dependency in option missing when pushing"),
						),
						None => {
							unexpected_token!(tok, pos);
						}
					},
					_ => {
						let val = parse_arg(tok, pos)?;
						match dep {
							Some(dep) => dep.push(val),
							None => deps.push(vec![val]),
						}
					}
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
