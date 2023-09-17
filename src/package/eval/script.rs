use anyhow::bail;
use mcvm_parse::conditions::ConditionKind;
use mcvm_parse::parse::Parsed;
use mcvm_parse::vars::{HashMapVariableStore, ReservedConstantVariables, VariableStore};
use mcvm_pkg::script_eval::{
	AddonInstructionData, ScriptEvalConfig, ScriptEvaluator as ScriptEvaluatorTrait,
};
use mcvm_pkg::RecommendedPackage;
use mcvm_shared::pkg::PackageID;

use super::conditions::eval_condition;
use super::{
	create_valid_addon_request, EvalData, EvalInput, EvalPermissions, RequiredPackage, Routine,
	MAX_NOTICE_CHARACTERS, MAX_NOTICE_INSTRUCTIONS,
};

/// Evaluate a script package
pub fn eval_script_package<'a>(
	pkg_id: PackageID,
	parsed: &Parsed,
	routine: Routine,
	input: EvalInput<'a>,
) -> anyhow::Result<EvalData<'a>> {
	let mut eval = EvalData::new(input, pkg_id, &routine);
	
	eval.vars.set_reserved_constants(ReservedConstantVariables {
		mc_version: &eval.input.constants.version,
	});

	let reason = eval.reason;

	mcvm_pkg::script_eval::eval_script_package(
		parsed,
		&mut ScriptEvaluator,
		&mut eval,
		&ScriptEvalConfig { reason },
	)?;

	Ok(eval)
}

struct ScriptEvaluator;

impl ScriptEvaluatorTrait for ScriptEvaluator {
	type Shared<'a> = EvalData<'a>;
	type VariableStore = HashMapVariableStore;

	fn get_variable_store<'a>(
		&self,
		shared: &'a mut Self::Shared<'_>,
	) -> &'a mut Self::VariableStore {
		&mut shared.vars
	}

	fn eval_condition(
		&mut self,
		shared: &mut Self::Shared<'_>,
		condition: &ConditionKind,
	) -> anyhow::Result<bool> {
		eval_condition(condition, shared)
	}

	fn add_addon(
		&mut self,
		shared: &mut Self::Shared<'_>,
		addon: AddonInstructionData,
	) -> anyhow::Result<()> {
		if shared.addon_reqs.iter().any(|x| x.addon.id == addon.id) {
			bail!("Duplicate addon id '{}'", addon.id);
		}

		let addon_req = create_valid_addon_request(addon, shared.id.clone(), &shared.input)?;
		shared.addon_reqs.push(addon_req);

		Ok(())
	}

	fn add_bundled(&mut self, shared: &mut Self::Shared<'_>, pkg: PackageID) -> anyhow::Result<()> {
		shared.bundled.push(pkg);
		Ok(())
	}

	fn add_command(
		&mut self,
		shared: &mut Self::Shared<'_>,
		command: Vec<String>,
	) -> anyhow::Result<()> {
		match shared.input.params.perms {
			EvalPermissions::Elevated => {}
			_ => bail!("Insufficient permissions to run the 'cmd' instruction"),
		}
		shared.commands.push(command);
		Ok(())
	}

	fn add_compat(
		&mut self,
		shared: &mut Self::Shared<'_>,
		compat: (PackageID, PackageID),
	) -> anyhow::Result<()> {
		shared.compats.push(compat);
		Ok(())
	}

	fn add_conflict(
		&mut self,
		shared: &mut Self::Shared<'_>,
		pkg: PackageID,
	) -> anyhow::Result<()> {
		shared.conflicts.push(pkg);
		Ok(())
	}

	fn add_dependency(
		&mut self,
		shared: &mut Self::Shared<'_>,
		dep: Vec<RequiredPackage>,
	) -> anyhow::Result<()> {
		shared.deps.push(dep);
		Ok(())
	}

	fn add_extension(
		&mut self,
		shared: &mut Self::Shared<'_>,
		pkg: PackageID,
	) -> anyhow::Result<()> {
		shared.extensions.push(pkg);
		Ok(())
	}

	fn add_notice(&mut self, shared: &mut Self::Shared<'_>, notice: String) -> anyhow::Result<()> {
		if shared.notices.len() > MAX_NOTICE_INSTRUCTIONS {
			bail!("Max number of notice instructions was exceded (>{MAX_NOTICE_INSTRUCTIONS})");
		}
		if notice.len() > MAX_NOTICE_CHARACTERS {
			bail!("Notice message is too long (>{MAX_NOTICE_CHARACTERS})");
		}
		shared.notices.push(notice);
		Ok(())
	}

	fn add_recommendation(
		&mut self,
		shared: &mut Self::Shared<'_>,
		pkg: RecommendedPackage,
	) -> anyhow::Result<()> {
		shared.recommendations.push(pkg);
		Ok(())
	}
}
