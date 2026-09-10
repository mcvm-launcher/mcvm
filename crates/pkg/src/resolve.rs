use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use itertools::Itertools;
use nitro_shared::pkg::{
	ArcPkgReq, PackageID, PackageOverrides, ResolutionError, is_package_overridden,
};
use nitro_shared::versions::VersionPattern;

use crate::properties::PackageProperties;
use crate::{ConfiguredPackage, EvalInput, PackageEvaluator};

use crate::{PkgRequest, PkgRequestSource};

/// Find all package dependencies from a set of required packages
pub async fn resolve<'a, E: PackageEvaluator<'a>>(
	packages: &[E::ConfiguredPackage],
	mut evaluator: E,
	constant_eval_input: E::EvalInput,
	common_input: &E::CommonInput,
	overrides: PackageOverrides,
) -> Result<ResolutionResult, ResolutionError> {
	let mut resolver = Resolver {
		tasks: VecDeque::new(),
		dependencies: HashMap::new(),
		refusals: HashSet::new(),
		recommendations: HashSet::new(),
		compats: HashSet::new(),
		extensions: HashSet::new(),
		suppressions: HashSet::new(),
		constant_input: constant_eval_input,
		package_configs: HashMap::new(),
		overrides,
	};

	let suppressed: Vec<_> = resolver.overrides.suppress.to_vec();
	for package in suppressed {
		resolver.suppress_package(Arc::new(PkgRequest::parse(
			package,
			PkgRequestSource::UserRequire,
		)));
	}

	// Used to keep track of which packages have been preloaded and are good to further evaluate as tasks
	let mut preloaded_packages = HashSet::with_capacity(packages.len());

	// Preload all of the user's configured packages
	let collected_packages: Vec<_> = packages
		.iter()
		.filter_map(|x| {
			let req = x.get_package();
			if is_package_overridden(&req, &resolver.overrides.suppress) {
				None
			} else {
				Some(req)
			}
		})
		.collect();
	if let Err(e) = evaluator
		.preload_packages(&collected_packages, common_input)
		.await
	{
		return Err(ResolutionError::FailedToPreload(e));
	}
	preloaded_packages.extend(collected_packages);

	// Create the initial EvalPackage tasks and constraints from the installed packages
	for config in packages.iter().sorted_by_key(|x| x.get_package()) {
		let req = config.get_package();
		resolver.update_dependency(&req, DependencyKind::UserRequire);
		resolver.package_configs.insert(req.clone(), config.clone());
	}

	// Resolve all of the tasks
	// The strategy for preloading is to complete tasks until none of them have preloaded packages, then preload all of them and repeat
	'outer: loop {
		let mut num_skipped = 0;

		loop {
			// We have skipped all the tasks and need to finally preload them
			if num_skipped >= resolver.tasks.len() && !resolver.tasks.is_empty() {
				break;
			}

			if let Some(task) = resolver.tasks.pop_front() {
				// Skip this task if it is not preloaded
				#[allow(irrefutable_let_patterns)]
				if let Task::EvalPackage { dest, .. } = &task
					&& !preloaded_packages.contains(dest)
				{
					num_skipped += 1;
					resolver.tasks.push_back(task);
					continue;
				}

				if let Err(mut e) =
					resolve_task(task, common_input, &mut evaluator, &mut resolver).await
				{
					make_err_reqs_displayable(&mut e, &mut evaluator, common_input).await;
					return Err(e);
				}
				resolver.check_compats();

				// Reset the skip count since it is no longer valid
				num_skipped = 0;
			} else {
				break 'outer;
			}
		}

		// Preload all of the packages
		let to_preload: Vec<_> = resolver
			.tasks
			.iter()
			.filter_map(|x| {
				#[allow(irrefutable_let_patterns)]
				if let Task::EvalPackage { dest, .. } = x {
					if is_package_overridden(dest, &resolver.overrides.suppress) {
						None
					} else {
						Some(dest.clone())
					}
				} else {
					None
				}
			})
			.collect();

		if let Err(e) = evaluator.preload_packages(&to_preload, common_input).await {
			return Err(ResolutionError::FailedToPreload(e));
		};

		preloaded_packages.extend(to_preload);
	}

	let mut unfulfilled_recommendations = Vec::new();

	// Final check for constraints
	for recommendation in &resolver.recommendations {
		if recommendation.invert {
			if resolver.is_required(&recommendation.req) {
				let package = evaluator
					.make_req_displayable(&recommendation.req, common_input)
					.await;
				unfulfilled_recommendations.push(RecommendedPackage {
					req: package.clone(),
					invert: true,
				});
			}
		} else if !resolver.is_required(&recommendation.req) {
			let package = evaluator
				.make_req_displayable(&recommendation.req, common_input)
				.await;
			unfulfilled_recommendations.push(RecommendedPackage {
				req: package.clone(),
				invert: false,
			});
		}
	}

	for package in &resolver.extensions {
		if !resolver.is_required(package) {
			let package = evaluator.make_req_displayable(package, common_input).await;
			let source = package.source.get_source();
			let source = if let Some(source) = source {
				Some(evaluator.make_req_displayable(&source, common_input).await)
			} else {
				None
			};
			return Err(ResolutionError::ExtensionNotFulfilled(
				source,
				package.clone(),
			));
		}
	}

	let out = ResolutionResult {
		packages: resolver.collect_packages(),
		unfulfilled_recommendations,
	};

	Ok(out)
}

