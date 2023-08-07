use std::fmt::Display;

use anyhow::bail;
use mcvm_shared::instance::Side;
use mcvm_shared::later::Later;
use mcvm_shared::modifications::{ModloaderMatch, PluginLoaderMatch};

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
	Name(Later<String>),
	Description(Later<String>),
	LongDescription(Later<String>),
	Version(Later<String>),
	Authors(Vec<String>),
	PackageMaintainers(Vec<String>),
	Website(Later<String>),
	SupportLink(Later<String>),
	Documentation(Later<String>),
	Source(Later<String>),
	Issues(Later<String>),
	Community(Later<String>),
	Icon(Later<String>),
	Banner(Later<String>),
	License(Later<String>),
	Features(Vec<String>),
	DefaultFeatures(Vec<String>),
	ModrinthID(Later<String>),
	CurseForgeID(Later<String>),
	SupportedModloaders(Vec<ModloaderMatch>),
	SupportedPluginLoaders(Vec<PluginLoaderMatch>),
	SupportedSides(Vec<Side>),
	Addon {
		id: Value,
		file_name: Value,
		kind: Option<AddonKind>,
		url: Value,
		path: Value,
		version: Value,
	},
	Set(Later<String>, Value),
	Require(Vec<Vec<super::parse::require::Package>>),
	Refuse(Value),
	Recommend(Value),
	Bundle(Value),
	Compat(Value, Value),
	Extend(Value),
	Finish(),
	Fail(Option<FailReason>),
	Notice(Value),
	Cmd(Vec<Value>),
}

