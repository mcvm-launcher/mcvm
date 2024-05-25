use mcvm_shared::versions::VersionPattern;
use mcvm_shared::Side;

use super::EvalData;
use mcvm_parse::conditions::{ArchCondition, ConditionKind, OSCondition};
use mcvm_parse::vars::VariableStore;

/// Evaluates a script condition to a boolean
pub fn eval_condition(condition: &ConditionKind, eval: &EvalData) -> anyhow::Result<bool> {
	match condition {
		ConditionKind::Not(condition) => eval_condition(condition.get(), eval).map(|op| !op),
		ConditionKind::And(left, right) => {
			Ok(eval_condition(left, eval)? && eval_condition(right.get(), eval)?)
		}
		ConditionKind::Or(left, right) => {
			Ok(eval_condition(left, eval)? || eval_condition(right.get(), eval)?)
		}
		ConditionKind::Version(version) => {
			let version = version.get(&eval.vars)?;
			let version = VersionPattern::from(&version);
			Ok(version.matches_single(
				&eval.input.constants.version,
				&eval.input.constants.version_list,
			))
		}
		ConditionKind::Side(side) => Ok(eval.input.params.side == *side.get()),
		ConditionKind::Modloader(loader) => Ok(loader.get().matches(
			&eval
				.input
				.constants
				.modifications
				.get_modloader(eval.input.params.side),
		)),
		ConditionKind::PluginLoader(loader) => Ok(loader
			.get()
			.matches(&eval.input.constants.modifications.server_type)
			&& matches!(eval.input.params.side, Side::Server)),
		ConditionKind::Feature(feature) => Ok(eval
			.input
			.params
			.features
			.contains(&feature.get(&eval.vars)?)),
		ConditionKind::OS(os) => Ok(check_os_condition(os.get())),
		ConditionKind::Arch(arch) => Ok(check_arch_condition(arch.get())),
		ConditionKind::Stability(stability) => Ok(eval.input.params.stability == *stability.get()),
		ConditionKind::Language(lang) => Ok(eval.input.constants.language == *lang.get()),
		ConditionKind::ContentVersion(version) => {
			let version = version.get(&eval.vars)?;
			let version = VersionPattern::from(&version);
			let _ = version;
			// TODO
			Ok(true)
		}
		ConditionKind::Value(left, right) => Ok(left.get(&eval.vars)? == right.get(&eval.vars)?),
		ConditionKind::Defined(var) => Ok(eval.vars.var_exists(var.get())),
		ConditionKind::Const(val) => Ok(val.get_clone()),
	}
}

/// Checks an OS condition to see if it matches the current operating system
pub fn check_os_condition(condition: &OSCondition) -> bool {
	match condition {
		OSCondition::Windows => cfg!(target_os = "windows"),
		OSCondition::Linux => cfg!(target_os = "linux"),
		OSCondition::MacOS => cfg!(target_os = "macos"),
		OSCondition::Unix => cfg!(target_family = "unix"),
		OSCondition::Other => {
			!(cfg!(target_os = "windows") || cfg!(target_os = "linux") || cfg!(target_os = "macos"))
		}
	}
}

/// Checks an arch condition to see if it matches the current system architecture
pub fn check_arch_condition(condition: &ArchCondition) -> bool {
	if cfg!(target_arch = "x86") {
		return condition == &ArchCondition::X86;
	}
	if cfg!(target_arch = "x86_64") {
		return condition == &ArchCondition::X86_64;
	}
	if cfg!(target_arch = "arm") {
		return condition == &ArchCondition::Arm;
	}
	condition == &ArchCondition::Other
}