/// Result from package resolution
pub struct ResolutionResult {
	/// The list of packages to install
	pub packages: Vec<ResolutionPackageResult>,
	/// Package recommendations that were not satisfied
	pub unfulfilled_recommendations: Vec<RecommendedPackage>,
}

/// A single package resulting from resolution
#[derive(Debug)]
pub struct ResolutionPackageResult {
	/// The request for this package. The content version probably doesn't mean anything.
	pub req: ArcPkgReq,
}

/// Recommended package that has a PkgRequest instead of a String
#[derive(PartialEq, Eq, Hash)]
pub struct RecommendedPackage {
	/// Package to recommend
	pub req: ArcPkgReq,
	/// Whether to invert this recommendation to recommend against a package
	pub invert: bool,
}

/// Resolve a single task
async fn resolve_task<'a, E: PackageEvaluator<'a>>(
	task: Task,
	common_input: &E::CommonInput,
	evaluator: &mut E,
	resolver: &mut Resolver<'a, E>,
) -> Result<(), ResolutionError> {
	match task {
		Task::EvalPackage { dest } => {
			if is_package_overridden(&dest, &resolver.overrides.suppress) {
				return Ok(());
			}

			let result = resolve_eval_package(dest.clone(), common_input, evaluator, resolver)
				.await
				.map_err(|e| ResolutionError::PackageContext(dest.clone(), Box::new(e)));

			let config = resolver.package_configs.get(&dest);

			// Skip errors for optional packages or dependencies of optional packages
			if let Err(e) = result {
				let Some(config) = config else {
					return Err(e);
				};

				if let Some(original_source) = dest.source.get_original_source()
					&& let Some(config) = resolver.package_configs.get(original_source)
					&& config.is_optional()
				{
					return Ok(());
				}

				if !config.is_optional() {
					return Err(e);
				}
			}
		}
	}

	Ok(())
}

