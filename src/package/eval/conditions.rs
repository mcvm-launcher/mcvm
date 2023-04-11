use anyhow::bail;

use crate::data::addon::{ModloaderMatch, PluginLoaderMatch};
use crate::data::instance::Side;
use crate::unexpected_token;
use crate::util::versions::VersionPattern;

use super::eval::EvalData;
use super::instruction::parse_arg;
use super::lex::{TextPos, Token};
use super::Value;

#[derive(Debug, Clone)]
pub enum ConditionKind {
	Not(Option<Box<ConditionKind>>),
	Version(Value),
	Side(Option<Side>),
	Modloader(Option<ModloaderMatch>),
	PluginLoader(Option<PluginLoaderMatch>),
	Feature(Value),
	Value(Value, Value),
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
			_ => None,
		}
	}

	/// Add arguments to the condition from tokens
	pub fn parse(&mut self, tok: &Token, pos: &TextPos) -> anyhow::Result<()> {
		match self {
			Self::Not(condition) => match condition {
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
			},
			Self::Version(val) | Self::Feature(val) => *val = parse_arg(tok, pos)?,
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
			Self::Value(left, right) => match left {
				Value::None => *left = parse_arg(tok, pos)?,
				_ => *right = parse_arg(tok, pos)?,
			},
		}
		Ok(())
	}

	pub fn eval(&self, eval: &EvalData) -> anyhow::Result<bool> {
		match self {
			Self::Not(condition) => condition
				.as_ref()
				.expect("Not condition is missing")
				.eval(eval)
				.map(|op| !op),
			Self::Version(version) => {
				let version = version.get(&eval.vars)?;
				let version = VersionPattern::from(&version);
				Ok(version.matches_single(&eval.constants.version, &eval.constants.versions))
			}
			Self::Side(side) => {
				Ok(eval.constants.side == *side.as_ref().expect("If side is missing"))
			}
			Self::Modloader(loader) => Ok(loader
				.as_ref()
				.expect("If modloader is missing")
				.matches(&eval.constants.modloader)),
			Self::PluginLoader(loader) => Ok(loader
				.as_ref()
				.expect("If plugin_loader is missing")
				.matches(&eval.constants.plugin_loader)),
			Self::Feature(feature) => {
				Ok(eval.constants.features.contains(&feature.get(&eval.vars)?))
			}
			Self::Value(left, right) => Ok(left.get(&eval.vars)? == right.get(&eval.vars)?),
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
