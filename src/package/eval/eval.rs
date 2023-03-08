use super::super::{Package, PkgError};
use super::instruction::{InstrKind, Instruction};
use super::parse::{Block, BlockId};
use super::Value;
use crate::data::addon::{Addon, AddonLocation, AddonRequest, Modloader, PluginLoader};
use crate::data::instance::InstKind;
use crate::io::files::paths::Paths;
use crate::package::reg::PkgIdentifier;
use crate::util::versions::VersionPattern;

use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum EvalError {
	#[error("Evaluator failed to start")]
	Start,
	#[error("Package reported an error:\n{}", .0.to_string())]
	Fail(FailReason),
	#[error("Package is not authorized to perform this action:\n{}", .0)]
	Permissions(#[from] PermissionsError),
	#[error("Variable '{}' is not defined", .0)]
	VarNotDefined(String),
	#[error("Routine '{}' does not exist", .0)]
	RoutineDoesNotExist(String),
	#[error("Expected 'url' or 'path' key for addon '{}'", .0)]
	NoAddonLocation(String),
}

#[derive(Debug, thiserror::Error)]
pub enum PermissionsError {
	#[error("Tried to access '{}' from the local filesystem", .0)]
	LocalFile(String),
}

#[derive(Debug, Clone)]
pub enum EvalLevel {
	None,
	Info,
	Dependencies,
	All,
}

impl EvalLevel {
	pub fn is_info(&self) -> bool {
		match self {
			Self::None => false,
			_ => true,
		}
	}

	pub fn is_deps(&self) -> bool {
		match self {
			Self::Dependencies | Self::All => true,
			_ => false,
		}
	}

	pub fn is_all(&self) -> bool {
		match self {
			Self::All => true,
			_ => false,
		}
	}
}

#[derive(Debug, Clone)]
// Permissions level for an evaluation
pub enum EvalPermissions {
	Restricted,
	Standard,
	Elevated,
}

impl EvalPermissions {
	pub fn from_str(string: &str) -> Option<Self> {
		match string {
			"restricted" => Some(Self::Restricted),
			"standard" => Some(Self::Standard),
			"elevated" => Some(Self::Elevated),
			_ => None,
		}
	}
}

// A routine that we will run
pub enum Routine {
	Install,
	Dependencies,
}

impl Routine {
	pub fn to_string(&self) -> String {
		String::from(match self {
			Self::Install => "install",
			Self::Dependencies => "install",
		})
	}

	pub fn get_level(&self) -> EvalLevel {
		match self {
			Self::Install => EvalLevel::All,
			Self::Dependencies => EvalLevel::Dependencies,
		}
	}
}

#[derive(Debug, Clone)]
pub enum FailReason {
	None,
	UnsupportedVersion,
	UnsupportedModloader,
}

impl FailReason {
	pub fn from_string(string: &str) -> Option<Self> {
		match string {
			"unsupported_version" => Some(Self::UnsupportedVersion),
			"unsupported_modloader" => Some(Self::UnsupportedModloader),
			_ => None,
		}
	}

	pub fn to_string(&self) -> String {
		match self {
			Self::None => String::from(""),
			Self::UnsupportedVersion => String::from("Unsupported Minecraft version"),
			Self::UnsupportedModloader => String::from("Unsupported modloader"),
		}
	}
}

#[derive(Debug, Clone)]
pub struct EvalConstants {
	pub version: String,
	pub modloader: Modloader,
	pub plugin_loader: PluginLoader,
	pub side: InstKind,
	pub features: Vec<String>,
	pub versions: Vec<String>,
	pub perms: EvalPermissions,
}

#[derive(Debug, Clone)]
pub struct EvalData {
	pub vars: HashMap<String, String>,
	pub addon_reqs: Vec<AddonRequest>,
	pub constants: EvalConstants,
	pub id: PkgIdentifier,
	pub level: EvalLevel,
	pub deps: Vec<Vec<VersionPattern>>,
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
		}
	}
}

pub struct EvalResult {
	vars_to_set: HashMap<String, String>,
	finish: bool,
	addon_reqs: Vec<AddonRequest>,
	deps: Vec<Vec<VersionPattern>>,
}

impl EvalResult {
	pub fn new() -> Self {
		Self {
			vars_to_set: HashMap::new(),
			finish: false,
			addon_reqs: Vec::new(),
			deps: Vec::new(),
		}
	}

	pub fn merge(&mut self, other: EvalResult) {
		self.vars_to_set.extend(other.vars_to_set);
		self.finish = other.finish;
		self.addon_reqs.extend(other.addon_reqs);
		self.deps.extend(other.deps);
	}
}

impl Package {
	pub async fn eval(
		&mut self,
		paths: &Paths,
		routine: Routine,
		constants: EvalConstants,
	) -> Result<EvalData, PkgError> {
		self.ensure_loaded(paths, false)?;
		self.parse(paths)?;
		if let Some(data) = &mut self.data {
			if let Some(parsed) = &mut data.parsed {
				let routine_name = routine.to_string();
				let routine_id = parsed
					.routines
					.get(&routine_name)
					.ok_or(EvalError::RoutineDoesNotExist(routine_name.clone()))?;
				let block = parsed
					.blocks
					.get(routine_id)
					.ok_or(EvalError::RoutineDoesNotExist(routine_name))?;

				let mut eval = EvalData::new(constants, self.id.clone(), &routine);

				match eval.level {
					EvalLevel::All | EvalLevel::Info | EvalLevel::Dependencies => {
						for instr in &block.contents {
							let result = instr.eval(&eval, &parsed.blocks)?;
							for (var, val) in result.vars_to_set {
								eval.vars.insert(var, val);
							}
							eval.addon_reqs.extend(result.addon_reqs);
							eval.deps.extend(result.deps);
							if result.finish {
								break;
							}
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

fn eval_block(
	block: &Block,
	eval: &EvalData,
	blocks: &HashMap<BlockId, Block>,
) -> Result<EvalResult, EvalError> {
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
	pub fn eval(
		&self,
		eval: &EvalData,
		blocks: &HashMap<BlockId, Block>,
	) -> Result<EvalResult, EvalError> {
		let mut out = EvalResult::new();
		if eval.level.is_all() {
			match &self.kind {
				InstrKind::If(condition, block) => {
					if condition.kind.eval(eval)? {
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
					return Err(EvalError::Fail(
						reason.as_ref().unwrap_or(&FailReason::None).clone(),
					));
				}
				InstrKind::Addon {
					name,
					kind,
					url,
					force,
					append,
					path,
				} => {
					let name = match append {
						Value::None => name.get(&eval.vars)?,
						_ => append.get(&eval.vars)? + "-" + &name.get(&eval.vars)?,
					};
					let addon = Addon::new(
						kind.as_ref().expect("Addon kind missing").clone(),
						&name,
						eval.id.clone(),
					);

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
								return Err(EvalError::Permissions(PermissionsError::LocalFile(
									path,
								)))
							}
						}
					} else {
						return Err(EvalError::NoAddonLocation(name));
					}
				}
				_ => {}
			}
		}
		if eval.level.is_deps() {
			match &self.kind {
				InstrKind::Rely(deps, ..) => {
					for dep in deps {
						let mut dep_to_push = Vec::new();
						for dep in dep {
							dep_to_push.push(VersionPattern::from(&dep.get(&eval.vars)?));
						}
						out.deps.push(dep_to_push);
					}
				}
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
