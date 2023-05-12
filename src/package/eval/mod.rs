pub mod conditions;
pub mod parse;

use anyhow::{anyhow, bail};
use mcvm_shared::addon::{is_filename_valid, Addon};
use serde::{Deserialize, Serialize};

use self::conditions::eval_condition;

use super::Package;
use crate::data::addon::{AddonLocation, AddonRequest};
use crate::io::files::paths::Paths;
use crate::util::validate_identifier;
use mcvm_parse::instruction::{InstrKind, Instruction};
use mcvm_parse::parse::{Block, BlockId};
use mcvm_parse::{FailReason, Value};
use mcvm_shared::instance::Side;
use mcvm_shared::modifications::{Modloader, PluginLoader};
use mcvm_shared::pkg::PkgIdentifier;

use std::collections::HashMap;
use std::path::PathBuf;

/// What instructions the evaluator will evaluate (depends on what routine we are running)
#[derive(Debug, Clone)]
pub enum EvalLevel {
	/// When we are installing the addons of a package
	Install,
	/// When we are resolving package relationships
	Resolve,
}

/// Permissions level for an evaluation
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum EvalPermissions {
	Restricted,
	#[default]
	Standard,
	Elevated,
}

/// A routine with a special meaning / purpose.
pub enum Routine {
	Install,
	/// Install routine, except for resolution
	InstallResolve,
}

impl Routine {
	pub fn get_routine_name(&self) -> String {
		String::from(match self {
			Self::Install => "install",
			Self::InstallResolve => "install",
		})
	}

	pub fn get_level(&self) -> EvalLevel {
		match self {
			Self::Install => EvalLevel::Install,
			Self::InstallResolve => EvalLevel::Resolve,
		}
	}
}

/// Constants provided by the function calling eval
#[derive(Debug, Clone)]
pub struct EvalConstants {
	pub version: String,
	pub modloader: Modloader,
	pub plugin_loader: PluginLoader,
	pub side: Side,
	pub features: Vec<String>,
	pub versions: Vec<String>,
	pub perms: EvalPermissions,
}

/// Persistent state for evaluation
#[derive(Debug, Clone)]
pub struct EvalData {
	pub vars: HashMap<String, String>,
	pub addon_reqs: Vec<AddonRequest>,
	pub constants: EvalConstants,
	pub id: PkgIdentifier,
	pub level: EvalLevel,
	pub deps: Vec<Vec<String>>,
	pub conflicts: Vec<String>,
}

impl EvalData {
	pub fn new(constants: EvalConstants, id: PkgIdentifier, routine: &Routine) -> Self {
		Self {
			vars: HashMap::new(),
			addon_reqs: Vec::new(),
			constants,
			id,
			level: routine.get_level(),
			deps: Vec::new(),
			conflicts: Vec::new(),
		}
	}
}

/// Result from an evaluation subfunction. We merge this with the main EvalData
pub struct EvalResult {
	vars_to_set: HashMap<String, String>,
	finish: bool,
	addon_reqs: Vec<AddonRequest>,
	deps: Vec<Vec<String>>,
	pub conflicts: Vec<String>,
}

impl EvalResult {
	pub fn new() -> Self {
		Self {
			vars_to_set: HashMap::new(),
			finish: false,
			addon_reqs: Vec::new(),
			deps: Vec::new(),
			conflicts: Vec::new(),
		}
	}

	/// Merge multiple EvalResults
	pub fn merge(&mut self, other: EvalResult) {
		self.vars_to_set.extend(other.vars_to_set);
		self.finish = other.finish;
		self.addon_reqs.extend(other.addon_reqs);
		self.deps.extend(other.deps);
	}
}

impl Default for EvalResult {
	fn default() -> Self {
		Self::new()
	}
}

impl Package {
	/// Evaluate a routine on a package
	pub async fn eval(
		&mut self,
		paths: &Paths,
		routine: Routine,
		constants: &EvalConstants,
	) -> anyhow::Result<EvalData> {
		self.ensure_loaded(paths, false).await?;
		self.parse(paths).await?;
		let parsed = self.data.get_mut().parsed.get_mut();
		let routine_name = routine.get_routine_name();
		let routine_id = parsed
			.routines
			.get(&routine_name)
			.ok_or(anyhow!("Routine {} does not exist", routine_name.clone()))?;
		let block = parsed
			.blocks
			.get(routine_id)
			.ok_or(anyhow!("Routine {} does not exist", routine_name))?;

		let mut eval = EvalData::new(constants.clone(), self.id.clone(), &routine);

		for instr in &block.contents {
			let result = eval_instr(instr, &eval, &parsed.blocks)?;
			for (var, val) in result.vars_to_set {
				eval.vars.insert(var, val);
			}
			eval.addon_reqs.extend(result.addon_reqs);
			eval.deps.extend(result.deps);
			if result.finish {
				break;
			}
		}

		Ok(eval)
	}
}

