use anyhow::{anyhow, bail, Context};
use mcvm_parse::instruction::{InstrKind, Instruction};
use mcvm_parse::parse::{Block, Parsed};
use mcvm_parse::vars::{ReservedConstantVariables, Value, VariableStore};
use mcvm_parse::FailReason;
use mcvm_pkg::RecommendedPackage;
use mcvm_shared::pkg::{PackageAddonOptionalHashes, PkgIdentifier};

use super::conditions::eval_condition;
use super::{
	create_valid_addon_request, EvalData, EvalInput, EvalLevel, EvalPermissions, RequiredPackage,
	Routine, MAX_NOTICE_CHARACTERS, MAX_NOTICE_INSTRUCTIONS,
};

/// Result from an evaluation subfunction. Mostly used to know when to end
/// evaluation early.
pub struct EvalResult {
	finish: bool,
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

/// Evaluate a script package
pub fn eval_script_package<'a>(
	pkg_id: PkgIdentifier,
	parsed: &Parsed,
	routine: Routine,
	input: EvalInput<'a>,
) -> anyhow::Result<EvalData<'a>> {
	let routine_name = routine.get_routine_name();
	let routine_id = parsed
		.routines
		.get(&routine_name)
		.ok_or(anyhow!("Routine {} does not exist", routine_name.clone()))?;
	let block = parsed
		.blocks
		.get(routine_id)
		.ok_or(anyhow!("Routine {} does not exist", routine_name))?;

	let mut eval = EvalData::new(input, pkg_id, &routine);
	eval.vars.set_reserved_constants(ReservedConstantVariables {
		mc_version: &eval.input.constants.version,
	});

	for instr in &block.contents {
		let result = eval_instr(instr, &mut eval, parsed)?;
		if result.finish {
			break;
		}
	}

	Ok(eval)
}

/// Evaluate a block of instructions
fn eval_block(block: &Block, eval: &mut EvalData, parsed: &Parsed) -> anyhow::Result<EvalResult> {
	let mut out = EvalResult::new();

	for instr in &block.contents {
		let result = eval_instr(instr, eval, parsed)?;
		if result.finish {
			out.finish = true;
			break;
		}
	}

	Ok(out)
}

/// Evaluate an instruction
pub fn eval_instr(
	instr: &Instruction,
	eval: &mut EvalData,
	parsed: &Parsed,
) -> anyhow::Result<EvalResult> {
	let mut out = EvalResult::new();
	match eval.level {
		EvalLevel::Install | EvalLevel::Resolve => match &instr.kind {
			InstrKind::If(condition, block) => {
				if eval_condition(&condition.kind, eval)? {
					let block = parsed.blocks.get(block).expect("If block missing");
					out = eval_block(block, eval, parsed)?;
				}
			}
			InstrKind::Call(routine) => {
				let routine = routine.get();
				let routine = parsed
					.routines
					.get(routine)
					.ok_or(anyhow!("Routine '{routine}' does not exist"))?;
				let block = parsed.blocks.get(routine).expect("Block does not exist");
				out = eval_block(block, eval, parsed)?;
			}
			InstrKind::Set(var, val) => {
				let var = var.get();
				eval.vars
					.try_set_var(var.to_owned(), val.get(&eval.vars)?)
					.context("Failed to set variable")?;
			}
			InstrKind::Finish() => out.finish = true,
			InstrKind::Fail(reason) => {
				out.finish = true;
				let reason = reason.as_ref().unwrap_or(&FailReason::None).clone();
				bail!(
					"Package script failed explicitly with reason: {}",
					reason.to_string()
				);
			}
			InstrKind::Require(deps) => {
				if let EvalLevel::Resolve = eval.level {
					for dep in deps {
						let mut dep_to_push = Vec::new();
						for dep in dep {
							dep_to_push.push(RequiredPackage {
								value: dep.value.get(&eval.vars)?,
								explicit: dep.explicit,
							});
						}
						eval.deps.push(dep_to_push);
					}
				}
			}
			InstrKind::Refuse(package) => {
				if let EvalLevel::Resolve = eval.level {
					eval.conflicts.push(package.get(&eval.vars)?);
				}
			}
			InstrKind::Recommend(invert, package) => {
				if let EvalLevel::Resolve = eval.level {
					let recommendation = RecommendedPackage {
						value: package.get(&eval.vars)?,
						invert: *invert,
					};
					eval.recommendations.push(recommendation);
				}
			}
			InstrKind::Bundle(package) => {
				if let EvalLevel::Resolve = eval.level {
					eval.bundled.push(package.get(&eval.vars)?);
				}
			}
			InstrKind::Compat(package, compat) => {
				if let EvalLevel::Resolve = eval.level {
					eval.compats
						.push((package.get(&eval.vars)?, compat.get(&eval.vars)?));
				}
			}
			InstrKind::Extend(package) => {
				if let EvalLevel::Resolve = eval.level {
					eval.extensions.push(package.get(&eval.vars)?);
				}
			}
			InstrKind::Notice(notice) => {
				if eval.notices.len() > MAX_NOTICE_INSTRUCTIONS {
					bail!("Max number of notice instructions was exceded (>{MAX_NOTICE_INSTRUCTIONS})");
				}
				let notice = notice.get(&eval.vars)?;
				if notice.len() > MAX_NOTICE_CHARACTERS {
					bail!("Notice message is too long (>{MAX_NOTICE_CHARACTERS})");
				}
				eval.notices.push(notice);
			}
			InstrKind::Cmd(command) => {
				match eval.input.params.perms {
					EvalPermissions::Elevated => {}
					_ => bail!("Insufficient permissions to run the 'cmd' instruction"),
				}
				if let EvalLevel::Install = eval.level {
					let command = get_value_vec(command, &eval.vars)?;

					eval.commands.push(command);
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
				if let EvalLevel::Install = eval.level {
					let id = id.get(&eval.vars)?;
					if eval.addon_reqs.iter().any(|x| x.addon.id == id) {
						bail!("Duplicate addon id '{id}'");
					}

					let kind = kind.as_ref().expect("Addon kind missing");
					let hashes = PackageAddonOptionalHashes {
						sha256: hashes.sha256.get_as_option(&eval.vars)?,
						sha512: hashes.sha512.get_as_option(&eval.vars)?,
					};
					let addon_req = create_valid_addon_request(
						id,
						url.get_as_option(&eval.vars)?,
						path.get_as_option(&eval.vars)?,
						*kind,
						file_name.get_as_option(&eval.vars)?,
						version.get_as_option(&eval.vars)?,
						eval.id.clone(),
						hashes,
						&eval.input,
					)?;
					eval.addon_reqs.push(addon_req);
				}
			}
			_ => bail!("Instruction is not allowed in this routine context"),
		},
	}

	Ok(out)
}

/// Utility function to convert a vec of values to a vec of strings
fn get_value_vec(vec: &[Value], vars: &impl VariableStore) -> anyhow::Result<Vec<String>> {
	let out = vec.iter().map(|x| x.get(vars));
	let out = out.collect::<anyhow::Result<_>>()?;
	Ok(out)
}
