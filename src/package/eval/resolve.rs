use std::collections::VecDeque;

use anyhow::{bail, Context};
use color_print::cprintln;
use itertools::Itertools;
use reqwest::Client;

use crate::io::files::paths::Paths;

use super::{EvalConstants, EvalParameters, Routine};
use crate::package::reg::{PkgRegistry, PkgRequest, PkgRequestSource};
use crate::package::PkgProfileConfig;

enum ConstraintKind {
	Require(PkgRequest),
	UserRequire(PkgRequest),
	Refuse(PkgRequest),
	Recommend(PkgRequest),
	Bundle(PkgRequest),
	Compat(PkgRequest, PkgRequest),
	Extend(PkgRequest),
}

/// A requirement for the installation of the packages
struct Constraint {
	kind: ConstraintKind,
}

/// A task that needs to be completed for resolution
enum Task {
	/// Evaluate a package and its relationships
	EvalPackage {
		dest: PkgRequest,
		params: Option<EvalParameters>,
	},
}

/// State for resolution
struct Resolver<'a> {
	tasks: VecDeque<Task>,
	constraints: Vec<Constraint>,
	constants: &'a EvalConstants,
	default_params: EvalParameters,
}

impl<'a> Resolver<'a> {
	fn is_required_fn(constraint: &Constraint, req: &PkgRequest) -> bool {
		matches!(
			&constraint.kind,
			ConstraintKind::Require(dest)
			| ConstraintKind::UserRequire(dest)
			| ConstraintKind::Bundle(dest) if dest == req
		)
	}

	/// Whether a package has been required by an existing constraint
	pub fn is_required(&self, req: &PkgRequest) -> bool {
		self.constraints
			.iter()
			.any(|x| Self::is_required_fn(x, req))
	}

	/// Whether a package has been required by the user
	pub fn is_user_required(&self, req: &PkgRequest) -> bool {
		self.constraints.iter().any(|x| {
			matches!(&x.kind, ConstraintKind::UserRequire(dest) if dest == req)
				|| matches!(&x.kind, ConstraintKind::Bundle(dest) if dest.source.is_user_bundled())
		})
	}

	/// Remove the require constraint of a package if it exists
	pub fn remove_require_constraint(&mut self, req: &PkgRequest) {
		let index = self
			.constraints
			.iter()
			.position(|x| Self::is_required_fn(x, req));
		if let Some(index) = index {
			self.constraints.swap_remove(index);
		}
	}

	fn is_refused_fn(constraint: &Constraint, req: &PkgRequest) -> bool {
		matches!(
			&constraint.kind,
			ConstraintKind::Refuse(dest) if dest == req
		)
	}

	/// Whether a package has been refused by an existing constraint
	pub fn is_refused(&self, req: &PkgRequest) -> bool {
		self.constraints.iter().any(|x| Self::is_refused_fn(x, req))
	}

