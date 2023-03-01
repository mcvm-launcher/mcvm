use super::parse::{BlockId, Block};
use super::instruction::{Instruction, InstrKind};
use super::super::{Package, PkgError};
use crate::data::instance::InstKind;
use crate::data::asset::{Modloader, AssetDownload, Asset};
use crate::package::reg::PkgIdentifier;
use crate::util::versions::MinecraftVersion;
use crate::io::files::paths::Paths;

use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum EvalError {
	#[error("Variable '{}' is not defined", .0)]
	VarNotDefined(String),
	#[error("Routine '{}' does not exist", .0)]
	RoutineDoesNotExist(String),
	#[error("Evaluator failed to start")]
	Start
}

#[derive(Debug, Clone)]
pub enum EvalLevel {
	None,
	Info,
	All
}

impl EvalLevel {
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

// A routine that we will run
pub enum Routine {
	Install
}

impl Routine {
	pub fn to_string(&self) -> String {
		String::from(match self {
			Self::Install => "install"
		})
	}

	pub fn get_level(&self) -> EvalLevel {
		match self {
			Self::Install => EvalLevel::All
		}
	}
}

#[derive(Debug, Clone)]
pub struct EvalConstants {
	pub version: MinecraftVersion,
	pub modloader: Modloader,
	pub side: InstKind,
	pub features: Vec<String>
}

#[derive(Debug, Clone)]
pub struct EvalData {
	pub vars: HashMap<String, String>,
	pub downloads: Vec<AssetDownload>,
	pub constants: EvalConstants,
	pub id: PkgIdentifier,
	pub level: EvalLevel
}

impl EvalData {
	pub fn new(constants: EvalConstants, id: PkgIdentifier, routine: &Routine) -> Self {
		Self {
			vars: HashMap::new(),
			downloads: Vec::new(),
			constants,
			id,
			level: routine.get_level()
		}
	}
}

pub struct EvalResult {
	vars_to_set: HashMap<String, String>,
	finish: bool,
	downloads: Vec<AssetDownload>
}

impl EvalResult {
	pub fn new() -> Self {
		Self {
			vars_to_set: HashMap::new(),
			finish: false,
			downloads: Vec::new()
		}
	}

	pub fn merge(&mut self, other: EvalResult) {
		self.vars_to_set.extend(other.vars_to_set);
		self.finish = other.finish;
		self.downloads.extend(other.downloads);
	}
}

impl Package {
	pub async fn eval(&mut self, paths: &Paths, routine: Routine, constants: EvalConstants)
	-> Result<EvalData, PkgError> {
		self.ensure_loaded(paths)?;
		self.parse(paths)?;
		if let Some(data) = &mut self.data {
			if let Some(parsed) = &mut data.parsed {
				let routine_name = routine.to_string();
				let routine_id = parsed.routines.get(&routine_name)
					.ok_or(EvalError::RoutineDoesNotExist(routine_name.clone()))?;
				let block = parsed.blocks.get(routine_id)
					.ok_or(EvalError::RoutineDoesNotExist(routine_name))?;

				let mut eval = EvalData::new(constants, self.id.clone(), &routine);

				match eval.level {
					EvalLevel::All | EvalLevel::Info => {
						for instr in &block.contents {
							let result = instr.eval(&eval, &parsed.blocks)?;
							for (var, val) in result.vars_to_set {
								eval.vars.insert(var, val);
							}
							eval.downloads.extend(result.downloads);
							if result.finish {
								break;
							}
						}

						for asset in &eval.downloads {
							asset.download(&paths).await?;
						}
					}
					EvalLevel::None => {}
				}
				return Ok(eval);
			}
		}
		Err(PkgError::Eval(EvalError::Start))
	}
}

fn eval_block(block: &Block, eval: &EvalData, blocks: &HashMap<BlockId, Block>)
-> Result<EvalResult, EvalError> {
	// We clone this so that state can be changed between each instruction
	let mut eval_clone = eval.clone();
	let mut out = EvalResult::new();

	for instr in &block.contents {
		let result = instr.eval(&eval_clone, blocks)?;
		for (var, val) in result.vars_to_set.clone() {
			eval_clone.vars.insert(var, val);
		}
		if result.finish {
			out.finish = true;
			break;
		}
		out.merge(result);
	}

	Ok(out)
}

impl Instruction {
	pub fn eval(&self, eval: &EvalData, blocks: &HashMap<BlockId, Block>)
	-> Result<EvalResult, EvalError> {
		let mut out = EvalResult::new();
		if eval.level.is_all() {
			match &self.kind {
				InstrKind::If(condition, block) => {
					if condition.kind.eval(eval)? {
						let result = eval_block(
							blocks.get(block).expect("If block missing"),
							eval, blocks
						)?;
						out.merge(result);
					}
				}
				InstrKind::Set(var, val) => {
					let var = var.as_ref().expect("Set variable missing");
					out.vars_to_set.insert(var.to_owned(), val.get(&eval.vars)?);
				}
				InstrKind::Finish() => out.finish = true,
				InstrKind::Asset {
					name,
					kind,
					url
				} => {
					let asset = Asset::new(
						kind.as_ref().expect("Asset kind missing").clone(),
						&name.get(&eval.vars)?,
						eval.id.clone()
					);

					out.downloads.push(AssetDownload::new(asset, &url.get(&eval.vars)?));
				},
				_ => {}
			}
		}
		if eval.level.is_info() {
			match &self.kind {
				_ => {}
			}
		}

		Ok(out)
	}
}
