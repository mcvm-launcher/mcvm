use std::collections::VecDeque;

use anyhow::{bail, Context};
use color_print::cprintln;

use crate::io::files::paths::Paths;

use super::{EvalConstants, Routine};
use crate::package::reg::{PkgRegistry, PkgRequest, PkgRequestSource};
use crate::package::PkgProfileConfig;

enum ConstraintKind {
	Require(PkgRequest),
	UserRequire(PkgRequest),
	Refuse(PkgRequest),
	Recommend(PkgRequest),
	Bundle(PkgRequest),
}

/// A requirement for the installation of the packages
struct Constraint {
	kind: ConstraintKind,
}

/// A task that needs to be completed for resolution
enum Task {
	EvalDeps {
		dest: PkgRequest,
		constants: Option<EvalConstants>,
	},
}

/// State for resolution
struct Resolver {
	tasks: VecDeque<Task>,
	constraints: Vec<Constraint>,
}

impl Resolver {
	fn is_required_fn(constraint: &Constraint, req: &PkgRequest) -> bool {
		matches!(
			&constraint.kind,
			ConstraintKind::Require(dest)
			| ConstraintKind::UserRequire(dest)
			| ConstraintKind::Bundle(dest) if dest.same_as(req)
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
		self.constraints
			.iter()
			.any(|x| matches!(&x.kind, ConstraintKind::UserRequire(dest) if dest.same_as(req)))
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
			ConstraintKind::Refuse(dest) if dest.same_as(req)
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
					if dest.same_as(req) {
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

	/// Creates an error if this package is disallowed in the constraints
	pub fn check_constraints(&self, req: &PkgRequest) -> anyhow::Result<()> {
		if self.is_refused(&req) {
			let refusers = self.get_refusers(&req);
			bail!(
				"Package '{req}' is incompatible with existing packages {}",
				refusers.join(", ")
			);
		}

		Ok(())
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

/// Resolve a single task
async fn resolve_task(
	task: Task,
	resolver: &mut Resolver,
	reg: &mut PkgRegistry,
	constants: &EvalConstants,
	paths: &Paths,
) -> anyhow::Result<()> {
	match task {
		Task::EvalDeps {
			dest,
			constants: user_constants,
		} => {
			// Pick which evaluation constants to use
			let user_constants = &user_constants;
			let constants = if let Some(constants) = user_constants {
				constants
			} else {
				constants
			};

			// Make sure that this package fits the constraints as well
			resolver.check_constraints(&dest)?;

			let result = reg
				.eval(&dest, paths, Routine::InstallResolve, constants)
				.await?;

			for conflict in result.conflicts {
				let req = PkgRequest::new(
					&conflict,
					PkgRequestSource::Dependency(Box::new(dest.clone())),
				);
				if resolver.is_required(&req) {
					bail!("Package '{req}' is incompatible with existing package '{dest}'");
				}
				resolver.constraints.push(Constraint {
					kind: ConstraintKind::Refuse(req),
				});
			}

			for dep in result.deps.iter().flatten() {
				let req =
					PkgRequest::new(&dep.value, PkgRequestSource::Dependency(Box::new(dest.clone())));
				if dep.explicit && !resolver.is_user_required(&req) {
					bail!("Package '{req}' has been explicitly required by package '{dest}'. This means it must be required by the user in their config.");
				}
				resolver.check_constraints(&req)?;
				if !resolver.is_required(&req) {
					resolver.constraints.push(Constraint {
						kind: ConstraintKind::Require(req.clone()),
					});
					resolver.tasks.push_back(Task::EvalDeps {
						dest: req,
						constants: None,
					});
				}
			}

			for pkg in result.bundled {
				let req =
					PkgRequest::new(&pkg, PkgRequestSource::Dependency(Box::new(dest.clone())));
				resolver.check_constraints(&req)?;
				resolver.remove_require_constraint(&req);
				resolver.constraints.push(Constraint {
					kind: ConstraintKind::Bundle(req.clone()),
				});
				resolver.tasks.push_back(Task::EvalDeps {
					dest: req,
					constants: None,
				});
			}

			for recommendation in result.recommendations {
				let req =
					PkgRequest::new(&recommendation, PkgRequestSource::Dependency(Box::new(dest.clone())));
				resolver.constraints.push(Constraint {
					kind: ConstraintKind::Recommend(req),
				});
			}

			Ok::<(), anyhow::Error>(())
		}
		.with_context(|| package_context_error_message(dest.source.get_source(), &dest))?,
	}

	Ok(())
}

/// Find all package dependencies from a set of required packages
pub async fn resolve(
	packages: &[PkgProfileConfig],
	constants: &EvalConstants,
	paths: &Paths,
	reg: &mut PkgRegistry,
) -> anyhow::Result<Vec<PkgRequest>> {
	let mut resolver = Resolver {
		tasks: VecDeque::new(),
		constraints: Vec::new(),
	};

	// Create the initial EvalDeps from the installed packages
	for config in packages {
		let mut constants = constants.clone();
		constants.features = config.features.clone();
		constants.perms = config.permissions.clone();
		resolver.constraints.push(Constraint {
			kind: ConstraintKind::UserRequire(config.req.clone()),
		});
		resolver.tasks.push_back(Task::EvalDeps {
			dest: config.req.clone(),
			constants: Some(constants),
		});
	}
	for req in reg.iter_requests() {
		if !resolver.is_required(req) {
			resolver.constraints.push(Constraint {
				kind: ConstraintKind::Require(req.clone()),
			});
			resolver.tasks.push_back(Task::EvalDeps {
				dest: req.clone(),
				constants: None,
			});
		}
	}

	while let Some(task) = resolver.tasks.pop_front() {
		resolve_task(task, &mut resolver, reg, constants, paths).await?;
	}

	for constraint in resolver.constraints.iter() {
		if let ConstraintKind::Recommend(package) = &constraint.kind {
			if !resolver.is_required(package) {
				cprintln!("<y>Warning: A package recommends the use of the package '{}', which is not installed.", package);
			}
		}
	}

	Ok(resolver.collect_packages())
}

/// Creates the error message for package context
fn package_context_error_message(source: Option<&PkgRequest>, dest: &PkgRequest) -> String {
	if let Some(req) = source {
		format!("In package '{req}'")
	} else {
		format!("In user-required package '{dest}'")
	}
}