	/// Get all refusers of this package
	pub fn get_refusers(&self, req: &PkgRequest) -> Vec<String> {
		self.constraints
			.iter()
			.filter_map(|x| {
				if let ConstraintKind::Refuse(dest) = &x.kind {
					if dest == req {
						Some(
							dest.source
								.get_source()
								.map(|source| source.name.clone())
								.unwrap_or(String::from("User-refused")),
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
	pub fn compat_exists(&self, package: &PkgRequest, compat_package: &PkgRequest) -> bool {
		self.constraints.iter().any(|x| {
			matches!(
				&x.kind,
				ConstraintKind::Compat(src, dest) if src == package && dest == compat_package
			)
		})
	}

	/// Creates an error if this package is disallowed in the constraints
	pub fn check_constraints(&self, req: &PkgRequest) -> anyhow::Result<()> {
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
	pub fn check_compats(&mut self) {
		let mut constraints_to_add = Vec::new();
		for constraint in &self.constraints {
			if let ConstraintKind::Compat(package, compat_package) = &constraint.kind {
				if self.is_required(package) && !self.is_required(compat_package) {
					constraints_to_add.push(Constraint {
						kind: ConstraintKind::Require(compat_package.clone()),
					});
					self.tasks.push_back(Task::EvalPackage {
						dest: compat_package.clone(),
						params: None,
					});
				}
			}
		}
		self.constraints.extend(constraints_to_add);
	}

	/// Collect all needed packages for final output
	pub fn collect_packages(self) -> Vec<PkgRequest> {
		self.constraints
			.iter()
			.filter_map(|x| match &x.kind {
				ConstraintKind::Require(dest)
				| ConstraintKind::UserRequire(dest)
				| ConstraintKind::Bundle(dest) => Some(dest.clone()),
				_ => None,
			})
			.collect()
	}
}

/// Resolve an EvalPackage task
async fn resolve_eval_package(
	package: PkgRequest,
	params: &Option<EvalParameters>,
	resolver: &mut Resolver<'_>,
	reg: &mut PkgRegistry,
	paths: &Paths,
	client: &Client,
) -> anyhow::Result<()> {
	let params = params.as_ref().unwrap_or(&resolver.default_params).clone();
	// Make sure that this package fits the constraints as well
	resolver
		.check_constraints(&package)
		.context("Package did not fit existing constraints")?;

	let result = reg
		.eval(
			&package,
			paths,
			Routine::InstallResolve,
			resolver.constants,
			params,
			client,
		)
		.await
		.context("Failed to evaluate package")?;

	for conflict in result.conflicts.iter().sorted() {
		let req = PkgRequest::new(
			conflict,
			PkgRequestSource::Refused(Box::new(package.clone())),
		);
		if resolver.is_required(&req) {
			bail!(
				"Package '{}' is incompatible with this package.",
				req.debug_sources(String::new())
			);
		}
		resolver.constraints.push(Constraint {
			kind: ConstraintKind::Refuse(req),
		});
	}

	for dep in result.deps.iter().flatten().sorted() {
		let req = PkgRequest::new(
			&dep.value,
			PkgRequestSource::Dependency(Box::new(package.clone())),
		);
		if dep.explicit && !resolver.is_user_required(&req) {
			bail!("Package '{req}' has been explicitly required by this package. This means it must be required by the user in their config.");
		}
		resolver.check_constraints(&req)?;
		if !resolver.is_required(&req) {
			resolver.constraints.push(Constraint {
				kind: ConstraintKind::Require(req.clone()),
			});
			resolver.tasks.push_back(Task::EvalPackage {
				dest: req,
				params: None,
			});
		}
	}

	for bundled in result.bundled.iter().sorted() {
		let req = PkgRequest::new(
			bundled,
			PkgRequestSource::Bundled(Box::new(package.clone())),
		);
		resolver.check_constraints(&req)?;
		resolver.remove_require_constraint(&req);
		resolver.constraints.push(Constraint {
			kind: ConstraintKind::Bundle(req.clone()),
		});
		resolver.tasks.push_back(Task::EvalPackage {
			dest: req,
			params: None,
		});
	}

	for (check_package, compat_package) in result.compats.iter().sorted() {
		let check_package = PkgRequest::new(
			check_package,
			PkgRequestSource::Dependency(Box::new(package.clone())),
		);
		let compat_package = PkgRequest::new(
			compat_package,
			PkgRequestSource::Dependency(Box::new(package.clone())),
		);
		if !resolver.compat_exists(&check_package, &compat_package) {
			resolver.constraints.push(Constraint {
				kind: ConstraintKind::Compat(check_package, compat_package),
			});
		}
	}

	for extension in result.extensions.iter().sorted() {
		let req = PkgRequest::new(
			extension,
			PkgRequestSource::Dependency(Box::new(package.clone())),
		);
		resolver.constraints.push(Constraint {
			kind: ConstraintKind::Extend(req),
		});
	}

	for recommendation in result.recommendations.iter().sorted() {
		let req = PkgRequest::new(
			recommendation,
			PkgRequestSource::Dependency(Box::new(package.clone())),
		);
		resolver.constraints.push(Constraint {
			kind: ConstraintKind::Recommend(req),
		});
	}

	Ok(())
}

/// Resolve a single task
async fn resolve_task(
	task: Task,
	resolver: &mut Resolver<'_>,
	reg: &mut PkgRegistry,
	paths: &Paths,
	client: &Client,
) -> anyhow::Result<()> {
	match task {
		Task::EvalPackage { dest, params } => {
			resolve_eval_package(dest.clone(), &params, resolver, reg, paths, client)
				.await
				.with_context(|| package_context_error_message(&dest))?;
		}
	}

	Ok(())
}

/// Find all package dependencies from a set of required packages
pub async fn resolve(
	packages: &[PkgProfileConfig],
	constants: &EvalConstants,
	default_params: EvalParameters,
	paths: &Paths,
	reg: &mut PkgRegistry,
) -> anyhow::Result<Vec<PkgRequest>> {
	let mut resolver = Resolver {
		tasks: VecDeque::new(),
		constraints: Vec::new(),
		constants,
		default_params,
	};

	// Create the initial EvalPackage from the installed packages
	for config in packages.iter().sorted_by_key(|x| &x.req) {
		let params = EvalParameters {
			side: resolver.default_params.side,
			features: config.features.clone(),
			perms: config.permissions.clone(),
			stability: config.stability,
		};
		resolver.constraints.push(Constraint {
			kind: ConstraintKind::UserRequire(config.req.clone()),
		});
		resolver.tasks.push_back(Task::EvalPackage {
			dest: config.req.clone(),
			params: Some(params),
		});
	}

	let client = Client::new();

	while let Some(task) = resolver.tasks.pop_front() {
		resolve_task(task, &mut resolver, reg, paths, &client).await?;
		resolver.check_compats();
	}

	for constraint in resolver.constraints.iter() {
		match &constraint.kind {
			ConstraintKind::Recommend(package) => {
				if !resolver.is_required(package) {
					let source = package.source.get_source();
					if let Some(source) = source {
						cprintln!(
							"<y>Warning: The package '{}' recommends the use of the package '{}', which is not installed.",
							source.debug_sources(String::new()),
							package
						);
					} else {
						cprintln!(
							"<y>Warning: A package recommends the use of the package '{}', which is not installed.",
							package
						);
					}
				}
			}
			ConstraintKind::Extend(package) => {
				if !resolver.is_required(package) {
					let source = package.source.get_source();
					if let Some(source) = source {
						bail!(
							"The package '{}' extends the functionality of the package '{}', which is not installed.",
							source.debug_sources(String::new()),
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

	Ok(resolver.collect_packages())
}

/// Creates the error message for package context
fn package_context_error_message(package: &PkgRequest) -> String {
	format!("In package '{}'", package.debug_sources(String::new()))
}
