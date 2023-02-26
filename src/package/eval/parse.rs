use std::collections::HashMap;

use crate::io::files::paths::Paths;

use super::super::{Package, PkgError};
use super::lex::{lex, LexError, Token, reduce_tokens, TextPos};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
	#[error("{}", .0)]
	Lex(#[from] LexError),
	#[error("Unexpected token '{}' at {}", .0, .1)]
	UnexpectedToken(String, TextPos)
}

#[derive(Debug, Clone)]
pub enum InstrKind {
	None,
	Routine(String),
	If
}

#[derive(Debug, Clone)]
pub struct Instruction {
	kind: InstrKind
}

impl Instruction {
	pub fn new() -> Self {
		Self {
			kind: InstrKind::None
		}
	}

	pub fn new_kind(kind: InstrKind) -> Self {
		Self {
			kind
		}
	}
}

// Data for the root of an instruction
pub enum ParseRoot {
	Instruction(String, Vec<String>),
	Routine(String),
	None
}

// Data used for parsing
#[derive(Debug)]
pub struct ParseData {
	routines: HashMap<String, Vec<Instruction>>,
	instruction_n: u32,
	routine: String,
	block: Vec<Instruction>,
	instruction: Instruction,
	push_to_routine: bool
}

impl ParseData {
	pub fn new() -> Self {
		let default_rtn = String::from("__default__");
		Self {
			routines: HashMap::from([(default_rtn.clone(), Vec::new())]),
			instruction_n: 0,
			routine: default_rtn,
			block: Vec::new(),
			instruction: Instruction::new(),
			push_to_routine: false
		}
	}

	// Reset the instruction data
	pub fn reset_instruction(&mut self) {
		self.instruction_n += 1;
		self.block.push(self.instruction.clone());
		self.instruction = Instruction::new();
	}
}

impl Package {
	pub fn parse(&mut self, paths: &Paths) -> Result<(), PkgError> {
		self.ensure_loaded(paths)?;
		if let Some(data) = &self.data {
			let tokens = match lex(&data.contents) {
				Ok(tokens) => Ok(tokens),
				Err(e) => Err(ParseError::from(e))
			}?;
			let tokens = reduce_tokens(&tokens);

			let mut prs = ParseData::new();
			for (tok, pos) in tokens.iter() {
				let mut instr_finished = false;
				let mut block_finished = false;
				match &mut prs.instruction.kind {
					InstrKind::None => {
						match tok {
							Token::Routine => {
								prs.instruction = Instruction::new_kind(InstrKind::Routine(String::new()));
							}
							_ => {}
						}
						Ok(())
					}
					InstrKind::Routine(name) => {
						match tok {
							Token::Ident(ident) => {
								*name = ident.to_string();
								instr_finished = true;
							}
							_ => {
								return Err(PkgError::Parse(ParseError::UnexpectedToken(tok.as_string(), pos.clone())))
							}
						}
						Ok(())
					}
					_ => {
						Ok::<(), PkgError>(())
					}
				}?;

				if instr_finished {
					prs.reset_instruction();
				}
			}
			dbg!(&prs);
		}
		Ok(())
	}
}
