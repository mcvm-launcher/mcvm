use mcvm_shared::{instance::Side, versions::VersionPattern};

use super::EvalData;
use mcvm_parse::{
	conditions::{ConditionKind, OsCondition},
	Value,
};

pub fn eval_condition(condition: &ConditionKind, eval: &EvalData) -> anyhow::Result<bool> {
	match condition {
		ConditionKind::Not(condition) => {
			eval_condition(condition.as_ref().expect("Not condition is missing"), eval)
				.map(|op| !op)
		}
		ConditionKind::And(left, right) => Ok(eval_condition(left, eval)?
			&& eval_condition(
				right.as_ref().expect("Right and condition is missing"),
				eval,
			)?),
		ConditionKind::Or(left, right) => Ok(eval_condition(left, eval)?
			|| eval_condition(right.as_ref().expect("Right or condition is missing"), eval)?),
		ConditionKind::Version(version) => {
			let version = version.get(&eval.vars)?;
			let version = VersionPattern::from(&version);
			Ok(version.matches_single(&eval.constants.version, &eval.constants.version_list))
		}
		ConditionKind::Side(side) => {
			Ok(eval.params.side == *side.as_ref().expect("If side is missing"))
		}
		ConditionKind::Modloader(loader) => Ok(loader
			.as_ref()
			.expect("If modloader is missing")
			.matches(&eval.constants.modifications.get_modloader(eval.params.side))),
		ConditionKind::PluginLoader(loader) => Ok(loader
			.as_ref()
			.expect("If plugin_loader is missing")
			.matches(&eval.constants.modifications.server_type)
			&& matches!(eval.params.side, Side::Server)),
		ConditionKind::Feature(feature) => {
			Ok(eval.constants.features.contains(&feature.get(&eval.vars)?))
		}
		ConditionKind::Os(os) => Ok(match os.as_ref().expect("If OS is missing") {
			OsCondition::Windows => cfg!(windows),
			OsCondition::Linux => cfg!(linux),
			OsCondition::Other => !(cfg!(windows) || cfg!(linux)),
		}),
		ConditionKind::Value(left, right) => Ok(left.get(&eval.vars)? == right.get(&eval.vars)?),
		ConditionKind::Defined(value) => Ok(match value {
			Value::None => false,
			Value::Constant(..) => true,
			Value::Var(var) => eval.vars.contains_key(var),
		}),
	}
}
