use super::{parse::{BlockId, ParseError}, lex::{Token, TextPos}, Value};
use super::conditions::Condition;
use crate::data::asset::AssetKind;

#[derive(Debug, Clone)]
pub enum InstrKind {
	Unknown(String, Vec<String>),
	If(Condition, BlockId),
	Name(Value),
	Version(Value),
	AssetType(Option<AssetKind>),
	AssetName(Value),
	DownloadAsset(Value),
	Set(Option<String>, Value),
	Finish(),
	Fail()
}

#[derive(Debug, Clone)]
pub struct Instruction {
	pub kind: InstrKind,
	parse_var: bool
}

impl Instruction {
	pub fn new(kind: InstrKind) -> Self {
		Self {
			kind,
			parse_var: false
		}
	}

	pub fn from_str(string: &str) -> Self {
		let kind = match string {
			"name" => InstrKind::Name(Value::None),
			"version" => InstrKind::Version(Value::None),
			"asset_type" => InstrKind::AssetType(None),
			"asset_name" => InstrKind::AssetName(Value::None),
			"download_asset" => InstrKind::DownloadAsset(Value::None),
			"set" => InstrKind::Set(None, Value::None),
			"finish" => InstrKind::Finish(),
			"fail" => InstrKind::Fail(),
			string => InstrKind::Unknown(string.to_owned(), Vec::new())
		};
		Instruction::new(kind)
	}

	// Parses a token and returns true if finished
	pub fn parse(&mut self, tok: &Token, pos: &TextPos) -> Result<bool, ParseError> {
		if let Token::Semicolon = tok {
			Ok(true)
		} else {
			match &mut self.kind {
				InstrKind::Unknown(.., args) => match tok {
					Token::Ident(name) => {
						args.push(name.clone());
					}
					Token::Str(text) => {
						args.push(text.clone());
					}
					_ => {}
				}
				InstrKind::AssetName(val) |
				InstrKind::DownloadAsset(val) |
				InstrKind::Name(val) |
				InstrKind::Version(val) => {
					match parse_arg(tok, pos, self.parse_var)? {
						ParseArgResult::ParseVar => self.parse_var = true,
						ParseArgResult::Value(new_val) => {
							*val = new_val;
							self.parse_var = false;
						}
					}
				}
				InstrKind::AssetType(kind) => {
					match tok {
						Token::Ident(name) => *kind = AssetKind::from_str(name),
						_ => return Err(ParseError::UnexpectedToken(tok.as_string(), pos.clone()))
					}
				}
				InstrKind::Set(var, val) => {
					if var.is_some() {
						match parse_arg(tok, pos, self.parse_var)? {
							ParseArgResult::ParseVar => self.parse_var = true,
							ParseArgResult::Value(new_val) => {
								*val = new_val;
								self.parse_var = false;
							}
						}
					} else {
						match tok {
							Token::Ident(name) => *var = Some(name.clone()),
							_ => return Err(ParseError::UnexpectedToken(tok.as_string(), pos.clone()))
						}
					}
				}
				_ => {}
			}

			Ok(false)
		}
	}
}

pub enum ParseArgResult {
	Value(Value),
	ParseVar
}

// Parses a generic instruction argument
pub fn parse_arg(tok: &Token, pos: &TextPos, parse_var: bool) -> Result<ParseArgResult, ParseError> {
	match tok {
		Token::Dollar => Ok(ParseArgResult::ParseVar),
		Token::Ident(name) => if parse_var {
			Ok(ParseArgResult::Value(Value::Var(name.clone())))
		} else {
			Err(ParseError::UnexpectedToken(tok.as_string(), pos.clone()))
		}
		Token::Str(text) => Ok(ParseArgResult::Value(Value::Constant(text.clone()))),
		Token::Num(num) => Ok(ParseArgResult::Value(Value::Constant(num.to_string().clone()))),
		_ => Err(ParseError::UnexpectedToken(tok.as_string(), pos.clone()))
	}
}
