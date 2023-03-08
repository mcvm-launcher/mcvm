use crate::data::addon::AddonKind;
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
	#[error("Unknown addon key '{}' {}", .0, .1)]
	UnknownAddonKey(String, TextPos),
	#[error("Expected 'yes' or 'no', but got {} {}", .0, .1)]
	YesNo(String, TextPos),
	#[error("Unknown reason '{}' {}", .0, .1)]
	UnknownReason(String, TextPos),
	#[error("Unknown condition '{}' {}", .0, .1)]
	UnknownCondition(String, TextPos),
	#[error("Unknown condition argument '{}' {}", .0, .1)]
	UnknownConditionArg(String, TextPos)
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

// State of the addon parser
#[derive(Debug)]
enum AddonMode {
	Opening,
	Key,
	Colon,
	Value,
	Comma
}

// Current key for the addon parser
#[derive(Debug)]
enum AddonKey {
	None,
	Kind,
	Url,
	Force,
	Append,
	Path
}

// Mode for what we are currently parsing
#[derive(Debug)]
enum ParseMode {
	Root,
	Routine(Option<String>),
	Instruction(Instruction),
	If(Option<Condition>),
	Addon {
		mode: AddonMode,
		key: AddonKey,
		name: Value,
		kind: Option<AddonKind>,
		url: Value,
		force: bool,
		append: Value,
		path: Value
	}
}

#[macro_export]
macro_rules! unexpected_token {
	($tok:expr, $pos:expr) => {
		return Err(PkgError::Parse(ParseError::UnexpectedToken($tok.as_string(), $pos.clone())))
	};
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
		self.ensure_loaded(paths, false)?;
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
									"addon" => {
										prs.mode = ParseMode::Addon {
											mode: AddonMode::Opening,
											key: AddonKey::None,
											name: Value::None,
											kind: None,
											url: Value::None,
											force: false,
											append: Value::None,
											path: Value::None
										};
									}
									name => {
										prs.mode = ParseMode::Instruction(Instruction::from_str(name, pos)?);
									}
								}
							}
							Token::Curly(side) => {
								match side {
									Side::Left => unexpected_token!(tok, pos),
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
										Side::Right => unexpected_token!(tok, pos)
									}
								}
								_ => unexpected_token!(tok, pos)
							}
						} else {
							match tok {
								Token::Ident(ident) => {
									*name = Some(ident.to_string());
								}
								_ => unexpected_token!(tok, pos)
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
							Token::Curly(Side::Right) => unexpected_token!(tok, pos),
							_ => match condition {
								Some(condition) => condition.parse(tok, pos)?,
								None => match tok {
									Token::Ident(name) => match ConditionKind::from_str(name) {
										Some(new_condition) => *condition = Some(Condition::new(new_condition)),
										None => return Err(PkgError::Parse(ParseError::UnknownCondition(name.clone(), pos.clone())))
									},
									_ => unexpected_token!(tok, pos)
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
					ParseMode::Addon {
						mode,
						key,
						name,
						kind,
						url,
						force,
						append,
						path
					} => {
						match mode {
							AddonMode::Opening => match tok {
								Token::Paren(Side::Left) => {
									if let Value::None = name {
										unexpected_token!(tok, pos)
									}
									*mode = AddonMode::Key;
								}
								_ => *name = parse_arg(tok, pos)?
							}
							AddonMode::Key => match tok {
								Token::Ident(name) => {
									match name.as_str() {
										"kind" => *key = AddonKey::Kind,
										"url" => *key = AddonKey::Url,
										"force" => *key = AddonKey::Force,
										"append" => *key = AddonKey::Append,
										"path" => *key = AddonKey::Path,
										_ => return Err(PkgError::Parse(ParseError::UnknownAddonKey(name.to_owned(), pos.clone())))
									}
									*mode = AddonMode::Colon;
								}
								_ => unexpected_token!(tok, pos)
							}
							AddonMode::Colon => match tok {
								Token::Colon => *mode = AddonMode::Value,
								_ => unexpected_token!(tok, pos)
							}
							AddonMode::Value => {
								match tok {
									Token::Ident(name) => {
										match key {
											AddonKey::Kind => *kind = AddonKind::from_str(name),
											AddonKey::Force => match yes_no(name) {
												Some(value) => *force = value,
												None => return Err(PkgError::Parse(ParseError::YesNo(name.to_owned(), pos.clone())))
											}
											_ => unexpected_token!(tok, pos)
										}
										*mode = AddonMode::Comma;
									}
									_ => {
										match key {
											AddonKey::Url => *url = parse_arg(tok, pos)?,
											AddonKey::Append => *append = parse_arg(tok, pos)?,
											AddonKey::Path => *path = parse_arg(tok, pos)?,
											_ => unexpected_token!(tok, pos)
										}
										*mode = AddonMode::Comma;
									}
								}
							}
							AddonMode::Comma => match tok {
								Token::Comma => *mode = AddonMode::Key,
								Token::Paren(Side::Right) => {
									instr_to_push = Some(Instruction::new(InstrKind::Addon {
										name: name.clone(),
										kind: kind.clone(),
										url: url.clone(),
										force: *force,
										append: append.clone(),
										path: path.clone()
									}));
									prs.mode = ParseMode::Root;
								}
								_ => unexpected_token!(tok, pos)
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
