use std::fmt::Display;

use anyhow::bail;
use mcvm_shared::later::Later;
use mcvm_shared::modifications::{ModloaderMatch, PluginLoaderMatch};
use mcvm_shared::pkg::PackageAddonHashes;
use mcvm_shared::Side;

use super::conditions::Condition;
use super::lex::{TextPos, Token};
use super::parse::BlockId;
use super::vars::Value;
use super::FailReason;
use crate::unexpected_token;
use mcvm_shared::addon::AddonKind;

/// A command / statement run in a package script
#[derive(Debug, Clone)]
pub struct Instruction {
	/// What type of instruction this is
	pub kind: InstrKind,
}

/// Type of an instruction
#[derive(Debug, Clone)]
pub enum InstrKind {
	/// Check conditions
	If {
		/// The condition to check
		condition: Condition,
		/// The block to run if the condition succeeds
		if_block: BlockId,
		/// The chain of else blocks to run if the initial condition fails
		else_blocks: Vec<ElseBlock>,
	},
	/// Set the package name metadata
	Name(Later<String>),
	/// Set the package description metadata
	Description(Later<String>),
	/// Set the package long description metadata
	LongDescription(Later<String>),
	/// Set the package authors metadata
	Authors(Vec<String>),
	/// Set the package maintainers metadata
	PackageMaintainers(Vec<String>),
	/// Set the package website metadata
	Website(Later<String>),
	/// Set the package support link metadata
	SupportLink(Later<String>),
	/// Set the package documentation metadata
	Documentation(Later<String>),
	/// Set the package source metadata
	Source(Later<String>),
	/// Set the package issues metadata
	Issues(Later<String>),
	/// Set the package community metadata
	Community(Later<String>),
	/// Set the package icon metadata
	Icon(Later<String>),
	/// Set the package banner metadata
	Banner(Later<String>),
	/// Set the package gallery metadata
	Gallery(Vec<String>),
	/// Set the package license metadata
	License(Later<String>),
	/// Set the package keywords metadata
	Keywords(Vec<String>),
	/// Set the package categories metadata
	Categories(Vec<String>),
	/// Set the package features property
	Features(Vec<String>),
	/// Set the package default features property
	DefaultFeatures(Vec<String>),
	/// Set the package Modrinth ID property
	ModrinthID(Later<String>),
	/// Set the package CurseForge ID property
	CurseForgeID(Later<String>),
	/// Set the package supported modloaders property
	SupportedModloaders(Vec<ModloaderMatch>),
	/// Set the package supported plugin loaders property
	SupportedPluginLoaders(Vec<PluginLoaderMatch>),
	/// Set the package supported sides property
	SupportedSides(Vec<Side>),
	/// Set the package tags property
	Tags(Vec<String>),
	/// Install an addon
	Addon {
		/// The ID of the addon
		id: Value,
		/// The filename of the addon
		file_name: Value,
		/// What kind of addon this is
		kind: Option<AddonKind>,
		/// The URL to the addon file; may not exist
		url: Value,
		/// The path to the addon file; may not exist
		path: Value,
		/// The version of the addon
		version: Value,
		/// The addon's hashes
		hashes: PackageAddonHashes<Value>,
	},
	/// Set a variable to a value
	Set(Later<String>, Value),
	/// Require a package
	Require(Vec<Vec<super::parse::require::Package>>),
	/// Refuse a package
	Refuse(Value),
	/// Recommend a package
	Recommend(bool, Value),
	/// Bundle a package
	Bundle(Value),
	/// Compat with a package
	Compat(Value, Value),
	/// Extend a package
	Extend(Value),
	/// Finish evaluation early
	Finish(),
	/// Fail evaluation
	Fail(Option<FailReason>),
	/// Present a notice to the user
	Notice(Value),
	/// Run a command
	Cmd(Vec<Value>),
	/// Call another routine
	Call(Later<String>),
}

