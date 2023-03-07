use crate::data::addon::{ModloaderMatch, PluginLoaderMatch};
use crate::data::instance::InstKind;
use crate::util::versions::VersionPattern;

use super::Value;
use super::eval::{EvalError, EvalData};
use super::lex::{Token, TextPos};
use super::parse::ParseError;
use super::instruction::parse_arg;

#[derive(Debug, Clone)]
pub enum ConditionKind {
	Not(Option<Box<ConditionKind>>),
	Version(Value),
	Side(Option<InstKind>),
	Modloader(Option<ModloaderMatch>),
	PluginLoader(Option<PluginLoaderMatch>),
	Feature(Value),
	Value(Value, Value)
}

impl ConditionKind {
	pub fn from_str(string: &str) -> Option<Self> {
		match string {
			"not" => Some(Self::Not(None)),
			"version" => Some(Self::Version(Value::None)),
			"side" => Some(Self::Side(None)),
			"modloader" => Some(Self::Modloader(None)),
			"pluginloader" => Some(Self::PluginLoader(None)),
			"feature" => Some(Self::Feature(Value::None)),
			"value" => Some(Self::Value(Value::None, Value::None)),
			_ => None
		}
	}
	
	pub fn parse(&mut self, tok: &Token, pos: &TextPos) -> Result<(), ParseError> {
		match self {
			Self::Not(condition) => match condition {
				Some(condition) => {
					return condition.parse(tok, pos);
				}
				None => match tok {
					Token::Ident(name) => match Self::from_str(name) {
						Some(nested_cond) => *condition = Some(Box::new(nested_cond)),
						None => return Err(ParseError::UnknownCondition(name.clone(), pos.clone()))
					},
					_ => return Err(ParseError::UnexpectedToken(tok.as_string(), pos.clone()))
				}
			}
			Self::Version(val) |
			Self::Feature(val) => *val = parse_arg(tok, pos)?,
			Self::Side(side) => match tok {
				Token::Ident(name) => match InstKind::from_str(name) {
					Some(kind) => *side = Some(kind),
					None => {}
				}
				_ => return Err(ParseError::UnexpectedToken(tok.as_string(), pos.clone()))
			}
			Self::Modloader(loader) => match tok {
				Token::Ident(name) => match ModloaderMatch::from_str(name) {
					Some(kind) => *loader = Some(kind),
					None => {}
				}
				_ => return Err(ParseError::UnexpectedToken(tok.as_string(), pos.clone()))
			}
			Self::PluginLoader(loader) => match tok {
				Token::Ident(name) => match PluginLoaderMatch::from_str(name) {
					Some(kind) => *loader = Some(kind),
					None => {}
				}
				_ => return Err(ParseError::UnexpectedToken(tok.as_string(), pos.clone()))
			}
			Self::Value(left, right) => match left {
				Value::None => *left = parse_arg(tok, pos)?,
				_ => *right = parse_arg(tok, pos)?
			}
		}
		Ok(())
	}

	pub fn eval(&self, eval: &EvalData)
	-> Result<bool, EvalError> {
		match self {
			Self::Not(condition) => {
				condition.as_ref().expect("Not condition is missing").eval(eval).map(|op| !op)
			}
			Self::Version(version) => {
				let version = version.get(&eval.vars)?;
				let version = VersionPattern::from(&version);
				Ok(version.matches_single(&eval.constants.version, &eval.constants.versions))
			}
			Self::Side(side) => {
				Ok(eval.constants.side == *side.as_ref().expect("If side is missing"))
			}
			Self::Modloader(loader) => {
				Ok(loader.as_ref().expect("If modloader is missing").matches(&eval.constants.modloader))
			}
			Self::PluginLoader(loader) => {
				Ok(loader.as_ref().expect("If pluginloader is missing").matches(&eval.constants.pluginloader))
			}
			Self::Feature(feature) => {
				Ok(eval.constants.features.contains(&feature.get(&eval.vars)?))
			}
			Self::Value(left, right) => {
				Ok(left.get(&eval.vars)? == right.get(&eval.vars)?)
			}
		}
	}
}

#[derive(Debug, Clone)]
pub struct Condition {
	pub kind: ConditionKind
}

impl Condition {
	pub fn new(kind: ConditionKind) -> Self {
		Self {
			kind
		}
	}
	
	pub fn parse(&mut self, tok: &Token, pos: &TextPos) -> Result<(), ParseError> {
		self.kind.parse(tok, pos)?;
		Ok(())
	}
}
