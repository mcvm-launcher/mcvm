use super::parse::{BlockId, Block};
use super::instruction::{Instruction, InstrKind};
use super::super::{Package, PkgError};
use crate::data::instance::InstKind;
use crate::data::asset::Modloader;
use crate::util::versions::MinecraftVersion;
use crate::io::files::paths::Paths;

use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum EvalError {
	#[error("Variable '{}' is not defined", .0)]
	VarNotDefined(String),
	#[error("Routine '{}' does not exist", .0)]
	RoutineDoesNotExist(String)
}

pub enum EvalPermissions {
	None,
	Info,
	All
}

impl EvalPermissions {
	pub fn is_info(&self) -> bool {
		match self {
			Self::None => false,
			_ => true
		}
	}

	pub fn is_all(&self) -> bool {
		match self {
			Self::All => true,
			_ => false
		}
	}
}

pub struct EvalConstants {
	pub perms: EvalPermissions,
	pub version: MinecraftVersion,
	pub modloader: Modloader,
	pub side: InstKind
}

pub struct EvalData<'a> {
	pub vars: HashMap<String, String>,
	pub constants: &'a EvalConstants
}

impl<'a> EvalData<'a> {
	pub fn new(constants: &'a EvalConstants) -> Self {
		Self {
			vars: HashMap::new(),
			constants
		}
	}
}

pub struct EvalResult {
	vars_to_set: Vec<(String, String)>
}

impl EvalResult {
	pub fn new() -> Self {
		Self {
			vars_to_set: Vec::new()
		}
	}

	pub fn merge(&mut self, mut other: EvalResult) {
		self.vars_to_set.append(&mut other.vars_to_set);
	}
}

impl Package {
	pub fn eval(&mut self, paths: &Paths, routine: &str, constants: &EvalConstants) -> Result<(), PkgError> {
		self.ensure_loaded(paths)?;
		self.parse(paths)?;
		if let Some(data) = &mut self.data {
			if let Some(parsed) = &mut data.parsed {
				let routine_id = parsed.routines.get(routine)
					.ok_or(EvalError::RoutineDoesNotExist(routine.to_owned()))?;
				let block = parsed.blocks.get(routine_id)
					.ok_or(EvalError::RoutineDoesNotExist(routine.to_owned()))?;

				let mut eval = EvalData::new(constants);

				match constants.perms {
					EvalPermissions::All | EvalPermissions::Info => {
						for instr in &block.contents {
							let result = instr.eval(constants, &eval, &parsed.blocks)?;
							for (var, val) in result.vars_to_set {
								eval.vars.insert(var, val);
							}	
						}
					}
					EvalPermissions::None => {}
				}
			}
		}
		Ok(())
	}
}

fn eval_block(block: &Block, constants: &EvalConstants, eval: &EvalData, blocks: &HashMap<BlockId, Block>)
	-> Result<EvalResult, EvalError> {
		let mut out = EvalResult::new();

		for instr in &block.contents {
			out.merge(instr.eval(constants, eval, blocks)?);
		}

		Ok(out)
	}

impl Instruction {
	pub fn eval(&self, constants: &EvalConstants, eval: &EvalData, blocks: &HashMap<BlockId, Block>)
	-> Result<EvalResult, EvalError> {
		if constants.perms.is_all() {
			match &self.kind {
				InstrKind::If(condition, block) => {
					if condition.kind.eval(constants, eval)? {
						eval_block(blocks.get(block).expect("If block missing"), constants, eval, blocks)?;
					}
				}
				_ => {}
			}
		}
		if constants.perms.is_info() {}
		Ok(EvalResult::new())
	}
}
