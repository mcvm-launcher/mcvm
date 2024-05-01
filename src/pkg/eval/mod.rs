/// Evaluating script package conditions
pub mod conditions;
/// Evaluating declarative packages
pub mod declarative;
/// Evaluating script packages
pub mod script;

use anyhow::bail;
use anyhow::Context;
use async_trait::async_trait;
use mcvm_parse::routine::INSTALL_ROUTINE;
use mcvm_parse::vars::HashMapVariableStore;
use mcvm_pkg::properties::PackageProperties;
use mcvm_pkg::resolve::ResolutionResult;
use mcvm_pkg::script_eval::AddonInstructionData;
use mcvm_pkg::script_eval::EvalReason;
use mcvm_pkg::ConfiguredPackage;
use mcvm_pkg::PackageContentType;
use mcvm_pkg::RecommendedPackage;
use mcvm_pkg::RequiredPackage;
use mcvm_pkg::{
	PackageEvalRelationsResult as EvalRelationsResultTrait,
	PackageEvaluator as PackageEvaluatorTrait,
};
use mcvm_shared::addon::{is_addon_version_valid, is_filename_valid, Addon};
use mcvm_shared::lang::Language;
use mcvm_shared::output;
use mcvm_shared::output::MCVMOutput;
use mcvm_shared::output::MessageContents;
use mcvm_shared::output::MessageLevel;
use mcvm_shared::pkg::ArcPkgReq;
use mcvm_shared::pkg::PackageID;
use mcvm_shared::util::is_valid_identifier;
use reqwest::Client;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use self::conditions::check_arch_condition;
use self::conditions::check_os_condition;
use self::declarative::eval_declarative_package;
use self::script::eval_script_package;

use super::reg::PkgRegistry;
use super::Package;
use crate::data::addon::{self, AddonLocation, AddonRequest};
use crate::data::config::package::PackageConfig;
use crate::data::config::package::PackageConfigSource;
use crate::data::config::plugin::PluginManager;
use crate::data::config::profile::GameModifications;
use crate::io::files::paths::Paths;
use crate::util::hash::{
	get_hash_str_as_hex, HASH_SHA256_RESULT_LENGTH, HASH_SHA512_RESULT_LENGTH,
};
use mcvm_shared::pkg::PackageStability;
use mcvm_shared::Side;

use std::path::PathBuf;

/// Max notice instructions per package
const MAX_NOTICE_INSTRUCTIONS: usize = 10;
/// Max characters per notice instruction
const MAX_NOTICE_CHARACTERS: usize = 128;

/// Permissions level for an evaluation
#[derive(Deserialize, Serialize, Debug, Copy, Clone, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum EvalPermissions {
	/// Restricts certain operations that would normally be allowed
	Restricted,
	/// Standard permissions. Allow all common operations
	#[default]
	Standard,
	/// Allow execution of things that could compromise security
	Elevated,
}

/// Context / purpose for when we are evaluating
pub enum Routine {
	/// Install the package
	Install,
	/// Install routine, except for resolution
	InstallResolve,
}

impl Routine {
	/// Get the routine name of this routine
	pub fn get_routine_name(&self) -> String {
		match self {
			Self::Install => INSTALL_ROUTINE,
			Self::InstallResolve => INSTALL_ROUTINE,
		}
		.into()
	}

	/// Get the EvalReason of this routine
	pub fn get_reason(&self) -> EvalReason {
		match self {
			Self::Install => EvalReason::Install,
			Self::InstallResolve => EvalReason::Resolve,
		}
	}
}

/// Combination of both EvalConstants and EvalParameters
#[derive(Debug, Clone)]
pub struct EvalInput<'a> {
	/// Constant values
	pub constants: &'a EvalConstants,
	/// Changing values
	pub params: EvalParameters,
}

/// Constants for the evaluation that will be the same across every package
#[derive(Debug, Clone)]
pub struct EvalConstants {
	/// The Minecraft version
	pub version: String,
	/// The modifications to the game
	pub modifications: GameModifications,
	/// The list of available Minecraft versions
	pub version_list: Vec<String>,
	/// The user's configured language
	pub language: Language,
	/// The configured default stability for the profile
	pub profile_stability: PackageStability,
}

