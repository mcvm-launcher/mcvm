use crate::data::asset::AssetKind;
use crate::io::files::paths::Paths;
use crate::package::eval::conditions::ConditionKind;
use crate::util::yes_no;
use super::super::{Package, PkgError};
use super::Value;
use super::conditions::Condition;
use super::lex::{lex, LexError, Token, reduce_tokens, TextPos, Side};
use super::instruction::{Instruction, InstrKind, parse_arg};

use std::collections::HashMap;

static DEFAULT_ROUTINE: &str = "__default__";

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
	#[error("{}", .0)]
	Lex(#[from] LexError),
	#[error("Unexpected token '{}' {}", .0, .1)]
	UnexpectedToken(String, TextPos),
	#[error("Routines must be declared at the root scope {}", .0)]
	UnexpectedRoutine(TextPos),
	#[error("Unknown instruction '{}' {}", .0, .1)]
	UnknownInstr(String, TextPos),
	#[error("Unknown asset key '{}' {}", .0, .1)]
	UnknownAssetKey(String, TextPos),
	#[error("Expected 'yes' or 'no', but got {} {}", .0, .1)]
	YesNo(String, TextPos)
}

pub type BlockId = u16;

#[derive(Debug, Clone)]
pub struct Block {
	pub contents: Vec<Instruction>,
	parent: Option<BlockId>
}

impl Block {
	pub fn new(parent: Option<BlockId>) -> Self {
		Self {
			contents: Vec::new(),
			parent
		}
	}

	pub fn push(&mut self, instr: Instruction) {
		self.contents.push(instr);
	}
}

#[derive(Debug)]
pub struct Parsed {
	pub blocks: HashMap<BlockId, Block>,
	pub routines: HashMap<String, BlockId>,
	id_count: BlockId
}

impl Parsed {
	pub fn new() -> Self {
		let mut out = Self {
			blocks: HashMap::new(),
			routines: HashMap::new(),
			id_count: 0
		};
		out.routines = HashMap::from([(String::from(DEFAULT_ROUTINE), out.new_block(None))]);
		out
	}

	// Creates a new block and returns its ID
	pub fn new_block(&mut self, parent: Option<BlockId>) -> BlockId {
		self.id_count += 1;
		self.blocks.insert(self.id_count, Block::new(parent));
		self.id_count
	}

	// Creates a new routine and its associated block, then returns the block's ID
	pub fn new_routine(&mut self, name: &str) -> BlockId {
		self.new_block(None);
		self.routines.insert(name.to_owned(), self.id_count);
		self.id_count
	}
}

// State of the asset parser
#[derive(Debug)]
enum AssetMode {
	Opening,
	Key,
	Colon,
	Value,
	Comma
}

// Current key for the asset parser
#[derive(Debug)]
enum AssetKey {
	None,
	Kind,
	Url,
	Force
}

// Mode for what we are currently parsing
#[derive(Debug)]
enum ParseMode {
	Root,
	Routine(Option<String>),
	Instruction(Instruction),
	If(Option<Condition>),
	Asset {
		mode: AssetMode,
		key: AssetKey,
		name: Value,
		kind: Option<AssetKind>,
		url: Value,
		force: bool
	}
}

// Data used for parsing
#[derive(Debug)]
pub struct ParseData {
	parsed: Parsed,
	instruction_n: u32,
	block: BlockId,
	mode: ParseMode
}

impl ParseData {
	pub fn new() -> Self {
		Self {
			parsed: Parsed::new(),
			instruction_n: 0,
			block: 1,
			mode: ParseMode::Root
		}
	}

	// Push a new instruction to the block
	pub fn new_instruction(&mut self, instr: Instruction) {
		self.instruction_n += 1;
		if let Some(block) = self.parsed.blocks.get_mut(&self.block) {
			block.push(instr);
		}
		self.mode = ParseMode::Root;
	}

	// Finish the current block
	pub fn new_block(&mut self) {
		if let Some(block) = self.parsed.blocks.get_mut(&self.block) {
			if let Some(parent) = block.parent {
				self.block = parent;
			}
		}
	}
}