impl Display for InstrKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::If(..) => "if",
				Self::Name(..) => "name",
				Self::Description(..) => "description",
				Self::LongDescription(..) => "long_description",
				Self::Version(..) => "version",
				Self::Authors(..) => "authors",
				Self::PackageMaintainers(..) => "package_maintainers",
				Self::Website(..) => "website",
				Self::SupportLink(..) => "support_link",
				Self::Documentation(..) => "documentation",
				Self::Source(..) => "source",
				Self::Issues(..) => "issues",
				Self::Community(..) => "community",
				Self::Icon(..) => "icon",
				Self::Banner(..) => "banner",
				Self::License(..) => "license",
				Self::Features(..) => "features",
				Self::DefaultFeatures(..) => "default_features",
				Self::ModrinthID(..) => "modrinth_id",
				Self::CurseForgeID(..) => "curseforge_id",
				Self::SupportedModloaders(..) => "supported_modloaders",
				Self::SupportedPluginLoaders(..) => "supported_plugin_loaders",
				Self::SupportedSides(..) => "supported_sides",
				Self::Addon { .. } => "addon",
				Self::Set(..) => "set",
				Self::Require(..) => "require",
				Self::Refuse(..) => "refuse",
				Self::Recommend(..) => "recommend",
				Self::Bundle(..) => "bundle",
				Self::Compat(..) => "compat",
				Self::Extend(..) => "extend",
				Self::Finish() => "finish",
				Self::Fail(..) => "fail",
				Self::Notice(..) => "notice",
				Self::Cmd(..) => "cmd",
			}
		)
	}
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
			"name" => Ok::<InstrKind, anyhow::Error>(InstrKind::Name(Later::Empty)),
			"description" => Ok(InstrKind::Description(Later::Empty)),
			"long_description" => Ok(InstrKind::LongDescription(Later::Empty)),
			"version" => Ok(InstrKind::Version(Later::Empty)),
			"authors" => Ok(InstrKind::Authors(Vec::new())),
			"package_maintainers" => Ok(InstrKind::PackageMaintainers(Vec::new())),
			"website" => Ok(InstrKind::Website(Later::Empty)),
			"support_link" => Ok(InstrKind::SupportLink(Later::Empty)),
			"documentation" => Ok(InstrKind::Documentation(Later::Empty)),
			"source" => Ok(InstrKind::Source(Later::Empty)),
			"issues" => Ok(InstrKind::Issues(Later::Empty)),
			"community" => Ok(InstrKind::Community(Later::Empty)),
			"icon" => Ok(InstrKind::Icon(Later::Empty)),
			"banner" => Ok(InstrKind::Banner(Later::Empty)),
			"license" => Ok(InstrKind::License(Later::Empty)),
			"features" => Ok(InstrKind::Features(Vec::new())),
			"default_features" => Ok(InstrKind::DefaultFeatures(Vec::new())),
			"modrinth_id" => Ok(InstrKind::ModrinthID(Later::Empty)),
			"curseforge_id" => Ok(InstrKind::CurseForgeID(Later::Empty)),
			"supported_modloaders" => Ok(InstrKind::SupportedModloaders(Vec::new())),
			"supported_plugin_loaders" => Ok(InstrKind::SupportedPluginLoaders(Vec::new())),
			"supported_sides" => Ok(InstrKind::SupportedSides(Vec::new())),
			"set" => Ok(InstrKind::Set(Later::Empty, Value::None)),
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

	/// Checks if this instruction is finished parsing
	/// Only works for simple instructions. Will panic for instructions with special parse modes
	pub fn is_finished_parsing(&self) -> bool {
		match &self.kind {
			InstrKind::Name(val)
			| InstrKind::Description(val)
			| InstrKind::LongDescription(val)
			| InstrKind::Version(val)
			| InstrKind::SupportLink(val)
			| InstrKind::Documentation(val)
			| InstrKind::Source(val)
			| InstrKind::Issues(val)
			| InstrKind::Community(val)
			| InstrKind::Icon(val)
			| InstrKind::Banner(val)
			| InstrKind::License(val)
			| InstrKind::ModrinthID(val)
			| InstrKind::CurseForgeID(val)
			| InstrKind::Website(val) => val.is_full(),
			InstrKind::Features(val)
			| InstrKind::Authors(val)
			| InstrKind::PackageMaintainers(val)
			| InstrKind::DefaultFeatures(val) => !val.is_empty(),
			InstrKind::Refuse(val)
			| InstrKind::Recommend(val)
			| InstrKind::Bundle(val)
			| InstrKind::Extend(val)
			| InstrKind::Notice(val) => val.is_some(),
			InstrKind::SupportedModloaders(val) => !val.is_empty(),
			InstrKind::SupportedPluginLoaders(val) => !val.is_empty(),
			InstrKind::SupportedSides(val) => !val.is_empty(),
			InstrKind::Compat(val1, val2) => val1.is_some() && val2.is_some(),
			InstrKind::Set(var, val) => var.is_full() && val.is_some(),
			InstrKind::Cmd(list) => !list.is_empty(),
			InstrKind::Fail(..) | InstrKind::Finish() => true,
			InstrKind::If(..) | InstrKind::Addon { .. } | InstrKind::Require(..) => {
				unimplemented!()
			}
		}
	}

	/// Parses a token and returns true if finished.
	pub fn parse(&mut self, tok: &Token, pos: &TextPos) -> anyhow::Result<bool> {
		if let Token::Semicolon = tok {
			if !self.is_finished_parsing() {
				bail!("Instruction was incomplete {pos}");
			}
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
				| InstrKind::License(text)
				| InstrKind::ModrinthID(text)
				| InstrKind::CurseForgeID(text) => {
					if text.is_empty() {
						text.fill(parse_string(tok, pos)?);
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
				InstrKind::Cmd(list) => list.push(parse_arg(tok, pos)?),
				InstrKind::SupportedModloaders(list) => match tok {
					Token::Ident(name) => {
						if let Some(val) = ModloaderMatch::parse_from_str(name) {
							list.push(val);
						} else {
							bail!("Value is not a valid modloader match argument")
						}
					}
					_ => unexpected_token!(tok, pos),
				},
				InstrKind::SupportedPluginLoaders(list) => match tok {
					Token::Ident(name) => {
						if let Some(val) = PluginLoaderMatch::parse_from_str(name) {
							list.push(val);
						} else {
							bail!("Value is not a valid plugin loader match argument")
						}
					}
					_ => unexpected_token!(tok, pos),
				},
				InstrKind::SupportedSides(list) => match tok {
					Token::Ident(name) => {
						if let Some(val) = Side::parse_from_str(name) {
							list.push(val);
						} else {
							bail!("Value is not a valid side argument")
						}
					}
					_ => unexpected_token!(tok, pos),
				},
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
					if var.is_full() {
						if let Value::None = val {
							*val = parse_arg(tok, pos)?;
						} else {
							unexpected_token!(tok, pos);
						}
					} else {
						match tok {
							Token::Ident(name) => var.fill(name.clone()),
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

impl Display for Instruction {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.kind)
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
