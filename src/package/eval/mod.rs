pub mod conditions;
/// Used for evaluating declarative packages
pub mod declarative;
pub mod resolve;

use anyhow::{anyhow, bail};
use mcvm_parse::properties::PackageProperties;
use mcvm_parse::routine::INSTALL_ROUTINE;
use mcvm_pkg::PackageContentType;
use mcvm_shared::addon::{is_addon_version_valid, is_filename_valid, Addon, AddonKind};
use mcvm_shared::lang::Language;
use mcvm_shared::util::is_valid_identifier;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use self::conditions::eval_condition;
use self::declarative::eval_declarative_package;

use super::Package;
use crate::data::addon::{self, AddonLocation, AddonRequest};
use crate::data::config::profile::GameModifications;
use crate::io::files::paths::Paths;
use mcvm_parse::instruction::{InstrKind, Instruction};
use mcvm_parse::parse::{Block, BlockId, Parsed};
use mcvm_parse::FailReason;
use mcvm_shared::instance::Side;
use mcvm_shared::pkg::{PackageStability, PkgIdentifier};

use std::collections::HashMap;
use std::path::PathBuf;

/// Max notice instructions per package
static MAX_NOTICE_INSTRUCTIONS: usize = 10;
/// Max characters per notice instruction
static MAX_NOTICE_CHARACTERS: usize = 128;

/// What instructions the evaluator will evaluate (depends on what routine we are running)
#[derive(Debug, Clone)]
pub enum EvalLevel {
	/// When we are installing the addons of a package
	Install,
	/// When we are resolving package relationships
	Resolve,
}

/// Permissions level for an evaluation
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum EvalPermissions {
	Restricted,
	#[default]
	Standard,
	Elevated,
}

/// A routine with a special meaning / purpose.
pub enum Routine {
	Install,
	/// Install routine, except for resolution
	InstallResolve,
}

impl Routine {
	pub fn get_routine_name(&self) -> String {
		String::from(match self {
			Self::Install => INSTALL_ROUTINE,
			Self::InstallResolve => INSTALL_ROUTINE,
		})
	}

	pub fn get_level(&self) -> EvalLevel {
		match self {
			Self::Install => EvalLevel::Install,
			Self::InstallResolve => EvalLevel::Resolve,
		}
	}
}

/// A required package
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct RequiredPackage {
	value: String,
	explicit: bool,
}

/// Constants for the evaluation that will be the same across every package
#[derive(Debug, Clone)]
pub struct EvalConstants {
	pub version: String,
	pub modifications: GameModifications,
	pub features: Vec<String>,
	pub version_list: Vec<String>,
	pub perms: EvalPermissions,
	pub language: Language,
}

/// Constants for the evaluation that may be different for each package
#[derive(Debug, Clone)]
pub struct EvalParameters {
	pub side: Side,
	pub features: Vec<String>,
	pub perms: EvalPermissions,
	pub stability: PackageStability,
}

/// Combination of both EvalConstants and EvalParameters
#[derive(Debug, Clone)]
pub struct EvalInput<'a> {
	pub constants: &'a EvalConstants,
	pub params: EvalParameters,
}

/// Persistent state for evaluation
#[derive(Debug, Clone)]
pub struct EvalData<'a> {
	pub input: EvalInput<'a>,
	pub vars: HashMap<String, String>,
	pub addon_reqs: Vec<AddonRequest>,
	pub id: PkgIdentifier,
	pub level: EvalLevel,
	pub deps: Vec<Vec<RequiredPackage>>,
	pub conflicts: Vec<String>,
	pub recommendations: Vec<String>,
	pub bundled: Vec<String>,
	pub compats: Vec<(String, String)>,
	pub extensions: Vec<String>,
	pub notices: Vec<String>,
}

impl<'a> EvalData<'a> {
	pub fn new(input: EvalInput<'a>, id: PkgIdentifier, routine: &Routine) -> Self {
		Self {
			input,
			vars: HashMap::new(),
			addon_reqs: Vec::new(),
			id,
			level: routine.get_level(),
			deps: Vec::new(),
			conflicts: Vec::new(),
			recommendations: Vec::new(),
			bundled: Vec::new(),
			compats: Vec::new(),
			extensions: Vec::new(),
			notices: Vec::new(),
		}
	}
}

/// Result from an evaluation subfunction
pub struct EvalResult {
	finish: bool,
}

impl EvalResult {
	pub fn new() -> Self {
		Self { finish: false }
	}
}

impl Default for EvalResult {
	fn default() -> Self {
		Self::new()
	}
}

impl Package {
	/// Evaluate a routine on a package
	pub async fn eval<'a>(
		&mut self,
		paths: &Paths,
		routine: Routine,
		input: EvalInput<'a>,
		client: &Client,
	) -> anyhow::Result<EvalData<'a>> {
		self.parse(paths, client).await?;

		// Check properties
		let properties = self.get_properties(paths, client).await?;
		if eval_check_properties(&input, properties)? {
			return Ok(EvalData::new(input, self.id.clone(), &routine));
		}

		match self.content_type {
			PackageContentType::Script => {
				let parsed = self.data.get_mut().contents.get_mut().get_script_contents();
				let eval = eval_script_package(self.id.clone(), parsed, routine, input).await?;
				Ok(eval)
			}
			PackageContentType::Declarative => {
				let contents = self.data.get().contents.get().get_declarative_contents();
				let eval = eval_declarative_package(self.id.clone(), contents, input, routine)?;
				Ok(eval)
			}
		}
	}
}