/// Evaluate a block of instructions
fn eval_block(
	block: &Block,
	eval: &EvalData,
	blocks: &HashMap<BlockId, Block>,
) -> anyhow::Result<EvalResult> {
	// We clone this so that state can be changed between each instruction
	let mut eval_clone = eval.clone();
	let mut out = EvalResult::new();

	for instr in &block.contents {
		let result = eval_instr(instr, &eval_clone, blocks)?;
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

/// Evaluate an instruction
pub fn eval_instr(
	instr: &Instruction,
	eval: &EvalData,
	blocks: &HashMap<BlockId, Block>,
) -> anyhow::Result<EvalResult> {
	let mut out = EvalResult::new();
	match eval.level {
		EvalLevel::Install | EvalLevel::Resolve => match &instr.kind {
			InstrKind::If(condition, block) => {
				if eval_condition(&condition.kind, eval)? {
					let result =
						eval_block(blocks.get(block).expect("If block missing"), eval, blocks)?;
					out.merge(result);
				}
			}
			InstrKind::Set(var, val) => {
				let var = var.as_ref().expect("Set variable missing");
				out.vars_to_set.insert(var.to_owned(), val.get(&eval.vars)?);
			}
			InstrKind::Finish() => out.finish = true,
			InstrKind::Fail(reason) => {
				out.finish = true;
				let reason = reason.as_ref().unwrap_or(&FailReason::None).clone();
				bail!("Package script failed with reason: {}", reason.to_string());
			}
			InstrKind::Require(deps, ..) => if let EvalLevel::Resolve = eval.level {
				for dep in deps {
					let mut dep_to_push = Vec::new();
					for dep in dep {
						dep_to_push.push(dep.get(&eval.vars)?);
					}
					out.deps.push(dep_to_push);
				}
			}
			InstrKind::Refuse(package) => if let EvalLevel::Resolve = eval.level {
				out.conflicts.push(package.get(&eval.vars)?);
			}
			InstrKind::Addon {
				id,
				file_name,
				kind,
				url,
				force,
				append,
				path,
			} => if let EvalLevel::Install = eval.level {
				let id = id.get(&eval.vars)?;
				if eval.addon_reqs.iter().any(|x| x.addon.id == id) {
					bail!("Duplicate addon id '{id}'");
				}
				if !validate_identifier(&id) {
					bail!("Invalid addon identifier '{id}'");
				}
				let file_name = match append {
					Value::None => file_name.get(&eval.vars)?,
					_ => append.get(&eval.vars)? + "-" + &file_name.get(&eval.vars)?,
				};
				let kind = kind.as_ref().expect("Addon kind missing");

				if !is_filename_valid(*kind, &file_name) {
					bail!("Invalid addon filename '{file_name}' in addon '{id}'");
				}

				let addon = Addon::new(*kind, &id, &file_name, eval.id.clone());

				if let Value::Constant(..) | Value::Var(..) = url {
					let location = AddonLocation::Remote(url.get(&eval.vars)?);
					out.addon_reqs
						.push(AddonRequest::new(addon, location, *force));
				} else if let Value::Constant(..) | Value::Var(..) = path {
					let path = path.get(&eval.vars)?;
					match eval.constants.perms {
						EvalPermissions::Elevated => {
							let path = String::from(shellexpand::tilde(&path));
							let path = PathBuf::from(path);
							let location = AddonLocation::Local(path);
							out.addon_reqs
								.push(AddonRequest::new(addon, location, *force));
						}
						_ => {
							bail!("Insufficient permissions to add a local addon '{id}'");
						}
					}
				} else {
					bail!("No location (url/path) was specified for addon '{id}'");
				}
			}
			_ => bail!("Instruction is not allowed in this routine context"),
		},
	}

	Ok(out)
}
