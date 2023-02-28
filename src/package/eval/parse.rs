use crate::io::files::paths::Paths;
use crate::package::eval::conditions::ConditionKind;
use super::super::{Package, PkgError};
use super::conditions::Condition;
use super::lex::{lex, LexError, Token, reduce_tokens, TextPos, Side};
use super::instruction::{Instruction, InstrKind};

use std::collections::HashMap;

static DEFAULT_ROUTINE: &str = "__default__";

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
	#[error("{}", .0)]
	Lex(#[from] LexError),
	#[error("Unexpected token '{}' at {}", .0, .1)]
	UnexpectedToken(String, TextPos),
	#[error("Routines must be declared at the root scope {}", .0)]
	UnexpectedRoutine(TextPos),
	#[error("Unknown instruction '{}' at {}", .0, .1)]
	UnknownInstr(String, TextPos)
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

// Mode for what we are currently parsing
#[derive(Debug)]
enum ParseMode {
	Root,
	Routine(Option<String>),
	Instruction(Instruction),
	If(Option<Condition>)
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
			let tokens = match lex(&data.contents) {
				Ok(tokens) => Ok(tokens),
				Err(e) => Err(ParseError::from(e))
			}?;
			let tokens = reduce_tokens(&tokens);

			let mut prs = ParseData::new();
			for (tok, pos) in tokens.iter() {
				let mut instr_to_push = None;
				let mut mode_to_set = None;
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
									"if" => {
										prs.mode = ParseMode::If(None);
									},
									name => {
										prs.mode = ParseMode::Instruction(Instruction::from_str(name));
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
							Token::Curly(side) => {
								match side {
									Side::Left => if let Some(condition) = condition {
										prs.block = prs.parsed.new_block(Some(prs.block));
										instr_to_push = Some(Instruction::new(InstrKind::If(condition.clone(), prs.block)));
										prs.mode = ParseMode::Root;
									}
									Side::Right => {
										return Err(PkgError::Parse(ParseError::UnexpectedToken(tok.as_string(), pos.clone())))
									}
								}
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
				}?;

				if let Some(instr) = instr_to_push {
					prs.new_instruction(instr);
				}

				if let Some(mode) = mode_to_set {
					prs.mode = mode;
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