/// Constants for the evaluation that may be different for each package
#[derive(Debug, Clone)]
pub struct EvalParameters {
	/// The side (client/server) we are installing the package on
	pub side: Side,
	/// The configuration source of the package
	pub config_source: PackageConfigSource,
	/// Features enabled for the package
	pub features: Vec<String>,
	/// Permissions for the package
	pub perms: EvalPermissions,
	/// Requested stability of the package's contents
	pub stability: PackageStability,
	/// Requested worlds to put addons in
	pub worlds: Vec<String>,
}

impl EvalParameters {
	/// Create new EvalParameters with default parameters and a side
	pub fn new(side: Side) -> Self {
		Self {
			side,
			config_source: PackageConfigSource::Instance,
			features: Vec::new(),
			perms: EvalPermissions::default(),
			stability: PackageStability::default(),
			worlds: Vec::new(),
		}
	}
}

/// Persistent state for evaluation
#[derive(Debug, Clone)]
pub struct EvalData<'a> {
	/// Input to the evaluator
	pub input: EvalInput<'a>,
	/// Plugins
	pub plugins: PluginManager,
	/// ID of the package we are evaluating
	pub id: PackageID,
	/// Level of evaluation
	pub reason: EvalReason,
	/// Package properties
	pub properties: PackageProperties,
	/// Variables, used for script evaluation
	pub vars: HashMapVariableStore,
	/// The output of addon requests
	pub addon_reqs: Vec<AddonRequest>,
	/// The output dependencies
	pub deps: Vec<Vec<RequiredPackage>>,
	/// The output conflicts
	pub conflicts: Vec<PackageID>,
	/// The output recommendations
	pub recommendations: Vec<RecommendedPackage>,
	/// The output bundled packages
	pub bundled: Vec<PackageID>,
	/// The output compats
	pub compats: Vec<(PackageID, PackageID)>,
	/// The output package extensions
	pub extensions: Vec<PackageID>,
	/// The output notices
	pub notices: Vec<String>,
	/// The output commands
	pub commands: Vec<Vec<String>>,
	/// Whether the package uses custom instructions
	pub uses_custom_instructions: bool,
}

impl<'a> EvalData<'a> {
	/// Create a new EvalData
	pub fn new(
		input: EvalInput<'a>,
		id: PackageID,
		properties: PackageProperties,
		routine: &Routine,
		plugins: &PluginManager,
	) -> Self {
		Self {
			input,
			id,
			plugins: plugins.clone(),
			reason: routine.get_reason(),
			properties,
			vars: HashMapVariableStore::default(),
			addon_reqs: Vec::new(),
			deps: Vec::new(),
			conflicts: Vec::new(),
			recommendations: Vec::new(),
			bundled: Vec::new(),
			compats: Vec::new(),
			extensions: Vec::new(),
			notices: Vec::new(),
			commands: Vec::new(),
			uses_custom_instructions: false,
		}
	}
}

impl Package {
	/// Evaluate a routine on a package
	pub async fn eval<'a>(
		&mut self,
		paths: &'a Paths,
		routine: Routine,
		input: EvalInput<'a>,
		client: &Client,
		plugins: &'a PluginManager,
	) -> anyhow::Result<EvalData<'a>> {
		self.parse(paths, client).await?;

		// Check properties
		let properties = self.get_properties(paths, client).await?.clone();
		if eval_check_properties(&input, &properties)? {
			return Ok(EvalData::new(
				input,
				self.id.clone(),
				properties,
				&routine,
				plugins,
			));
		}

		match self.content_type {
			PackageContentType::Script => {
				let parsed = self.data.get_mut().contents.get_mut().get_script_contents();
				let eval = eval_script_package(
					self.id.clone(),
					parsed,
					routine,
					properties,
					input,
					plugins,
					paths,
				)?;
				Ok(eval)
			}
			PackageContentType::Declarative => {
				let contents = self.data.get().contents.get().get_declarative_contents();
				let eval = eval_declarative_package(
					self.id.clone(),
					contents,
					input,
					properties,
					routine,
					plugins,
				)?;
				Ok(eval)
			}
		}
	}
}