/// Resolve an EvalPackage task
async fn resolve_eval_package<'a, E: PackageEvaluator<'a>>(
	package: ArcPkgReq,
	common_input: &E::CommonInput,
	evaluator: &mut E,
	resolver: &mut Resolver<'a, E>,
) -> Result<(), ResolutionError> {
	// Make sure that this package fits the constraints as well
	resolver.check_constraints(&package)?;

	// Resolve versions
	let default = Vec::new();
	let dependency = resolver
		.dependencies
		.entry(package.clone())
		.or_insert_with(|| Dependency::new(package.clone(), DependencyKind::Require));

	dependency
		.canonicalize_versions(evaluator, common_input)
		.await;

	let properties = evaluator
		.get_package_properties(&package, common_input)
		.await
		.map_err(|e| ResolutionError::FailedToGetProperties(package.clone(), e))?;
	let available_versions = properties.content_versions.as_ref().unwrap_or(&default);

	let required_content_versions = dependency.get_versions(available_versions);
	let preferred_content_versions = dependency.get_preferred_versions();
	// We have overconstrained and no versions are left
	if required_content_versions.is_empty() && !available_versions.is_empty() {
		return Err(ResolutionError::NoValidVersionsFound(
			package,
			dependency.canonicalized_version_constraints.clone(),
		));
	}

	// Get the correct EvalInput
	let config = resolver.package_configs.get(&package);
	let input = override_eval_input::<E>(
		&properties,
		&resolver.constant_input,
		required_content_versions,
		preferred_content_versions,
		is_package_overridden(&package, &resolver.overrides.force),
		config,
	)?;

	let result = evaluator
		.eval_package_relations(&package, &input, common_input)
		.await
		.map_err(|e| ResolutionError::FailedToEvaluate(package.clone(), e))?;

	for conflict in result.conflicts.iter().sorted() {
		let req = Arc::new(PkgRequest::parse(
			conflict,
			PkgRequestSource::Refused(package.clone()),
		));
		if resolver.is_required(&req) {
			return Err(ResolutionError::IncompatiblePackage(
				req,
				vec![package.to_string().into()],
			));
		}
		resolver.refusals.insert(req);
	}

	for dep in result.deps.iter().flatten().sorted() {
		let req = Arc::new(PkgRequest::parse(
			&dep.value,
			PkgRequestSource::Dependency(package.clone()),
		));
		if dep.explicit && !resolver.is_user_required(&req) {
			return Err(ResolutionError::ExplicitRequireNotFulfilled(
				req,
				package.clone(),
			));
		}
		resolver.check_constraints(&req)?;
		resolver.update_dependency(&req, DependencyKind::Require);
	}

	for bundled in result.bundled.iter().sorted() {
		let req = Arc::new(PkgRequest::parse(
			bundled,
			PkgRequestSource::Bundled(package.clone()),
		));
		resolver.check_constraints(&req)?;

		resolver.update_dependency(&req, DependencyKind::Bundled);
	}

	for (check_package, compat_package) in result.compats.iter().sorted() {
		let check_package = Arc::new(PkgRequest::parse(
			check_package,
			PkgRequestSource::Dependency(package.clone()),
		));
		let compat_package = Arc::new(PkgRequest::parse(
			compat_package,
			PkgRequestSource::Dependency(package.clone()),
		));
		resolver.compats.insert((check_package, compat_package));
	}

	for extension in result.extensions.iter().sorted() {
		let req = Arc::new(PkgRequest::parse(
			extension,
			PkgRequestSource::Dependency(package.clone()),
		));
		resolver.extensions.insert(req);
	}

	for recommendation in result.recommendations.iter().sorted() {
		let req = Arc::new(PkgRequest::parse(
			&recommendation.value,
			PkgRequestSource::Dependency(package.clone()),
		));
		resolver.recommendations.insert(RecommendedPackage {
			req,
			invert: recommendation.invert,
		});
	}

	for inclusion in result.inclusions.iter().sorted() {
		let req = Arc::new(PkgRequest::parse(
			inclusion,
			PkgRequestSource::Dependency(package.clone()),
		));
		resolver.suppress_package(req);
	}

	Ok(())
}

/// Overrides the EvalInput for a package with config
fn override_eval_input<'a, E: PackageEvaluator<'a>>(
	properties: &PackageProperties,
	constant_eval_input: &E::EvalInput,
	required_content_versions: Vec<String>,
	preferred_content_versions: Vec<String>,
	force: bool,
	config: Option<&E::ConfiguredPackage>,
) -> Result<E::EvalInput, ResolutionError> {
	let input = {
		let mut constant_eval_input = constant_eval_input.clone();
		constant_eval_input
			.set_content_versions(required_content_versions, preferred_content_versions);
		constant_eval_input.set_force(force);

		if let Some(config) = config
			&& let Err(e) =
				config.override_configured_package_input(properties, &mut constant_eval_input)
		{
			return Err(ResolutionError::Misc(e));
		}
		constant_eval_input
	};

	Ok(input)
}

/// State for resolution
struct Resolver<'a, E: PackageEvaluator<'a>> {
	tasks: VecDeque<Task>,
	dependencies: HashMap<ArcPkgReq, Dependency>,
	refusals: HashSet<ArcPkgReq>,
	recommendations: HashSet<RecommendedPackage>,
	compats: HashSet<(ArcPkgReq, ArcPkgReq)>,
	extensions: HashSet<ArcPkgReq>,
	suppressions: HashSet<ArcPkgReq>,
	constant_input: E::EvalInput,
	package_configs: HashMap<ArcPkgReq, E::ConfiguredPackage>,
	overrides: PackageOverrides,
}