/// Check properties when evaluating. Returns true if the package should finish evaluating with no error
pub fn eval_check_properties<'a>(
	input: &EvalInput<'a>,
	properties: &PackageProperties,
) -> anyhow::Result<bool> {
	if let Some(supported_modloaders) = &properties.supported_modloaders {
		if !supported_modloaders.iter().any(|x| {
			x.matches(
				&input
					.constants
					.modifications
					.get_modloader(input.params.side),
			)
		}) {
			bail!("Package does not support this modloader");
		}
	}
	if let Some(supported_plugin_loaders) = &properties.supported_plugin_loaders {
		if !supported_plugin_loaders
			.iter()
			.any(|x| x.matches(&&input.constants.modifications.server_type))
		{
			bail!("Package does not support this plugin loader");
		}
	}

	if let Some(supported_sides) = &properties.supported_sides {
		if !supported_sides.contains(&input.params.side) {
			return Ok(true);
		}
	}

	Ok(false)
}

/// Evaluate a script package
pub async fn eval_script_package<'a>(
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

	for instr in &block.contents {
		let result = eval_instr(instr, &mut eval, &parsed.blocks)?;
		if result.finish {
			break;
		}
	}

	Ok(eval)
}

/// Evaluate a block of instructions
fn eval_block(
	block: &Block,
	eval: &mut EvalData,
	blocks: &HashMap<BlockId, Block>,
) -> anyhow::Result<EvalResult> {
	let mut out = EvalResult::new();

	for instr in &block.contents {
		let result = eval_instr(instr, eval, blocks)?;
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
	blocks: &HashMap<BlockId, Block>,
) -> anyhow::Result<EvalResult> {
	let mut out = EvalResult::new();
	match eval.level {
		EvalLevel::Install | EvalLevel::Resolve => match &instr.kind {
			InstrKind::If(condition, block) => {
				if eval_condition(&condition.kind, eval)? {
					out = eval_block(blocks.get(block).expect("If block missing"), eval, blocks)?;
				}
			}
			InstrKind::Set(var, val) => {
				let var = var.get();
				eval.vars.insert(var.to_owned(), val.get(&eval.vars)?);
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
			InstrKind::Recommend(package) => {
				if let EvalLevel::Resolve = eval.level {
					eval.recommendations.push(package.get(&eval.vars)?);
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
			InstrKind::Addon {
				id,
				file_name,
				kind,
				url,
				path,
				version,
			} => {
				if let EvalLevel::Install = eval.level {
					let id = id.get(&eval.vars)?;
					if eval.addon_reqs.iter().any(|x| x.addon.id == id) {
						bail!("Duplicate addon id '{id}'");
					}

					let kind = kind.as_ref().expect("Addon kind missing");
					let addon_req = create_valid_addon_request(
						id,
						url.get_as_option(&eval.vars)?,
						path.get_as_option(&eval.vars)?,
						*kind,
						file_name.get_as_option(&eval.vars)?,
						eval.id.clone(),
						version.get_as_option(&eval.vars)?,
						&eval.input.params.perms,
					)?;
					eval.addon_reqs.push(addon_req);
				}
			}
			_ => bail!("Instruction is not allowed in this routine context"),
		},
	}

	Ok(out)
}

/// Utility for evaluation that validates addon arguments and creates a request
pub fn create_valid_addon_request(
	id: String,
	url: Option<String>,
	path: Option<String>,
	kind: AddonKind,
	file_name: Option<String>,
	pkg_id: PkgIdentifier,
	version: Option<String>,
	perms: &EvalPermissions,
) -> anyhow::Result<AddonRequest> {
	if !is_valid_identifier(&id) {
		bail!("Invalid addon identifier '{id}'");
	}

	// Empty strings will break the filename so we convert them to none
	let version = version.filter(|x| !x.is_empty());
	if let Some(version) = &version {
		if !is_addon_version_valid(version) {
			bail!("Invalid addon version identifier '{version}' for addon '{id}'");
		}
	}

	let file_name =
		file_name.unwrap_or(addon::get_addon_instance_filename(&pkg_id.name, &id, &kind));

	if !is_filename_valid(kind, &file_name) {
		bail!("Invalid addon filename '{file_name}' in addon '{id}'");
	}

	let addon = Addon::new(kind, &id, &file_name, pkg_id, version);

	if let Some(url) = url {
		let location = AddonLocation::Remote(url);
		Ok(AddonRequest::new(addon, location))
	} else if let Some(path) = path {
		match perms {
			EvalPermissions::Elevated => {
				let path = String::from(shellexpand::tilde(&path));
				let path = PathBuf::from(path);
				let location = AddonLocation::Local(path);
				Ok(AddonRequest::new(addon, location))
			}
			_ => {
				bail!("Insufficient permissions to add a local addon '{id}'");
			}
		}
	} else {
		bail!("No location (url/path) was specified for addon '{id}'");
	}
}