impl Package {
	pub fn parse(&mut self, paths: &Paths) -> Result<(), PkgError> {
		self.ensure_loaded(paths)?;
		if let Some(data) = &mut self.data {
			if data.parsed.is_some() {
				return Ok(())
			}

			let tokens = match lex(&data.contents) {
				Ok(tokens) => Ok(tokens),
				Err(e) => Err(ParseError::from(e))
			}?;
			let tokens = reduce_tokens(&tokens);

			let mut prs = ParseData::new();
			for (tok, pos) in tokens.iter() {
				let mut instr_to_push = None;
				let mut mode_to_set = None;
				let mut block_to_set = None;
				let mut block_finished = false;
				match &mut prs.mode {
					ParseMode::Root => {
						match tok {
							Token::Routine => {
								if let Some(..) = prs.parsed.blocks.get_mut(&prs.block).expect("Block does not exist").parent {
									return Err(PkgError::Parse(ParseError::UnexpectedRoutine(pos.clone())))
								}
								prs.mode = ParseMode::Routine(None);
							}
							Token::Ident(name) => {
								match name.as_str() {
									"if" => prs.mode = ParseMode::If(None),
									"asset" => {
										prs.mode = ParseMode::Asset {
											mode: AssetMode::Opening,
											key: AssetKey::None,
											name: Value::None,
											kind: None,
											url: Value::None,
											force: false
										};
									}
									name => {
										prs.mode = ParseMode::Instruction(Instruction::from_str(name, pos)?);
									}
								}
							}
							Token::Curly(side) => {
								match side {
									Side::Left => {
										return Err(PkgError::Parse(ParseError::UnexpectedToken(tok.as_string(), pos.clone())))
									}
									Side::Right => {
										block_finished = true;
										prs.mode = ParseMode::Root;
									}
								}
							}
							_ => {}
						}
						Ok::<(), PkgError>(())
					}
					ParseMode::Routine(name) => {
						if let Some(name) = name {
							match tok {
								Token::Curly(side) => {
									match side {
										Side::Left => {
											prs.block = prs.parsed.new_routine(name);
											prs.mode = ParseMode::Root;
										}
										Side::Right => {
											return Err(PkgError::Parse(ParseError::UnexpectedToken(tok.as_string(), pos.clone())))
										}
									}
								}
								_ => {
									return Err(PkgError::Parse(ParseError::UnexpectedToken(tok.as_string(), pos.clone())))
								}
							}
						} else {
							match tok {
								Token::Ident(ident) => {
									*name = Some(ident.to_string());
								}
								_ => {
									return Err(PkgError::Parse(ParseError::UnexpectedToken(tok.as_string(), pos.clone())))
								}
							}
						}
						Ok(())
					}
					ParseMode::If(condition) => {
						match tok {
							Token::Curly(Side::Left) => if let Some(condition) = condition {
								let block = prs.parsed.new_block(Some(prs.block));
								block_to_set = Some(block);
								instr_to_push = Some(Instruction::new(InstrKind::If(condition.clone(), block)));
								prs.mode = ParseMode::Root;
							}
							Token::Curly(Side::Right) => {
								return Err(PkgError::Parse(ParseError::UnexpectedToken(tok.as_string(), pos.clone())))
							}
							_ => match condition {
								Some(condition) => condition.parse(tok, pos)?,
								None => match tok {
									Token::Ident(name) => match ConditionKind::from_str(name) {
										Some(new_condition) => *condition = Some(Condition::new(new_condition)),
										None => {}
									},
									_ => return Err(PkgError::Parse(ParseError::UnexpectedToken(tok.as_string(), pos.clone())))
								}
							}
						}

						Ok(())
					}
					ParseMode::Instruction(instr) => {
						if instr.parse(tok, pos)? {
							instr_to_push = Some(instr.clone());
							mode_to_set = Some(ParseMode::Root);
						}

						Ok(())
					}
					ParseMode::Asset {
						mode,
						key,
						name,
						kind,
						url,
						force
					} => {
						match mode {
							AssetMode::Opening => match tok {
								Token::Paren(Side::Left) => {
									if let Value::None = name {
										return Err(PkgError::Parse(ParseError::UnexpectedToken(tok.as_string(), pos.clone())))
									}
									*mode = AssetMode::Key;
								}
								_ => *name = parse_arg(tok, pos)?
							}
							AssetMode::Key => match tok {
								Token::Ident(name) => {
									match name.as_str() {
										"kind" => *key = AssetKey::Kind,
										"url" => *key = AssetKey::Url,
										"force" => *key = AssetKey::Force,
										_ => return Err(PkgError::Parse(ParseError::UnknownAssetKey(name.to_owned(), pos.clone())))
									}
									*mode = AssetMode::Colon;
								}
								_ => return Err(PkgError::Parse(ParseError::UnexpectedToken(tok.as_string(), pos.clone())))
							}
							AssetMode::Colon => match tok {
								Token::Colon => *mode = AssetMode::Value,
								_ => return Err(PkgError::Parse(ParseError::UnexpectedToken(tok.as_string(), pos.clone())))
							}
							AssetMode::Value => {
								match tok {
									Token::Ident(name) => {
										match key {
											AssetKey::Kind => *kind = AssetKind::from_str(name),
											AssetKey::Force => match yes_no(name) {
												Some(value) => *force = value,
												None => return Err(PkgError::Parse(ParseError::YesNo(name.to_owned(), pos.clone())))
											}
											_ => return Err(PkgError::Parse(ParseError::UnexpectedToken(tok.as_string(), pos.clone())))
										}
										*mode = AssetMode::Comma;
									}
									_ => {
										match key {
											AssetKey::Url => *url = parse_arg(tok, pos)?,
											_ => return Err(PkgError::Parse(ParseError::UnexpectedToken(tok.as_string(), pos.clone())))
										}
										*mode = AssetMode::Comma;
									}
								}
							}
							AssetMode::Comma => match tok {
								Token::Comma => *mode = AssetMode::Key,
								Token::Paren(Side::Right) => {
									instr_to_push = Some(Instruction::new(InstrKind::Asset {
										name: name.clone(),
										kind: kind.clone(),
										url: url.clone(),
										force: force.clone()
									}));
									prs.mode = ParseMode::Root;
								}
								_ => return Err(PkgError::Parse(ParseError::UnexpectedToken(tok.as_string(), pos.clone())))
							}
						}

						Ok(())
					}
				}?;

				if let Some(instr) = instr_to_push {
					prs.new_instruction(instr);
				}

				if let Some(mode) = mode_to_set {
					prs.mode = mode;
				}

				if let Some(block) = block_to_set {
					prs.block = block;
				}
				
				if block_finished {
					prs.new_block();
				}
			}
			data.parsed = Some(prs.parsed);
		}
		Ok(())
	}
}