impl<'a, E> Resolver<'a, E>
where
	E: PackageEvaluator<'a>,
{
	/// Whether a package has been required by an existing constraint
	pub fn is_required(&self, req: &ArcPkgReq) -> bool {
		self.dependencies.contains_key(req)
	}

	/// Whether a package has been required by the user
	pub fn is_user_required(&self, req: &ArcPkgReq) -> bool {
		self.dependencies
			.values()
			.any(|x| &x.pkg == req || x.pkg.source.is_user_bundled())
	}

	/// Updates a dependency
	pub fn update_dependency(&mut self, req: &ArcPkgReq, kind: DependencyKind) {
		if self.suppressions.contains(req) {
			return;
		}

		let (dependency, just_inserted) = if let Some(dependency) = self.dependencies.get_mut(req) {
			(dependency, false)
		} else {
			self.dependencies
				.insert(req.clone(), Dependency::new(req.clone(), kind));

			(self.dependencies.get_mut(req).unwrap(), true)
		};

		dependency.update_importance(kind);

		let version_changed = dependency.add_version_constraint(req.content_version.clone());

		// Update the package if it changed
		if just_inserted || version_changed {
			self.tasks
				.push_back(Task::EvalPackage { dest: req.clone() });
		}
	}

	/// Suppresses a package
	pub fn suppress_package(&mut self, req: ArcPkgReq) {
		self.dependencies.remove(&req);
		self.suppressions.insert(req);
	}

	/// Whether a package has been refused by an existing constraint
	pub fn is_refused(&self, req: &ArcPkgReq) -> bool {
		self.refusals.contains(req)
	}

	/// Get all refusers of this package
	pub fn get_refusers(&self, req: &ArcPkgReq) -> Vec<PackageID> {
		self.refusals
			.iter()
			.filter_map(|req2| {
				if req2 == req {
					Some(
						req2.source
							.get_source()
							.map(|source| source.id.clone())
							.unwrap_or("User-refused".into()),
					)
				} else {
					None
				}
			})
			.collect()
	}

	/// Creates an error if this package is disallowed in the constraints
	pub fn check_constraints(&self, req: &ArcPkgReq) -> Result<(), ResolutionError> {
		if self.is_refused(req) {
			let refusers = self.get_refusers(req);
			return Err(ResolutionError::IncompatiblePackage(req.clone(), refusers));
		}

		Ok(())
	}

	/// Checks compat constraints to see if new constraints are needed
	pub fn check_compats(&mut self) {
		let mut packages_to_require = Vec::new();
		for (check_package, compat_package) in &self.compats {
			if self.is_required(check_package) && !self.is_required(compat_package) {
				packages_to_require.push(compat_package.clone());
			}
		}
		for package in packages_to_require {
			self.update_dependency(&package, DependencyKind::Require);
		}
	}

	/// Collect all needed packages for final output
	pub fn collect_packages(self) -> Vec<ResolutionPackageResult> {
		self.dependencies
			.into_values()
			.map(|x| ResolutionPackageResult { req: x.pkg.clone() })
			.collect()
	}
}

#[derive(Clone)]
struct Dependency {
	pkg: ArcPkgReq,
	kind: DependencyKind,
	/// Version pattern constraints imposed on the available versions, uncanonicalized
	uncanonicalized_version_constraints: Vec<VersionPattern>,
	/// Version pattern constraints that have been canonicalized to the actual names
	canonicalized_version_constraints: Vec<VersionPattern>,
	/// Copies of uncanonicalized constraints that have been added to the canonical list but we keep for later comparisons
	already_canonicalized_version_constraints: Vec<VersionPattern>,
}

impl Dependency {
	/// Creates a new dependency
	pub fn new(pkg: ArcPkgReq, kind: DependencyKind) -> Self {
		Self {
			pkg,
			kind,
			uncanonicalized_version_constraints: Vec::new(),
			canonicalized_version_constraints: Vec::new(),
			already_canonicalized_version_constraints: Vec::new(),
		}
	}