/// Check properties when evaluating. Returns true if the package should finish evaluating with no error
pub fn eval_check_properties(
	input: &EvalInput,
	properties: &PackageProperties,
) -> anyhow::Result<bool> {
	if let Some(supported_versions) = &properties.supported_versions {
		if !supported_versions
			.iter()
			.any(|x| x.matches_single(&input.constants.version, &input.constants.version_list))
		{
			bail!("Package does not support this Minecraft version");
		}
	}

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
			.any(|x| x.matches(&input.constants.modifications.server_type))
		{
			bail!("Package does not support this plugin loader");
		}
	}

	if let Some(supported_sides) = &properties.supported_sides {
		if !supported_sides.contains(&input.params.side) {
			return Ok(true);
		}
	}

	if let Some(supported_operating_systems) = &properties.supported_operating_systems {
		if !supported_operating_systems.iter().any(check_os_condition) {
			bail!("Package does not support your operating system");
		}
	}

	if let Some(supported_architectures) = &properties.supported_architectures {
		if !supported_architectures.iter().any(check_arch_condition) {
			bail!("Package does not support your system architecture");
		}
	}

	Ok(false)
}

/// Utility for evaluation that validates addon arguments and creates a request
pub fn create_valid_addon_request(
	data: AddonInstructionData,
	pkg_id: PackageID,
	eval_input: &EvalInput,
) -> anyhow::Result<AddonRequest> {
	if !is_valid_identifier(&data.id) {
		bail!("Invalid addon identifier '{}'", data.id);
	}

	// Empty strings will break the filename so we convert them to none
	let version = data.version.filter(|x| !x.is_empty());
	if let Some(version) = &version {
		if !is_addon_version_valid(version) {
			bail!(
				"Invalid addon version identifier '{version}' for addon '{}'",
				data.id
			);
		}
	}

	let file_name = data.file_name.unwrap_or(addon::get_addon_instance_filename(
		&pkg_id, &data.id, &data.kind,
	));

	if !is_filename_valid(data.kind, &file_name) {
		bail!(
			"Invalid addon filename '{file_name}' in addon '{}'",
			data.id
		);
	}

	// Check hashes
	if let Some(hash) = &data.hashes.sha256 {
		let hex = get_hash_str_as_hex(hash).context("Failed to parse hash string")?;
		if hex.len() > HASH_SHA256_RESULT_LENGTH {
			bail!(
				"SHA-256 hash for addon '{}' is longer than {HASH_SHA256_RESULT_LENGTH} characters",
				data.id
			);
		}
	}

	if let Some(hash) = &data.hashes.sha512 {
		let hex = get_hash_str_as_hex(hash).context("Failed to parse hash string")?;
		if hex.len() > HASH_SHA512_RESULT_LENGTH {
			bail!(
				"SHA-512 hash for addon '{}' is longer than {HASH_SHA512_RESULT_LENGTH} characters",
				data.id
			);
		}
	}

	let addon = Addon {
		kind: data.kind,
		id: data.id.clone(),
		file_name,
		pkg_id,
		version,
		hashes: data.hashes,
	};

	if let Some(url) = data.url {
		let location = AddonLocation::Remote(url);
		Ok(AddonRequest::new(addon, location))
	} else if let Some(path) = data.path {
		match eval_input.params.perms {
			EvalPermissions::Elevated => {
				let path = shellexpand::tilde(&path).to_string();
				let path = PathBuf::from(path);
				let location = AddonLocation::Local(path);
				Ok(AddonRequest::new(addon, location))
			}
			_ => {
				bail!(
					"Insufficient permissions to add a local addon '{}'",
					data.id
				);
			}
		}
	} else {
		bail!(
			"No location (url/path) was specified for addon '{}'",
			data.id
		);
	}
}

/// Evaluator used as an input for dependency resolution
struct PackageEvaluator<'a> {
	reg: &'a mut PkgRegistry,
}

/// Common argument for the evaluator
struct EvaluatorCommonInput<'a> {
	paths: &'a Paths,
	client: &'a Client,
	plugins: PluginManager,
}

/// Newtype for PkgInstanceConfig
#[derive(Clone)]
struct EvalPackageConfig(PackageConfig, ArcPkgReq);

impl ConfiguredPackage for EvalPackageConfig {
	type EvalInput<'a> = EvalInput<'a>;

	fn get_package(&self) -> ArcPkgReq {
		self.1.clone()
	}

	fn override_configured_package_input(
		&self,
		properties: &PackageProperties,
		input: &mut Self::EvalInput<'_>,
	) -> anyhow::Result<()> {
		let features = self
			.0
			.calculate_features(properties)
			.context("Failed to calculate features")?;

		input.params.config_source = self.0.source;
		input.params.features = features;
		input.params.perms = self.0.permissions;
		input.params.stability = self.0.stability;

		Ok(())
	}
}