/// A non-nested else / else if block connected to an if
#[derive(Debug, Clone)]
pub struct ElseBlock {
	/// The block to run if this else succeeds
	pub block: BlockId,
	/// An additional condition that might need to be satisfied, used for else if.
	pub condition: Option<Condition>,
}

impl Display for InstrKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::If { .. } => "if",
				Self::Name(..) => "name",
				Self::Description(..) => "description",
				Self::LongDescription(..) => "long_description",
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
				Self::Gallery(..) => "gallery",
				Self::License(..) => "license",
				Self::Keywords(..) => "keywords",
				Self::Categories(..) => "categories",
				Self::Features(..) => "features",
				Self::DefaultFeatures(..) => "default_features",
				Self::ModrinthID(..) => "modrinth_id",
				Self::CurseForgeID(..) => "curseforge_id",
				Self::SupportedModloaders(..) => "supported_modloaders",
				Self::SupportedPluginLoaders(..) => "supported_plugin_loaders",
				Self::SupportedSides(..) => "supported_sides",
				Self::Tags(..) => "tags",
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
				Self::Call(..) => "call",
			}
		)
	}
}

impl Instruction {
	/// Create a new instruction
	pub fn new(kind: InstrKind) -> Self {
		Self { kind }
	}

	/// Starts an instruction from the provided string
	pub fn from_str(string: &str, pos: &TextPos) -> anyhow::Result<Self> {
		let kind = match string {
			"name" => Ok::<InstrKind, anyhow::Error>(InstrKind::Name(Later::Empty)),
			"description" => Ok(InstrKind::Description(Later::Empty)),
			"long_description" => Ok(InstrKind::LongDescription(Later::Empty)),
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
			"recommend" => Ok(InstrKind::Recommend(false, Value::None)),
			"bundle" => Ok(InstrKind::Bundle(Value::None)),
			"compat" => Ok(InstrKind::Compat(Value::None, Value::None)),
			"extend" => Ok(InstrKind::Extend(Value::None)),
			"notice" => Ok(InstrKind::Notice(Value::None)),
			"call" => Ok(InstrKind::Call(Later::Empty)),
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
			| InstrKind::Website(val)
			| InstrKind::Call(val) => val.is_full(),
			InstrKind::Features(val)
			| InstrKind::Authors(val)
			| InstrKind::PackageMaintainers(val)
			| InstrKind::DefaultFeatures(val)
			| InstrKind::Keywords(val)
			| InstrKind::Categories(val)
			| InstrKind::Tags(val)
			| InstrKind::Gallery(val) => !val.is_empty(),
			InstrKind::Refuse(val)
			| InstrKind::Recommend(_, val)
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
			InstrKind::If { .. } | InstrKind::Addon { .. } | InstrKind::Require(..) => {
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
				| InstrKind::DefaultFeatures(list)
				| InstrKind::Keywords(list)
				| InstrKind::Categories(list)
				| InstrKind::Tags(list)
				| InstrKind::Gallery(list) => list.push(parse_string(tok, pos)?),
				InstrKind::Cmd(list) => list.push(parse_arg(tok, pos)?),
				InstrKind::Recommend(inverted, val) => match tok {
					Token::Bang => {
						if *inverted || val.is_some() {
							unexpected_token!(tok, pos);
						}

						*inverted = true;
					}
					_ => {
						if let Value::None = val {
							*val = parse_arg(tok, pos)?;
						} else {
							unexpected_token!(tok, pos);
						}
					}
				},
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
				InstrKind::Call(routine) => {
					match tok {
						Token::Ident(name) => {
							if crate::routine::is_reserved(name) {
								bail!("Cannot use reserved routine name '{name}' in call instruction {}", pos.clone());
							}
							routine.fill(name.clone())
						}
						_ => unexpected_token!(tok, pos),
					}
				}
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
		Token::Str(text) => Ok(Value::Literal(text.clone())),
		Token::Num(num) => Ok(Value::Literal(num.to_string())),
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
