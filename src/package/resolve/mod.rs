use std::collections::VecDeque;

use anyhow::{bail, Context};

use crate::io::files::paths::Paths;

use super::eval::{EvalConstants, Routine};
use super::reg::{PkgRegistry, PkgRequest, PkgRequestSource};

enum ConstraintKind {
	Require {
		source: Option<PkgRequest>,
		dest: PkgRequest,
	},
	Refuse {
		source: Option<PkgRequest>,
		dest: PkgRequest,
	},
}

impl ConstraintKind {
	/// Whether this constraint was imposed by the user instead of a package
	pub fn is_user_defined(&self) -> bool {
		matches!(
			self,
			Self::Require { source: None, .. } | Self::Refuse { source: None, .. }
		)
	}
}

/// A requirement for the installation of the packages
struct Constraint {
	kind: ConstraintKind,
}

/// A task that needs to be completed for resolution
enum Task {
	EvalDeps {
		source: Option<PkgRequest>,
		dest: PkgRequest,
	},
}

/// State for resolution
struct Resolver {
	tasks: VecDeque<Task>,
	constraints: Vec<Constraint>,
}

impl Resolver {
	/// Whether a package has been refused by an existing constraint
	pub fn is_refused(&self, req: &PkgRequest) -> bool {
		self.constraints.iter().any(|x| {
			matches!(
				&x.kind,
				ConstraintKind::Refuse {
					source: _,
					dest,
				} if dest == req
			)
		})
	}

	/// Collect all needed packages for final output
	pub fn collect_packages(self) -> Vec<PkgRequest> {
		self.constraints
			.iter()
			.filter_map(|x| match &x.kind {
				ConstraintKind::Require { source: _, dest } => Some(dest.clone()),
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
		Task::EvalDeps { source, dest } => {
			let result = reg.eval(&dest, paths, Routine::Install, constants).await?;
			for dep in result.deps.iter().flatten() {
				let req = PkgRequest::new(dep, PkgRequestSource::Dependency);
				if resolver.is_refused(&req) {
					bail!("Package '{req}' is incompatible with existing packages");
				} else {
					resolver.constraints.push(Constraint {
						kind: ConstraintKind::Require {
							source: Some(dest.clone()),
							dest: req.clone(),
						},
					});
					resolver.tasks.push_back(Task::EvalDeps {
						source: Some(dest.clone()),
						dest: req,
					});
				}
			}

			Ok::<(), anyhow::Error>(())
		}
		.with_context(|| package_context_error_message(&source, &dest))?,
	}

	Ok(())
}

/// Find all package dependencies from a set of required packages
pub async fn resolve(
	constants: &EvalConstants,
	paths: &Paths,
	reg: &mut PkgRegistry,
) -> anyhow::Result<Vec<PkgRequest>> {
	let mut resolver = Resolver {
		tasks: VecDeque::new(),
		constraints: Vec::new(),
	};

	// Create the initial EvalDeps from the installed packages
	for req in reg.iter_requests() {
		resolver.constraints.push(Constraint {
			kind: ConstraintKind::Require {
				source: None,
				dest: req.clone(),
			},
		});
		resolver.tasks.push_back(Task::EvalDeps {
			source: None,
			dest: req.clone(),
		});
	}

	while let Some(task) = resolver.tasks.pop_front() {
		resolve_task(task, &mut resolver, reg, constants, paths).await?;
	}

	Ok(resolver.collect_packages())
}

/// Creates the error message for package context
fn package_context_error_message(source: &Option<PkgRequest>, dest: &PkgRequest) -> String {
	if let Some(req) = source {
		format!("In package '{req}'")
	} else {
		format!("In user-required package '{dest}'")
	}
}
