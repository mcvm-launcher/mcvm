use mcvm_shared::{instance::Side, versions::VersionPattern};

use super::EvalData;
use mcvm_parse::conditions::ConditionKind;

pub fn eval_condition(condition: &ConditionKind, eval: &EvalData) -> anyhow::Result<bool> {
	match condition {
		ConditionKind::Not(condition) => {
			eval_condition(condition.as_ref().expect("Not condition is missing"), eval)
				.map(|op| !op)
		}
		ConditionKind::Version(version) => {
			let version = version.get(&eval.vars)?;
			let version = VersionPattern::from(&version);
			Ok(version.matches_single(&eval.constants.version, &eval.constants.versions))
		}
		ConditionKind::Side(side) => {
			Ok(eval.constants.side == *side.as_ref().expect("If side is missing"))
		}
		ConditionKind::Modloader(loader) => {
			Ok(loader.as_ref().expect("If modloader is missing").matches(
				&eval
					.constants
					.modifications
					.get_modloader(eval.constants.side),
			))
		}
		ConditionKind::PluginLoader(loader) => Ok(loader
			.as_ref()
			.expect("If plugin_loader is missing")
			.matches(&eval.constants.modifications.server_type)
			&& matches!(eval.constants.side, Side::Server)),
		ConditionKind::Feature(feature) => {
			Ok(eval.constants.features.contains(&feature.get(&eval.vars)?))
		}
		ConditionKind::Value(left, right) => Ok(left.get(&eval.vars)? == right.get(&eval.vars)?),
	}
}
