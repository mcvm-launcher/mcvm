use anyhow::{anyhow, bail};
use serde::Deserialize;

use super::super::Package;
use super::instruction::{InstrKind, Instruction};
use super::parse::{Block, BlockId};
use super::Value;
use crate::data::addon::{Addon, AddonLocation, AddonRequest, Modloader, PluginLoader};
use crate::data::instance::Side;
use crate::io::files::paths::Paths;
use crate::package::reg::PkgIdentifier;
use crate::util::versions::VersionPattern;

use std::collections::HashMap;
use std::fmt::Display;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum EvalLevel {
	None,
	Info,
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
			Self::All => true,
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

// Permissions level for an evaluation
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum EvalPermissions {
	Restricted,
	#[default]
	Standard,
	Elevated,
}

// A routine that we will run
pub enum Routine {
	Install,
}

impl Routine {
	pub fn get_routine_name(&self) -> String {
		String::from(match self {
			Self::Install => "install",
		})
	}

	pub fn get_level(&self) -> EvalLevel {
		match self {
			Self::Install => EvalLevel::All,
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
}

impl Display for FailReason {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::None => "",
				Self::UnsupportedVersion => "Unsupported Minecraft version",
				Self::UnsupportedModloader => "Unsupported modloader",
			}
		)
	}
}

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
	) -> anyhow::Result<EvalData> {
		self.ensure_loaded(paths, false).await?;
		self.parse(paths).await?;
		if let Some(data) = &mut self.data {
			if let Some(parsed) = &mut data.parsed {
				let routine_name = routine.get_routine_name();
				let routine_id = parsed
					.routines
					.get(&routine_name)
					.ok_or(anyhow!("Routine {} does not exist", routine_name.clone()))?;
				let block = parsed
					.blocks
					.get(routine_id)
					.ok_or(anyhow!("Routine {} does not exist", routine_name))?;

				let mut eval = EvalData::new(constants, self.id.clone(), &routine);

				match eval.level {
					EvalLevel::All | EvalLevel::Info => {
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
		bail!("Evaluator failed to start")
	}
}

fn eval_block(
	block: &Block,
	eval: &EvalData,
	blocks: &HashMap<BlockId, Block>,
) -> anyhow::Result<EvalResult> {
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
	) -> anyhow::Result<EvalResult> {
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
					let reason = reason.as_ref().unwrap_or(&FailReason::None).clone();
					bail!("Package script failed with reason: {}", reason.to_string());
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
								bail!("Insufficient permissions to add a local addon {name}");
							}
						}
					} else {
						bail!("No location (url/path) was specified for addon {name}");
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