struct EvalRelationsResult {
	pub deps: Vec<Vec<RequiredPackage>>,
	pub conflicts: Vec<PackageID>,
	pub recommendations: Vec<mcvm_pkg::RecommendedPackage>,
	pub bundled: Vec<PackageID>,
	pub compats: Vec<(PackageID, PackageID)>,
	pub extensions: Vec<PackageID>,
}

impl EvalRelationsResultTrait for EvalRelationsResult {
	fn get_bundled(&self) -> Vec<PackageID> {
		self.bundled.clone()
	}

	fn get_compats(&self) -> Vec<(PackageID, PackageID)> {
		self.compats.clone()
	}

	fn get_conflicts(&self) -> Vec<PackageID> {
		self.conflicts.clone()
	}

	fn get_deps(&self) -> Vec<Vec<RequiredPackage>> {
		self.deps.clone()
	}
	fn get_extensions(&self) -> Vec<PackageID> {
		self.extensions.clone()
	}

	fn get_recommendations(&self) -> Vec<mcvm_pkg::RecommendedPackage> {
		self.recommendations.clone()
	}
}

#[async_trait]
impl<'a> PackageEvaluatorTrait<'a> for PackageEvaluator<'a> {
	type CommonInput = EvaluatorCommonInput<'a>;
	type ConfiguredPackage = EvalPackageConfig;
	type EvalInput<'b> = EvalInput<'b>;
	type EvalRelationsResult<'b> = EvalRelationsResult;

	async fn eval_package_relations(
		&mut self,
		pkg: &ArcPkgReq,
		input: &Self::EvalInput<'a>,
		common_input: &Self::CommonInput,
	) -> anyhow::Result<Self::EvalRelationsResult<'a>> {
		let eval = self
			.reg
			.eval(
				pkg,
				common_input.paths,
				Routine::InstallResolve,
				input.clone(),
				common_input.client,
				&common_input.plugins,
				&mut output::NoOp,
			)
			.await
			.context("Failed to evaluate dependencies for package")?;

		let result = EvalRelationsResult {
			deps: eval.deps,
			conflicts: eval.conflicts,
			recommendations: eval.recommendations,
			bundled: eval.bundled,
			compats: eval.compats,
			extensions: eval.extensions,
		};

		Ok(result)
	}

	async fn get_package_properties<'b>(
		&'b mut self,
		pkg: &ArcPkgReq,
		common_input: &Self::CommonInput,
	) -> anyhow::Result<&'b PackageProperties> {
		let properties = self
			.reg
			.get_properties(
				pkg,
				common_input.paths,
				common_input.client,
				&mut output::NoOp,
			)
			.await?;
		Ok(properties)
	}
}

/// Resolve package dependencies
pub async fn resolve(
	packages: &[PackageConfig],
	constants: &EvalConstants,
	default_params: EvalParameters,
	paths: &Paths,
	reg: &mut PkgRegistry,
	client: &Client,
	plugins: &PluginManager,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<ResolutionResult> {
	let evaluator = PackageEvaluator { reg };

	let input = EvalInput {
		constants,
		params: default_params,
	};

	let common_input = EvaluatorCommonInput {
		client,
		paths,
		plugins: plugins.clone(),
	};

	let packages = packages
		.iter()
		.map(|x| EvalPackageConfig((*x).clone(), x.get_request()))
		.collect::<Vec<_>>();

	let result = mcvm_pkg::resolve::resolve(&packages, evaluator, input, &common_input).await?;

	for package in &result.unfulfilled_recommendations {
		print_recommendation_warning(package, o);
	}

	Ok(result)
}

/// Prints an unfulfilled recommendation warning
fn print_recommendation_warning(
	package: &mcvm_pkg::resolve::RecommendedPackage,
	o: &mut impl MCVMOutput,
) {
	let source = package.req.source.get_source();
	let message = if package.invert {
		if let Some(source) = source {
			MessageContents::Warning(format!("The package '{}' recommends against the use of the package '{}', which is installed", source.debug_sources(), package.req))
		} else {
			MessageContents::Warning(format!(
				"A package recommends against the use of the package '{}', which is installed",
				package.req
			))
		}
	} else if let Some(source) = source {
		MessageContents::Warning(format!(
			"The package '{}' recommends the use of the package '{}', which is not installed",
			source.debug_sources(),
			package.req
		))
	} else {
		MessageContents::Warning(format!(
			"A package recommends the use of the package '{}', which is not installed",
			package.req
		))
	};

	o.display(message, MessageLevel::Important);
}
