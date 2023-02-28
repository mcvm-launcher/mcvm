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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct EvalConstants {
	pub perms: EvalPermissions,
	pub version: MinecraftVersion,
	pub modloader: Modloader,
	pub side: InstKind
}

#[derive(Debug, Clone)]
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
	vars_to_set: HashMap<String, String>,
	finish: bool
}

impl EvalResult {
	pub fn new() -> Self {
		Self {
			vars_to_set: HashMap::new(),
			finish: false
		}
	}

	pub fn merge(&mut self, other: EvalResult) {
		self.vars_to_set.extend(other.vars_to_set);
	}
}

impl Package {
	pub fn eval(&mut self, paths: &Paths, routine: &str, constants: &EvalConstants)
	-> Result<(), PkgError> {
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
							if result.finish {
								break;
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
	// We clone this so that state can be changed between each instruction
	let mut eval_clone = eval.clone();
	let mut finish = false;

	for instr in &block.contents {
		let result = instr.eval(constants, &eval_clone, blocks)?;
		for (var, val) in result.vars_to_set {
			eval_clone.vars.insert(var, val);
		}
		if result.finish {
			finish = true;
			break;
		}
	}

	Ok(EvalResult {
		vars_to_set: eval_clone.vars,
		finish
	})
}

impl Instruction {
	pub fn eval(&self, constants: &EvalConstants, eval: &EvalData, blocks: &HashMap<BlockId, Block>)
	-> Result<EvalResult, EvalError> {
		let mut out = EvalResult::new();
		if constants.perms.is_all() {
			match &self.kind {
				InstrKind::If(condition, block) => {
					if condition.kind.eval(constants, eval)? {
						let result = eval_block(
							blocks.get(block).expect("If block missing"),
							constants, eval, blocks
						)?;
						out.merge(result);
					}
				}
				InstrKind::Set(var, val) => {
					let var = var.as_ref().expect("Set variable missing");
					out.vars_to_set.insert(var.to_owned(), val.get(&eval.vars)?);
				}
				InstrKind::Finish() => out.finish = true,
				_ => {}
			}
		}
		if constants.perms.is_info() {}

		Ok(out)
	}
}
