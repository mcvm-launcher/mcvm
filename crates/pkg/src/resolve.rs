use std::collections::VecDeque;
use std::sync::Arc;

use anyhow::{bail, Context};
use itertools::Itertools;
use mcvm_shared::pkg::{ArcPkgReq, PackageID};

use crate::properties::PackageProperties;
use crate::{ConfiguredPackage, PackageEvalRelationsResult, PackageEvaluator};

use crate::{PkgRequest, PkgRequestSource};

/// Find all package dependencies from a set of required packages
pub async fn resolve<'a, E: PackageEvaluator<'a>>(
	packages: &[E::ConfiguredPackage],
	mut evaluator: E,
	constant_eval_input: E::EvalInput<'a>,
	common_input: &E::CommonInput,
) -> anyhow::Result<ResolutionResult> {
	let mut resolver = Resolver {
		tasks: VecDeque::new(),
		constraints: Vec::new(),
		constant_input: constant_eval_input,
	};

	// Create the initial EvalPackage tasks and constraints from the installed packages
	for config in packages.iter().sorted_by_key(|x| x.get_package()) {
		let req = config.get_package();
		let props = evaluator
			.get_package_properties(&req, common_input)
			.await
			.context("Failed to get package properties")?;

		resolver.constraints.push(Constraint {
			kind: ConstraintKind::UserRequire(
				req.clone(),
				props.content_versions.clone().unwrap_or_default(),
			),
		});
		resolver.tasks.push_back(Task::EvalPackage {
			dest: req.clone(),
			config: Some(config.clone()),
		});
	}

	while let Some(task) = resolver.tasks.pop_front() {
		resolve_task(task, common_input, &mut evaluator, &mut resolver).await?;
		resolver
			.check_compats(&mut evaluator, common_input)
			.await
			.context("Failed to check compats")?;
	}

	let mut unfulfilled_recommendations = Vec::new();

	for constraint in resolver.constraints.iter() {
		match &constraint.kind {
			ConstraintKind::Recommend(package, invert) => {
				if *invert {
					if resolver.is_required(package) {
						unfulfilled_recommendations.push(RecommendedPackage {
							req: package.clone(),
							invert: true,
						});
					}
				} else if !resolver.is_required(package) {
					unfulfilled_recommendations.push(RecommendedPackage {
						req: package.clone(),
						invert: false,
					});
				}
			}
			ConstraintKind::Extend(package) => {
				if !resolver.is_required(package) {
					let source = package.source.get_source();
					if let Some(source) = source {
						bail!(
							"The package '{}' extends the functionality of the package '{}', which is not installed.",
							source.debug_sources(),
							package
						);
					} else {
						bail!(
							"A package extends the functionality of the package '{}', which is not installed.",
							package
						);
					}
				}
			}
			_ => {}
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
	pub packages: Vec<ArcPkgReq>,
	/// Package recommendations that were not satisfied
	pub unfulfilled_recommendations: Vec<RecommendedPackage>,
}

/// Recommended package that has a PkgRequest instead of a String
pub struct RecommendedPackage {
	/// Package to recommend
	pub req: ArcPkgReq,
	/// Whether to invert this recommendation to recommend against a package
	pub invert: bool,
}

/// Resolve a single task
async fn resolve_task<'a, E: PackageEvaluator<'a>>(
	task: Task<'a, E>,
	common_input: &E::CommonInput,
	evaluator: &mut E,
	resolver: &mut Resolver<'a, E>,
) -> anyhow::Result<()> {
	match task {
		Task::EvalPackage { dest, config } => {
			resolve_eval_package(
				dest.clone(),
				config.as_ref(),
				common_input,
				evaluator,
				resolver,
			)
			.await
			.with_context(|| package_context_error_message(&dest))?;
		}
	}

	Ok(())
}

/// Resolve an EvalPackage task
async fn resolve_eval_package<'a, E: PackageEvaluator<'a>>(
	package: ArcPkgReq,
	config: Option<&E::ConfiguredPackage>,
	common_input: &E::CommonInput,
	evaluator: &mut E,
	resolver: &mut Resolver<'a, E>,
) -> anyhow::Result<()> {
	// Make sure that this package fits the constraints as well
	resolver
		.check_constraints(&package)
		.context("Package did not fit existing constraints")?;

	// Get the correct EvalInput
	let properties = evaluator
		.get_package_properties(&package, common_input)
		.await
		.context("Failed to get package properties")?;
	let input = override_eval_input::<E>(properties, &resolver.constant_input, config)?;

	let result = evaluator
		.eval_package_relations(&package, &input, common_input)
		.await
		.context("Failed to evaluate package")?;

	for conflict in result.get_conflicts().iter().sorted() {
		let req = Arc::new(PkgRequest::parse(
			conflict,
			PkgRequestSource::Refused(package.clone()),
		));
		if resolver.is_required(&req) {
			bail!(
				"Package '{}' is incompatible with this package.",
				req.debug_sources()
			);
		}
		resolver.constraints.push(Constraint {
			kind: ConstraintKind::Refuse(req),
		});
	}

	for dep in result.get_deps().iter().flatten().sorted() {
		let req = Arc::new(PkgRequest::parse(
			&dep.value,
			PkgRequestSource::Dependency(package.clone()),
		));
		if dep.explicit && !resolver.is_user_required(&req) {
			bail!("Package '{req}' has been explicitly required by this package. This means it must be required by the user in their config.");
		}
		resolver.check_constraints(&req)?;
		let props = evaluator
			.get_package_properties(&req, common_input)
			.await
			.context("Failed to get properties for required package")?;
		resolver.update_require_constraint(&req, props, RequireConstraint::Require)?;
	}

	for bundled in result.get_bundled().iter().sorted() {
		let req = Arc::new(PkgRequest::parse(
			bundled,
			PkgRequestSource::Bundled(package.clone()),
		));
		resolver.check_constraints(&req)?;
		let props = evaluator
			.get_package_properties(&req, common_input)
			.await
			.context("Failed to get properties for required package")?;
		resolver.update_require_constraint(&req, props, RequireConstraint::Bundle)?;

		// Upgrade a require constraint into a bundle
		let existing_versions = resolver
			.constraints
			.iter()
			.find_map(|x| {
				if let ConstraintKind::Require(req2, versions) = &x.kind {
					if req2 == &req {
						Some(versions)
					} else {
						None
					}
				} else {
					None
				}
			})
			.cloned();
		resolver.remove_require_constraint(&req);
		resolver.constraints.push(Constraint {
			kind: ConstraintKind::Bundle(req.clone(), existing_versions.unwrap_or_default()),
		});
	}

	for (check_package, compat_package) in result.get_compats().iter().sorted() {
		let check_package = Arc::new(PkgRequest::parse(
			check_package,
			PkgRequestSource::Dependency(package.clone()),
		));
		let compat_package = Arc::new(PkgRequest::parse(
			compat_package,
			PkgRequestSource::Dependency(package.clone()),
		));
		if !resolver.compat_exists(check_package.clone(), compat_package.clone()) {
			resolver.constraints.push(Constraint {
				kind: ConstraintKind::Compat(check_package, compat_package),
			});
		}
	}

	for extension in result.get_extensions().iter().sorted() {
		let req = Arc::new(PkgRequest::parse(
			extension,
			PkgRequestSource::Dependency(package.clone()),
		));
		resolver.constraints.push(Constraint {
			kind: ConstraintKind::Extend(req),
		});
	}

	for recommendation in result.get_recommendations().iter().sorted() {
		let req = Arc::new(PkgRequest::parse(
			&recommendation.value,
			PkgRequestSource::Dependency(package.clone()),
		));
		resolver.constraints.push(Constraint {
			kind: ConstraintKind::Recommend(req, recommendation.invert),
		});
	}

	Ok(())
}

/// Overrides the EvalInput for a package with config
fn override_eval_input<'a, E: PackageEvaluator<'a>>(
	properties: &PackageProperties,
	constant_eval_input: &E::EvalInput<'a>,
	config: Option<&E::ConfiguredPackage>,
) -> anyhow::Result<E::EvalInput<'a>> {
	let input = {
		let mut constant_eval_input = constant_eval_input.clone();
		if let Some(config) = config {
			config.override_configured_package_input(properties, &mut constant_eval_input)?;
		}
		constant_eval_input
	};

	Ok(input)
}

/// State for resolution
struct Resolver<'a, E: PackageEvaluator<'a>> {
	tasks: VecDeque<Task<'a, E>>,
	constraints: Vec<Constraint>,
	constant_input: E::EvalInput<'a>,
}

impl<'a, E> Resolver<'a, E>
where
	E: PackageEvaluator<'a>,
{
	fn is_required_fn(constraint: &Constraint, req: &ArcPkgReq) -> bool {
		matches!(
			&constraint.kind,
			ConstraintKind::Require(dest, _)
			| ConstraintKind::UserRequire(dest, _)
			| ConstraintKind::Bundle(dest, _) if dest == req
		)
	}

	/// Whether a package has been required by an existing constraint
	pub fn is_required(&self, req: &ArcPkgReq) -> bool {
		self.constraints
			.iter()
			.any(|x| Self::is_required_fn(x, req))
	}

	/// Whether a package has been required by the user
	pub fn is_user_required(&self, req: &ArcPkgReq) -> bool {
		self.constraints.iter().any(|x| {
			matches!(&x.kind, ConstraintKind::UserRequire(dest, _) if dest == req)
				|| matches!(&x.kind, ConstraintKind::Bundle(dest, _) if dest == req && dest.source.is_user_bundled())
		})
	}

	/// Remove the require constraint of a package if it exists
	pub fn remove_require_constraint(&mut self, req: &ArcPkgReq) {
		let index = self
			.constraints
			.iter()
			.position(|x| matches!(&x.kind, ConstraintKind::Require(req2, _) if req2 == req));
		if let Some(index) = index {
			self.constraints.swap_remove(index);
		}
	}

	/// Updates a require constraint, returning an error if the package is now overconstrained
	pub fn update_require_constraint(
		&mut self,
		req: &ArcPkgReq,
		properties: &PackageProperties,
		kind: RequireConstraint,
	) -> anyhow::Result<()> {
		// Find an existing constraint to update
		let existing_constraint = self.constraints.iter_mut().find_map(|x| {
			if let ConstraintKind::Require(req2, versions)
			| ConstraintKind::UserRequire(req2, versions)
			| ConstraintKind::Bundle(req2, versions) = &mut x.kind
			{
				if req2 == req {
					Some(versions)
				} else {
					None
				}
			} else {
				None
			}
		});

		// Update an existing constraint
		if let Some(existing_versions) = existing_constraint {
			// If the existing versions is already empty, that means this package just doesn't have any content versions
			if existing_versions.is_empty() {
				return Ok(());
			}
			// Futher constrain the list of possible content versions for the package
			let new_versions = req.content_version.get_matches(&existing_versions);
			// We have overconstrained to the point that there are no versions left
			if new_versions.is_empty() {
				bail!("Could not find a version of {req} that matches all of the content version requirements");
			}
			// If the number of versions is now smaller, that means a different version could be selected and we need to re-evaluate
			if new_versions.len() != existing_versions.len() {
				self.tasks.push_back(Task::EvalPackage {
					dest: req.clone(),
					config: None,
				});
			}

			*existing_versions = new_versions;
		} else {
			// Create a new constraint
			let versions = properties.content_versions.clone().unwrap_or_default();
			let kind = match kind {
				RequireConstraint::Require => ConstraintKind::Require(req.clone(), versions),
				RequireConstraint::UserRequire => {
					ConstraintKind::UserRequire(req.clone(), versions)
				}
				RequireConstraint::Bundle => ConstraintKind::Bundle(req.clone(), versions),
			};
			self.constraints.push(Constraint { kind });
			self.tasks.push_back(Task::EvalPackage {
				dest: req.clone(),
				config: None,
			});
		}

		Ok(())
	}

	fn is_refused_fn(constraint: &Constraint, req: ArcPkgReq) -> bool {
		matches!(
			&constraint.kind,
			ConstraintKind::Refuse(dest) if *dest == req
		)
	}

	/// Whether a package has been refused by an existing constraint
	pub fn is_refused(&self, req: &ArcPkgReq) -> bool {
		self.constraints
			.iter()
			.any(|x| Self::is_refused_fn(x, req.clone()))
	}

	/// Get all refusers of this package
	pub fn get_refusers(&self, req: &ArcPkgReq) -> Vec<PackageID> {
		self.constraints
			.iter()
			.filter_map(|x| {
				if let ConstraintKind::Refuse(dest) = &x.kind {
					if dest == req {
						Some(
							dest.source
								.get_source()
								.map(|source| source.id.clone())
								.unwrap_or("User-refused".into()),
						)
					} else {
						None
					}
				} else {
					None
				}
			})
			.collect()
	}

	/// Whether a compat constraint exists
	pub fn compat_exists(&self, package: ArcPkgReq, compat_package: ArcPkgReq) -> bool {
		self.constraints.iter().any(|x| {
			matches!(
				&x.kind,
				ConstraintKind::Compat(src, dest) if *src == package && *dest == compat_package
			)
		})
	}

	/// Creates an error if this package is disallowed in the constraints
	pub fn check_constraints(&self, req: &ArcPkgReq) -> anyhow::Result<()> {
		if self.is_refused(req) {
			let refusers = self.get_refusers(req);
			bail!(
				"Package '{req}' is incompatible with existing packages {}",
				refusers.join(", ")
			);
		}

		Ok(())
	}

	/// Checks compat constraints to see if new constraints are needed
	pub async fn check_compats(
		&mut self,
		evaluator: &mut E,
		common_input: &E::CommonInput,
	) -> anyhow::Result<()> {
		let mut packages_to_require = Vec::new();
		for constraint in &self.constraints {
			if let ConstraintKind::Compat(package, compat_package) = &constraint.kind {
				if self.is_required(package) && !self.is_required(compat_package) {
					packages_to_require.push(compat_package.clone());
				}
			}
		}
		for package in packages_to_require {
			let props = evaluator
				.get_package_properties(&package, common_input)
				.await
				.context("Failed to get package properties")?;
			self.update_require_constraint(&package, props, RequireConstraint::Require)?;
		}

		Ok(())
	}

	/// Collect all needed packages for final output
	pub fn collect_packages(self) -> Vec<ArcPkgReq> {
		self.constraints
			.iter()
			.filter_map(|x| match &x.kind {
				ConstraintKind::Require(dest, _)
				| ConstraintKind::UserRequire(dest, _)
				| ConstraintKind::Bundle(dest, _) => Some(dest.clone()),
				_ => None,
			})
			.collect()
	}
}

/// A requirement for the installation of the packages
#[derive(Debug)]
struct Constraint {
	kind: ConstraintKind,
}

#[derive(Debug, PartialEq, Eq)]
enum ConstraintKind {
	Require(ArcPkgReq, Vec<String>),
	UserRequire(ArcPkgReq, Vec<String>),
	Refuse(ArcPkgReq),
	Recommend(ArcPkgReq, bool),
	Bundle(ArcPkgReq, Vec<String>),
	Compat(ArcPkgReq, ArcPkgReq),
	Extend(ArcPkgReq),
}

/// A task that needs to be completed for resolution
enum Task<'a, E: PackageEvaluator<'a>> {
	/// Evaluate a package and its relationships
	EvalPackage {
		dest: Arc<PkgRequest>,
		/// For packages with a config
		config: Option<E::ConfiguredPackage>,
	},
}

/// Different types of require constraints
pub enum RequireConstraint {
	/// Require by a package
	Require,
	/// Require by the user
	UserRequire,
	/// A bundle dependency
	Bundle,
}

/// Creates the error message for package context
fn package_context_error_message(package: &PkgRequest) -> String {
	format!("In package '{}'", package.debug_sources())
}
