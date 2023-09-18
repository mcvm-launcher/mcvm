use mcvm_parse::conditions::ConditionKind;
use mcvm_parse::instruction::{InstrKind, Instruction};
use mcvm_parse::parse::{Block, Parsed};
use mcvm_parse::routine::INSTALL_ROUTINE;
use mcvm_parse::vars::{Value, VariableStore};
use mcvm_parse::FailReason;

use anyhow::{anyhow, bail, Context};
use mcvm_shared::addon::AddonKind;
use mcvm_shared::pkg::{PackageAddonOptionalHashes, PackageID};

use crate::{RecommendedPackage, RequiredPackage};

/// A trait for a type that has specific implementations of package script evaluation functions
pub trait ScriptEvaluator {
	/// The shared evaluation state to provide to methods
	type Shared<'a>;
	/// The type of variable store to use
	type VariableStore: VariableStore;

	/// Get the evaluator's variable store from the shared data
	fn get_variable_store<'a>(
		&self,
		shared: &'a mut Self::Shared<'_>,
	) -> &'a mut Self::VariableStore;

	/// Evaluate a condition
	fn eval_condition(
		&mut self,
		shared: &mut Self::Shared<'_>,
		condition: &ConditionKind,
	) -> anyhow::Result<bool>;

	/// Add a dependency
	fn add_dependency(
		&mut self,
		shared: &mut Self::Shared<'_>,
		dep: Vec<RequiredPackage>,
	) -> anyhow::Result<()>;

	/// Add a conflict
	fn add_conflict(&mut self, shared: &mut Self::Shared<'_>, pkg: PackageID)
		-> anyhow::Result<()>;

	/// Add a recommendation
	fn add_recommendation(
		&mut self,
		shared: &mut Self::Shared<'_>,
		pkg: RecommendedPackage,
	) -> anyhow::Result<()>;

	/// Add a bundled package
	fn add_bundled(&mut self, shared: &mut Self::Shared<'_>, pkg: PackageID) -> anyhow::Result<()>;

	/// Add a compat
	fn add_compat(
		&mut self,
		shared: &mut Self::Shared<'_>,
		compat: (PackageID, PackageID),
	) -> anyhow::Result<()>;

	/// Add an extension
	fn add_extension(
		&mut self,
		shared: &mut Self::Shared<'_>,
		pkg: PackageID,
	) -> anyhow::Result<()>;

	/// Add a notice
	fn add_notice(&mut self, shared: &mut Self::Shared<'_>, notice: String) -> anyhow::Result<()>;

	/// Add a command
	fn add_command(
		&mut self,
		shared: &mut Self::Shared<'_>,
		command: Vec<String>,
	) -> anyhow::Result<()>;

	/// Add an addon
	fn add_addon(
		&mut self,
		shared: &mut Self::Shared<'_>,
		addon: AddonInstructionData,
	) -> anyhow::Result<()>;

	/// Run a custom instruction
	fn run_custom(&mut self, shared: &mut Self::Shared<'_>, custom: String) -> anyhow::Result<()>;
}

/// Configuration for script evaluation
pub struct ScriptEvalConfig {
	/// The reason for evaluation
	pub reason: EvalReason,
}

/// For what reason we are evaluating the script, which determines
/// what instructions we run
#[derive(Debug, Clone, Copy)]
pub enum EvalReason {
	/// Installing addons
	Install,
	/// Resolving relations
	Resolve,
}

/// Evaluate a script package install routine with a script evaluator
pub fn eval_script_package<E: ScriptEvaluator>(
	parsed: &Parsed,
	e: &mut E,
	shared: &mut E::Shared<'_>,
	config: &ScriptEvalConfig,
) -> anyhow::Result<()> {
	let routine_id = parsed.routines.get(INSTALL_ROUTINE).ok_or(anyhow!(
		"Routine {} does not exist",
		INSTALL_ROUTINE.clone()
	))?;
	let block = parsed
		.blocks
		.get(routine_id)
		.ok_or(anyhow!("Routine {} does not exist", INSTALL_ROUTINE))?;

	for instr in &block.contents {
		let result = eval_instr(instr, parsed, e, shared, config)?;
		if result.finish {
			break;
		}
	}
	Ok(())
}

/// Evaluate a block of instructions
fn eval_block<E: ScriptEvaluator>(
	block: &Block,
	parsed: &Parsed,
	e: &mut E,
	shared: &mut E::Shared<'_>,
	config: &ScriptEvalConfig,
) -> anyhow::Result<EvalResult> {
	let mut out = EvalResult::new();

	for instr in &block.contents {
		let result = eval_instr(instr, parsed, e, shared, config)?;
		if result.finish {
			out.finish = true;
			break;
		}
	}

	Ok(out)
}

/// Evaluate an instruction
pub fn eval_instr<E: ScriptEvaluator>(
	instr: &Instruction,
	parsed: &Parsed,
	e: &mut E,
	shared: &mut E::Shared<'_>,
	config: &ScriptEvalConfig,
) -> anyhow::Result<EvalResult> {
	let mut out = EvalResult::new();

	// Used to put a nice anyhow context on all of them
	let result = {
		match config.reason {
			EvalReason::Install | EvalReason::Resolve => match &instr.kind {
				InstrKind::If {
					condition,
					if_block,
					else_blocks,
				} => {
					if e.eval_condition(shared, &condition.kind)? {
						let block = parsed.blocks.get(if_block).expect("If block missing");
						out = eval_block(block, parsed, e, shared, config)?;
					} else {
						// Eval the else block chain
						for else_block in else_blocks {
							if let Some(condition) = &else_block.condition {
								if !e.eval_condition(shared, &condition.kind)? {
									continue;
								}
							}
							let block = parsed
								.blocks
								.get(&else_block.block)
								.expect("If else block missing");
							out = eval_block(block, parsed, e, shared, config)?;
						}
					}
				}
				InstrKind::Call(routine) => {
					let routine = routine.get();
					let routine = parsed.routines.get(routine).ok_or(anyhow!(
						"Call instruction routine '{routine}' does not exist"
					))?;
					let block = parsed.blocks.get(routine).expect("Block does not exist");
					out = eval_block(block, parsed, e, shared, config)?;
				}
				InstrKind::Set(var, val) => {
					let var = var.get();
					let val = val.get(e.get_variable_store(shared))?;
					e.get_variable_store(shared)
						.try_set_var(var.to_owned(), val)
						.with_context(|| "Failed to set variable".to_string())?;
				}
				InstrKind::Finish() => out.finish = true,
				InstrKind::Fail(reason) => {
					out.finish = true;
					let reason = reason.as_ref().unwrap_or(&FailReason::None).clone();
					bail!(
						"Package script failed explicitly with reason: {}",
						reason.to_string(),
					);
				}
				InstrKind::Require(deps) => {
					if let EvalReason::Resolve = config.reason {
						for dep in deps {
							let mut dep_to_push = Vec::new();
							for dep in dep {
								dep_to_push.push(RequiredPackage {
									value: dep.value.get(e.get_variable_store(shared))?.into(),
									explicit: dep.explicit,
								});
							}
							e.add_dependency(shared, dep_to_push)?;
						}
					}
				}
				InstrKind::Refuse(package) => {
					if let EvalReason::Resolve = config.reason {
						let package = package.get(e.get_variable_store(shared))?;
						e.add_conflict(shared, package.into())?;
					}
				}
				InstrKind::Recommend(invert, package) => {
					if let EvalReason::Resolve = config.reason {
						let recommendation = RecommendedPackage {
							value: package.get(e.get_variable_store(shared))?.into(),
							invert: *invert,
						};
						e.add_recommendation(shared, recommendation)?;
					}
				}
				InstrKind::Bundle(package) => {
					if let EvalReason::Resolve = config.reason {
						let package = package.get(e.get_variable_store(shared))?;
						e.add_bundled(shared, package.into())?;
					}
				}
				InstrKind::Compat(package, compat) => {
					if let EvalReason::Resolve = config.reason {
						let package = package.get(e.get_variable_store(shared))?;
						let compat = compat.get(e.get_variable_store(shared))?;
						e.add_compat(shared, (package.into(), compat.into()))?;
					}
				}
				InstrKind::Extend(package) => {
					if let EvalReason::Resolve = config.reason {
						let package = package.get(e.get_variable_store(shared))?;
						e.add_extension(shared, package.into())?;
					}
				}
				InstrKind::Notice(notice) => {
					let notice = notice.get(e.get_variable_store(shared))?;
					e.add_notice(shared, notice)?;
				}
				InstrKind::Cmd(command) => {
					if let EvalReason::Install = config.reason {
						let command = get_value_vec(command, e.get_variable_store(shared))?;

						e.add_command(shared, command)?;
					}
				}
				InstrKind::Addon {
					id,
					file_name,
					kind,
					url,
					path,
					version,
					hashes,
				} => {
					if let EvalReason::Install = config.reason {
						let id = id.get(e.get_variable_store(shared))?;
						let kind = kind.as_ref().expect("Addon kind missing");
						let hashes = PackageAddonOptionalHashes {
							sha256: hashes.sha256.get_as_option(e.get_variable_store(shared))?,
							sha512: hashes.sha512.get_as_option(e.get_variable_store(shared))?,
						};
						let data = AddonInstructionData {
							id,
							file_name: file_name.get_as_option(e.get_variable_store(shared))?,
							kind: *kind,
							url: url.get_as_option(e.get_variable_store(shared))?,
							path: path.get_as_option(e.get_variable_store(shared))?,
							version: version.get_as_option(e.get_variable_store(shared))?,
							hashes,
						};
						e.add_addon(shared, data)?;
					}
				}
				_ => bail!("Instruction is not allowed in this routine context"),
			},
		}
		Ok::<(), anyhow::Error>(())
	};

	result.with_context(|| format!("In {} instruction at {}", instr, instr.pos))?;

	Ok(out)
}

/// Result from an evaluation subfunction. Mostly used to know when to end
/// evaluation early.
pub struct EvalResult {
	/// Whether to finish evaluation early
	pub finish: bool,
}

impl EvalResult {
	/// Creates a new EvalResult
	pub fn new() -> Self {
		Self { finish: false }
	}
}

impl Default for EvalResult {
	fn default() -> Self {
		Self::new()
	}
}

/// Utility function to convert a vec of values to a vec of strings
fn get_value_vec(vec: &[Value], vars: &impl VariableStore) -> anyhow::Result<Vec<String>> {
	let out = vec.iter().map(|x| x.get(vars));
	let out = out.collect::<anyhow::Result<_>>()?;
	Ok(out)
}

/// Data for implementing the addon instruction
pub struct AddonInstructionData {
	/// The ID of the addon
	pub id: String,
	/// The filename of the addon
	pub file_name: Option<String>,
	/// What kind of addon this is
	pub kind: AddonKind,
	/// The URL to the addon file; may not exist
	pub url: Option<String>,
	/// The path to the addon file; may not exist
	pub path: Option<String>,
	/// The version of the addon
	pub version: Option<String>,
	/// The addon's hashes
	pub hashes: PackageAddonOptionalHashes,
}
