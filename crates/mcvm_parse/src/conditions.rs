use anyhow::bail;

use crate::unexpected_token;
use mcvm_shared::instance::Side;
use mcvm_shared::modifications::{ModloaderMatch, PluginLoaderMatch};

use super::instruction::parse_arg;
use super::lex::{TextPos, Token};
use super::Value;

/// Value for the OS condition
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OsCondition {
	Windows,
	Linux,
	Other,
}

impl OsCondition {
	pub fn parse_from_str(string: &str) -> Option<Self> {
		match string {
			"windows" => Some(Self::Windows),
			"linux" => Some(Self::Linux),
			"other" => Some(Self::Other),
			_ => None,
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionKind {
	Not(Option<Box<ConditionKind>>),
	And(Box<ConditionKind>, Option<Box<ConditionKind>>),
	Or(Box<ConditionKind>, Option<Box<ConditionKind>>),
	Version(Value),
	Side(Option<Side>),
	Modloader(Option<ModloaderMatch>),
	PluginLoader(Option<PluginLoaderMatch>),
	Feature(Value),
	Value(Value, Value),
	Defined(Value),
	Os(Option<OsCondition>),
}

impl ConditionKind {
	pub fn from_str(string: &str) -> Option<Self> {
		match string {
			"not" => Some(Self::Not(None)),
			"version" => Some(Self::Version(Value::None)),
			"side" => Some(Self::Side(None)),
			"modloader" => Some(Self::Modloader(None)),
			"plugin_loader" => Some(Self::PluginLoader(None)),
			"feature" => Some(Self::Feature(Value::None)),
			"value" => Some(Self::Value(Value::None, Value::None)),
			"defined" => Some(Self::Defined(Value::None)),
			"os" => Some(Self::Os(None)),
			_ => None,
		}
	}

	/// Checks whether this condition is finished parsing
	pub fn is_finished_parsing(&self) -> bool {
		match &self {
			Self::Not(condition) => {
				matches!(condition, Some(condition) if condition.is_finished_parsing())
			}
			Self::And(left, right) | Self::Or(left, right) => {
				left.is_finished_parsing()
					&& matches!(right, Some(condition) if condition.is_finished_parsing())
			}
			Self::Version(val) | Self::Feature(val) | Self::Defined(val) => val.is_some(),
			Self::Side(val) => val.is_some(),
			Self::Modloader(val) => val.is_some(),
			Self::PluginLoader(val) => val.is_some(),
			Self::Os(val) => val.is_some(),
			Self::Value(left, right) => left.is_some() && right.is_some(),
		}
	}

	/// Add arguments to the condition from tokens
	pub fn parse(&mut self, tok: &Token, pos: &TextPos) -> anyhow::Result<()> {
		match tok {
			Token::Ident(name) if name == "and" => {
				if self.is_finished_parsing() {
					let current = Box::new(self.clone());
					*self = ConditionKind::And(current, None);
					return Ok(());
				}
			}
			_ => {
				if self.is_finished_parsing() {
					unexpected_token!(tok, pos);
				}
			}
		}
		match self {
			Self::Not(condition) | Self::And(_, condition) | Self::Or(_, condition) => {
				match condition {
					Some(condition) => {
						return condition.parse(tok, pos);
					}
					None => match tok {
						Token::Ident(name) => match Self::from_str(name) {
							Some(nested_cond) => *condition = Some(Box::new(nested_cond)),
							None => {
								bail!("Unknown condition '{}' {}", name.clone(), pos.clone());
							}
						},
						_ => unexpected_token!(tok, pos),
					},
				}
			}
			Self::Version(val) | Self::Feature(val) | Self::Defined(val) => {
				*val = parse_arg(tok, pos)?;
			}
			Self::Side(side) => match tok {
				Token::Ident(name) => match Side::from_str(name) {
					Some(kind) => *side = Some(kind),
					None => {
						bail!(
							"Unknown condition argument '{}' {}",
							name.to_owned(),
							pos.clone()
						);
					}
				},
				_ => unexpected_token!(tok, pos),
			},
			Self::Modloader(loader) => match tok {
				Token::Ident(name) => match ModloaderMatch::from_str(name) {
					Some(kind) => *loader = Some(kind),
					None => {
						bail!(
							"Unknown condition argument '{}' {}",
							name.to_owned(),
							pos.clone()
						);
					}
				},
				_ => unexpected_token!(tok, pos),
			},
			Self::PluginLoader(loader) => match tok {
				Token::Ident(name) => match PluginLoaderMatch::from_str(name) {
					Some(kind) => *loader = Some(kind),
					None => {
						bail!(
							"Unknown condition argument '{}' {}",
							name.to_owned(),
							pos.clone()
						);
					}
				},
				_ => unexpected_token!(tok, pos),
			},
			Self::Os(os) => match tok {
				Token::Ident(name) => match OsCondition::parse_from_str(name) {
					Some(kind) => *os = Some(kind),
					None => {
						bail!(
							"Unknown condition argument '{}' {}",
							name.to_owned(),
							pos.clone()
						);
					}
				},
				_ => unexpected_token!(tok, pos),
			},
			Self::Value(left, right) => match left {
				Value::None => *left = parse_arg(tok, pos)?,
				_ => *right = parse_arg(tok, pos)?,
			},
		}
		Ok(())
	}
}

#[derive(Debug, Clone)]
pub struct Condition {
	pub kind: ConditionKind,
}

impl Condition {
	pub fn new(kind: ConditionKind) -> Self {
		Self { kind }
	}

	pub fn parse(&mut self, tok: &Token, pos: &TextPos) -> anyhow::Result<()> {
		self.kind.parse(tok, pos)?;
		Ok(())
	}
}
