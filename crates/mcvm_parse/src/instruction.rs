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
	Name(Option<String>),
	Description(Option<String>),
	LongDescription(Option<String>),
	Version(Option<String>),
	Authors(Vec<String>),
	PackageMaintainers(Vec<String>),
	Website(Option<String>),
	SupportLink(Option<String>),
	Documentation(Option<String>),
	Source(Option<String>),
	Issues(Option<String>),
	Community(Option<String>),
	Icon(Option<String>),
	Banner(Option<String>),
	License(Option<String>),
	Features(Vec<String>),
	DefaultFeatures(Vec<String>),
	Addon {
		id: Value,
		file_name: Value,
		kind: Option<AddonKind>,
		url: Value,
		path: Value,
		version: Value,
	},
	Set(Option<String>, Value),
	Require(Vec<Vec<super::parse::require::Package>>),
	Refuse(Value),
	Recommend(Value),
	Bundle(Value),
	Compat(Value, Value),
	Extend(Value),
	Finish(),
	Fail(Option<FailReason>),
	Notice(Value),
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
			"name" => Ok::<InstrKind, anyhow::Error>(InstrKind::Name(None)),
			"description" => Ok(InstrKind::Description(None)),
			"long_description" => Ok(InstrKind::LongDescription(None)),
			"version" => Ok(InstrKind::Version(None)),
			"authors" => Ok(InstrKind::Authors(Vec::new())),
			"package_maintainers" => Ok(InstrKind::PackageMaintainers(Vec::new())),
			"website" => Ok(InstrKind::Website(None)),
			"support_link" => Ok(InstrKind::SupportLink(None)),
			"documentation" => Ok(InstrKind::Documentation(None)),
			"source" => Ok(InstrKind::Source(None)),
			"issues" => Ok(InstrKind::Issues(None)),
			"community" => Ok(InstrKind::Community(None)),
			"icon" => Ok(InstrKind::Icon(None)),
			"banner" => Ok(InstrKind::Banner(None)),
			"license" => Ok(InstrKind::License(None)),
			"features" => Ok(InstrKind::Features(Vec::new())),
			"default_features" => Ok(InstrKind::DefaultFeatures(Vec::new())),
			"set" => Ok(InstrKind::Set(None, Value::None)),
			"finish" => Ok(InstrKind::Finish()),
			"fail" => Ok(InstrKind::Fail(None)),
			"refuse" => Ok(InstrKind::Refuse(Value::None)),
			"recommend" => Ok(InstrKind::Recommend(Value::None)),
			"bundle" => Ok(InstrKind::Bundle(Value::None)),
			"compat" => Ok(InstrKind::Compat(Value::None, Value::None)),
			"extend" => Ok(InstrKind::Extend(Value::None)),
			"notice" => Ok(InstrKind::Notice(Value::None)),
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
				InstrKind::Name(text)
				| InstrKind::Description(text)
				| InstrKind::LongDescription(text)
				| InstrKind::Version(text)
				| InstrKind::Website(text)
				| InstrKind::SupportLink(text)
				| InstrKind::Documentation(text)
				| InstrKind::Source(text)
				| InstrKind::Issues(text)
				| InstrKind::Community(text)
				| InstrKind::Icon(text)
				| InstrKind::Banner(text)
				| InstrKind::License(text) => {
					if text.is_none() {
						*text = Some(parse_string(tok, pos)?);
					} else {
						unexpected_token!(tok, pos);
					}
				}
				InstrKind::Refuse(val)
				| InstrKind::Recommend(val)
				| InstrKind::Bundle(val)
				| InstrKind::Notice(val)
				| InstrKind::Extend(val) => {
					if let Value::None = val {
						*val = parse_arg(tok, pos)?;
					} else {
						unexpected_token!(tok, pos);
					}
				}
				InstrKind::Authors(list)
				| InstrKind::PackageMaintainers(list)
				| InstrKind::Features(list)
				| InstrKind::DefaultFeatures(list) => list.push(parse_string(tok, pos)?),
				InstrKind::Compat(package, compat) => {
					if let Value::None = package {
						*package = parse_arg(tok, pos)?;
					} else if let Value::None = compat {
						*compat = parse_arg(tok, pos)?;
					} else {
						unexpected_token!(tok, pos);
					}
				}
				InstrKind::Set(var, val) => {
					if var.is_some() {
						if let Value::None = val {
							*val = parse_arg(tok, pos)?;
						} else {
							unexpected_token!(tok, pos);
						}
					} else {
						match tok {
							Token::Ident(name) => *var = Some(name.clone()),
							_ => unexpected_token!(tok, pos),
						}
					}
				}
				InstrKind::Fail(reason) => match tok {
					Token::Ident(name) => {
						if reason.is_none() {
							*reason = match FailReason::from_string(name) {
								Some(reason) => Some(reason),
								None => {
									bail!("Unknown fail reason '{}' {}", name.clone(), pos.clone());
								}
							}
						} else {
							unexpected_token!(tok, pos);
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

/// Parses a constant string argument
pub fn parse_string(tok: &Token, pos: &TextPos) -> anyhow::Result<String> {
	match tok {
		Token::Str(text) => Ok(text.clone()),
		_ => unexpected_token!(tok, pos),
	}
}
