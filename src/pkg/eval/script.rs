use anyhow::bail;
use mcvm_parse::conditions::ConditionKind;
use mcvm_parse::parse::Parsed;
use mcvm_parse::vars::{HashMapVariableStore, ReservedConstantVariables, VariableStore};
use mcvm_pkg::properties::PackageProperties;
use mcvm_pkg::script_eval::{
	AddonInstructionData, ScriptEvalConfig, ScriptEvaluator as ScriptEvaluatorTrait,
};
use mcvm_pkg::RecommendedPackage;
use mcvm_plugin::hooks::{CustomPackageInstruction, CustomPackageInstructionArg};
use mcvm_shared::output::NoOp;
use mcvm_shared::pkg::PackageID;

use crate::io::paths::Paths;
use crate::plugin::PluginManager;

use super::conditions::eval_condition;
use super::{
	create_valid_addon_request, EvalData, EvalInput, EvalPermissions, RequiredPackage, Routine,
	MAX_NOTICE_CHARACTERS, MAX_NOTICE_INSTRUCTIONS,
};

struct SharedData<'a> {
	eval: EvalData<'a>,
	paths: &'a Paths,
}

/// Evaluate a script package
pub fn eval_script_package<'a>(
	pkg_id: PackageID,
	parsed: &Parsed,
	routine: Routine,
	properties: PackageProperties,
	input: EvalInput<'a>,
	plugins: PluginManager,
	paths: &'a Paths,
) -> anyhow::Result<EvalData<'a>> {
	let mut eval = EvalData::new(input, pkg_id, properties, &routine, plugins);

	eval.vars.set_reserved_constants(ReservedConstantVariables {
		mc_version: &eval.input.constants.version,
	});

	let reason = eval.reason;
	let mut data = SharedData { eval, paths };

	mcvm_pkg::script_eval::eval_script_package(
		parsed,
		&mut ScriptEvaluator,
		&mut data,
		&ScriptEvalConfig { reason },
	)?;

	Ok(data.eval)
}

struct ScriptEvaluator;

impl ScriptEvaluatorTrait for ScriptEvaluator {
	type Shared<'a> = SharedData<'a>;
	type VariableStore = HashMapVariableStore;

	fn get_variable_store<'a>(
		&self,
		shared: &'a mut Self::Shared<'_>,
	) -> &'a mut Self::VariableStore {
		&mut shared.eval.vars
	}

	fn eval_condition(
		&mut self,
		shared: &mut Self::Shared<'_>,
		condition: &ConditionKind,
	) -> anyhow::Result<bool> {
		eval_condition(condition, &shared.eval)
	}

	fn add_addon(
		&mut self,
		shared: &mut Self::Shared<'_>,
		addon: AddonInstructionData,
	) -> anyhow::Result<()> {
		if shared
			.eval
			.addon_reqs
			.iter()
			.any(|x| x.addon.id == addon.id)
		{
			bail!("Duplicate addon id '{}'", addon.id);
		}

		let addon_req =
			create_valid_addon_request(addon, shared.eval.id.clone(), &shared.eval.input)?;
		shared.eval.addon_reqs.push(addon_req);

		Ok(())
	}

	fn add_bundled(&mut self, shared: &mut Self::Shared<'_>, pkg: PackageID) -> anyhow::Result<()> {
		shared.eval.bundled.push(pkg);
		Ok(())
	}

	fn add_command(
		&mut self,
		shared: &mut Self::Shared<'_>,
		command: Vec<String>,
	) -> anyhow::Result<()> {
		match shared.eval.input.params.perms {
			EvalPermissions::Elevated => {}
			_ => bail!("Insufficient permissions to run the 'cmd' instruction"),
		}
		shared.eval.commands.push(command);
		Ok(())
	}

	fn add_compat(
		&mut self,
		shared: &mut Self::Shared<'_>,
		compat: (PackageID, PackageID),
	) -> anyhow::Result<()> {
		shared.eval.compats.push(compat);
		Ok(())
	}

	fn add_conflict(
		&mut self,
		shared: &mut Self::Shared<'_>,
		pkg: PackageID,
	) -> anyhow::Result<()> {
		shared.eval.conflicts.push(pkg);
		Ok(())
	}

	fn add_dependency(
		&mut self,
		shared: &mut Self::Shared<'_>,
		dep: Vec<RequiredPackage>,
	) -> anyhow::Result<()> {
		shared.eval.deps.push(dep);
		Ok(())
	}

	fn add_extension(
		&mut self,
		shared: &mut Self::Shared<'_>,
		pkg: PackageID,
	) -> anyhow::Result<()> {
		shared.eval.extensions.push(pkg);
		Ok(())
	}

	fn add_notice(&mut self, shared: &mut Self::Shared<'_>, notice: String) -> anyhow::Result<()> {
		if shared.eval.notices.len() > MAX_NOTICE_INSTRUCTIONS {
			bail!("Max number of notice instructions was exceded (>{MAX_NOTICE_INSTRUCTIONS})");
		}
		if notice.len() > MAX_NOTICE_CHARACTERS {
			bail!("Notice message is too long (>{MAX_NOTICE_CHARACTERS})");
		}
		shared.eval.notices.push(notice);
		Ok(())
	}

	fn add_recommendation(
		&mut self,
		shared: &mut Self::Shared<'_>,
		pkg: RecommendedPackage,
	) -> anyhow::Result<()> {
		shared.eval.recommendations.push(pkg);
		Ok(())
	}

	fn run_custom(
		&mut self,
		shared: &mut Self::Shared<'_>,
		command: String,
		args: Vec<String>,
	) -> anyhow::Result<()> {
		let arg = CustomPackageInstructionArg {
			pkg_id: shared.eval.id.to_string(),
			command,
			args,
		};
		let results = shared.eval.plugins.call_hook(
			CustomPackageInstruction,
			&arg,
			shared.paths,
			&mut NoOp,
		)?;

		if results.is_empty() {
			shared.eval.uses_custom_instructions = true;
		}

		for result in results {
			let result = result.result(&mut NoOp)?;
			if !result.handled {
				shared.eval.uses_custom_instructions = true;
			}

			for addon in result.addon_reqs {
				self.add_addon(shared, addon)?;
			}
			for bundled in result.bundled {
				self.add_bundled(shared, bundled)?;
			}
			for conflict in result.conflicts {
				self.add_conflict(shared, conflict)?;
			}
			for dep in result.deps {
				self.add_dependency(shared, dep)?;
			}
			for compat in result.compats {
				self.add_compat(shared, compat)?;
			}
			for notice in result.notices {
				self.add_notice(shared, notice)?;
			}
			for extension in result.extensions {
				self.add_extension(shared, extension)?;
			}
			for recommendation in result.recommendations {
				self.add_recommendation(shared, recommendation)?;
			}
		}

		Ok(())
	}
}
