use mcvm_shared::{instance::Side, versions::VersionPattern};

use super::EvalData;
use mcvm_parse::conditions::{ConditionKind, OSCondition};

pub fn eval_condition(condition: &ConditionKind, eval: &EvalData) -> anyhow::Result<bool> {
	match condition {
		ConditionKind::Not(condition) => {
			eval_condition(condition.get(), eval)
				.map(|op| !op)
		}
		ConditionKind::And(left, right) => Ok(eval_condition(left, eval)?
			&& eval_condition(
				right.get(),
				eval,
			)?),
		ConditionKind::Or(left, right) => Ok(eval_condition(left, eval)?
			|| eval_condition(right.get(), eval)?),
		ConditionKind::Version(version) => {
			let version = version.get(&eval.vars)?;
			let version = VersionPattern::from(&version);
			Ok(version.matches_single(
				&eval.input.constants.version,
				&eval.input.constants.version_list,
			))
		}
		ConditionKind::Side(side) => {
			Ok(eval.input.params.side == *side.get())
		}
		ConditionKind::Modloader(loader) => {
			Ok(loader.get().matches(
				&eval
					.input
					.constants
					.modifications
					.get_modloader(eval.input.params.side),
			))
		}
		ConditionKind::PluginLoader(loader) => Ok(loader
			.get()
			.matches(&eval.input.constants.modifications.server_type)
			&& matches!(eval.input.params.side, Side::Server)),
		ConditionKind::Feature(feature) => Ok(eval
			.input
			.constants
			.features
			.contains(&feature.get(&eval.vars)?)),
		ConditionKind::OS(os) => Ok(match os.get() {
			OSCondition::Windows => cfg!(target_os = "windows"),
			OSCondition::Linux => cfg!(target_os = "linux"),
			OSCondition::MacOS => cfg!(target_os = "macos"),
			OSCondition::Other => {
				!(cfg!(target_os = "windows")
					|| cfg!(target_os = "linux")
					|| cfg!(target_os = "macos"))
			}
		}),
		ConditionKind::Stability(stability) => {
			Ok(eval.input.params.stability == *stability.get())
		}
		ConditionKind::Language(lang) => {
			Ok(eval.input.constants.language == *lang.get())
		}
		ConditionKind::Value(left, right) => Ok(left.get(&eval.vars)? == right.get(&eval.vars)?),
		ConditionKind::Defined(var) => Ok(eval
			.vars
			.contains_key(var.get())),
	}
}