	/// Updates the kind of this dependency if the given kind is of higher importance.
	pub fn update_importance(&mut self, kind: DependencyKind) {
		if kind > self.kind {
			self.kind = kind;
		}
	}

	/// Adds a new version constraint to this dependency. Returns true if the constraints have changed
	pub fn add_version_constraint(&mut self, constraint: VersionPattern) -> bool {
		if constraint != VersionPattern::Any
			&& !self
				.uncanonicalized_version_constraints
				.contains(&constraint)
			&& !self.canonicalized_version_constraints.contains(&constraint)
			&& !self
				.already_canonicalized_version_constraints
				.contains(&constraint)
		{
			self.uncanonicalized_version_constraints.push(constraint);
			true
		} else {
			false
		}
	}

	/// Canonicalizes content versions constraints to their actual names from the package
	pub async fn canonicalize_versions<'a, E: PackageEvaluator<'a>>(
		&mut self,
		evaluator: &mut E,
		common_input: &E::CommonInput,
	) {
		for constraint in std::mem::take(&mut self.uncanonicalized_version_constraints) {
			let req = Arc::new(self.pkg.with_content_version(constraint.clone()));

			let req = evaluator.make_req_displayable(&req, common_input).await;
			self.canonicalized_version_constraints
				.push(req.content_version.clone());
			self.already_canonicalized_version_constraints
				.push(constraint);
		}
	}

	/// Gets the list of versions based on the canonicalized available versions and requirements
	pub fn get_versions(&self, available_versions: &[String]) -> Vec<String> {
		let mut out = available_versions.to_vec();
		for constraint in &self.canonicalized_version_constraints {
			out = constraint.get_matches(&out);
		}

		out
	}

	/// Gets the list of preferred versions from the constraints
	pub fn get_preferred_versions(&self) -> Vec<String> {
		self.canonicalized_version_constraints
			.iter()
			.filter_map(|x| {
				if let VersionPattern::Prefer(version) = x {
					Some(version.clone())
				} else {
					None
				}
			})
			.collect()
	}
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DependencyKind {
	Require,
	Bundled,
	UserRequire,
}

/// A task that needs to be completed for resolution
enum Task {
	/// Evaluate a package and its relationships
	EvalPackage { dest: Arc<PkgRequest> },
}

async fn make_err_reqs_displayable<'eval, E: PackageEvaluator<'eval>>(
	err: &mut ResolutionError,
	evaluator: &mut E,
	common_input: &E::CommonInput,
) {
	match err {
		ResolutionError::PackageContext(pkg_request, resolution_error) => {
			*pkg_request = evaluator
				.make_req_displayable(pkg_request, common_input)
				.await;
			Box::pin(make_err_reqs_displayable(
				resolution_error,
				evaluator,
				common_input,
			))
			.await;
		}
		ResolutionError::FailedToPreload(..) | ResolutionError::Misc(..) => {}
		ResolutionError::FailedToGetProperties(pkg_request, _) => {
			*pkg_request = evaluator
				.make_req_displayable(pkg_request, common_input)
				.await;
		}
		ResolutionError::NoValidVersionsFound(pkg_request, _) => {
			*pkg_request = evaluator
				.make_req_displayable(pkg_request, common_input)
				.await;
		}
		ResolutionError::ExtensionNotFulfilled(pkg_request, pkg_request1) => {
			if let Some(pkg_request) = pkg_request {
				*pkg_request = evaluator
					.make_req_displayable(pkg_request, common_input)
					.await;
			}
			*pkg_request1 = evaluator
				.make_req_displayable(pkg_request1, common_input)
				.await;
		}
		ResolutionError::ExplicitRequireNotFulfilled(pkg_request, pkg_request1) => {
			*pkg_request = evaluator
				.make_req_displayable(pkg_request, common_input)
				.await;
			*pkg_request1 = evaluator
				.make_req_displayable(pkg_request1, common_input)
				.await;
		}
		ResolutionError::IncompatiblePackage(pkg_request, _) => {
			*pkg_request = evaluator
				.make_req_displayable(pkg_request, common_input)
				.await;
		}
		ResolutionError::FailedToEvaluate(pkg_request, _) => {
			*pkg_request = evaluator
				.make_req_displayable(pkg_request, common_input)
				.await;
		}
	}
}
