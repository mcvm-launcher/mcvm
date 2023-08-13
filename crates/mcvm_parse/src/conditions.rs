use anyhow::bail;
use mcvm_shared::lang::Language;
use mcvm_shared::later::Later;
use mcvm_shared::pkg::PackageStability;
use serde::Deserialize;

use crate::unexpected_token;
use mcvm_shared::instance::Side;
use mcvm_shared::modifications::{ModloaderMatch, PluginLoaderMatch};

use super::instruction::parse_arg;
use super::lex::{TextPos, Token};
use super::Value;

/// Value for the OS condition
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OSCondition {
	Windows,
	Linux,
	MacOS,
	Other,
}

impl OSCondition {
	pub fn parse_from_str(string: &str) -> Option<Self> {
		match string {
			"windows" => Some(Self::Windows),
			"linux" => Some(Self::Linux),
			"macos" => Some(Self::MacOS),
			"other" => Some(Self::Other),
			_ => None,
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionKind {
	Not(Later<Box<ConditionKind>>),
	And(Box<ConditionKind>, Later<Box<ConditionKind>>),
	Or(Box<ConditionKind>, Later<Box<ConditionKind>>),
	Version(Value),
	Side(Later<Side>),
	Modloader(Later<ModloaderMatch>),
	PluginLoader(Later<PluginLoaderMatch>),
	Feature(Value),
	Value(Value, Value),
	Defined(Later<String>),
	OS(Later<OSCondition>),
	Stability(Later<PackageStability>),
	Language(Later<Language>),
}

impl ConditionKind {
	pub fn parse_from_str(string: &str) -> Option<Self> {
		match string {
			"not" => Some(Self::Not(Later::Empty)),
			"version" => Some(Self::Version(Value::None)),
			"side" => Some(Self::Side(Later::Empty)),
			"modloader" => Some(Self::Modloader(Later::Empty)),
			"plugin_loader" => Some(Self::PluginLoader(Later::Empty)),
			"feature" => Some(Self::Feature(Value::None)),
			"value" => Some(Self::Value(Value::None, Value::None)),
			"defined" => Some(Self::Defined(Later::Empty)),
			"os" => Some(Self::OS(Later::Empty)),
			"stability" => Some(Self::Stability(Later::Empty)),
			_ => None,
		}
	}

	/// Checks whether this condition is finished parsing
	pub fn is_finished_parsing(&self) -> bool {
		match &self {
			Self::Not(condition) => {
				matches!(condition, Later::Full(condition) if condition.is_finished_parsing())
			}
			Self::And(left, right) | Self::Or(left, right) => {
				left.is_finished_parsing()
					&& matches!(right, Later::Full(condition) if condition.is_finished_parsing())
			}
			Self::Version(val) | Self::Feature(val) => val.is_some(),
			Self::Side(val) => val.is_full(),
			Self::Modloader(val) => val.is_full(),
			Self::PluginLoader(val) => val.is_full(),
			Self::Defined(val) => val.is_full(),
			Self::OS(val) => val.is_full(),
			Self::Stability(val) => val.is_full(),
			Self::Language(val) => val.is_full(),
			Self::Value(left, right) => left.is_some() && right.is_some(),
		}
	}

	/// Add arguments to the condition from tokens
	pub fn parse(&mut self, tok: &Token, pos: &TextPos) -> anyhow::Result<()> {
		match tok {
			Token::Ident(name) => {
				if self.is_finished_parsing() {
					let current = Box::new(self.clone());
					match name.as_str() {
						"and" => *self = ConditionKind::And(current, Later::Empty),
						"or" => *self = ConditionKind::Or(current, Later::Empty),
						_ => bail!("Unknown condition combinator '{name}'"),
					}
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
					Later::Full(condition) => {
						return condition.parse(tok, pos);
					}
					Later::Empty => match tok {
						Token::Ident(name) => match Self::parse_from_str(name) {
							Some(nested_cond) => condition.fill(Box::new(nested_cond)),
							None => {
								bail!("Unknown condition '{}' {}", name.clone(), pos.clone());
							}
						},
						_ => unexpected_token!(tok, pos),
					},
				}
			}
			Self::Version(val) | Self::Feature(val) => {
				*val = parse_arg(tok, pos)?;
			}
			Self::Defined(var) => match tok {
				Token::Ident(name) => var.fill(name.clone()),
				_ => unexpected_token!(tok, pos),
			},
			Self::Side(side) => match tok {
				Token::Ident(name) => side.fill(check_enum_condition_argument(
					Side::parse_from_str(name),
					name,
					pos,
				)?),
				_ => unexpected_token!(tok, pos),
			},
			Self::Modloader(loader) => match tok {
				Token::Ident(name) => loader.fill(check_enum_condition_argument(
					ModloaderMatch::parse_from_str(name),
					name,
					pos,
				)?),
				_ => unexpected_token!(tok, pos),
			},
			Self::PluginLoader(loader) => match tok {
				Token::Ident(name) => loader.fill(check_enum_condition_argument(
					PluginLoaderMatch::parse_from_str(name),
					name,
					pos,
				)?),
				_ => unexpected_token!(tok, pos),
			},
			Self::OS(os) => match tok {
				Token::Ident(name) => os.fill(check_enum_condition_argument(
					OSCondition::parse_from_str(name),
					name,
					pos,
				)?),
				_ => unexpected_token!(tok, pos),
			},
			Self::Stability(stability) => match tok {
				Token::Ident(name) => stability.fill(check_enum_condition_argument(
					PackageStability::parse_from_str(name),
					name,
					pos,
				)?),
				_ => unexpected_token!(tok, pos),
			},
			Self::Language(lang) => match tok {
				Token::Ident(name) => lang.fill(check_enum_condition_argument(
					Language::parse_from_str(name),
					name,
					pos,
				)?),
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

/// Check the parsing of a condition argument
fn check_enum_condition_argument<T>(
	arg: Option<T>,
	ident: &str,
	pos: &TextPos,
) -> anyhow::Result<T> {
	match arg {
		Some(val) => Ok(val),
		None => {
			bail!(
				"Unknown condition argument '{}' {}",
				ident.to_owned(),
				pos.clone()
			);
		}
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
