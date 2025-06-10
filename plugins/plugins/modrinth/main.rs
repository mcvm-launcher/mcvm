use std::{
	collections::HashSet,
	path::{Path, PathBuf},
};

use anyhow::Context;
use mcvm_core::io::{
	files::{create_leading_dirs, update_hardlink},
	json_from_file, json_to_file,
};
use mcvm_net::{
	download::Client,
	modrinth::{self, Member, Project, Version},
};
use mcvm_pkg_gen::relation_substitution::RelationSubMethod;
use mcvm_plugin::{api::CustomPlugin, hooks::CustomRepoQueryResult};
use mcvm_shared::pkg::PackageSearchResults;
use serde::{Deserialize, Serialize};

fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::from_manifest_file("modrinth", include_str!("plugin.json"))?;

	plugin.query_custom_package_repository(|ctx, arg| {
		if arg.repository != "modrinth" {
			return Ok(None);
		}

		let data_dir = ctx.get_data_dir()?;
		let storage_dirs = StorageDirs::new(&data_dir);
		// Check for a cached package
		let path = storage_dirs.packages.join(&arg.package);
		if path.exists() {
			if let Ok(package) = std::fs::read_to_string(&path) {
				return Ok(Some(CustomRepoQueryResult {
					contents: package,
					content_type: mcvm::pkg_crate::PackageContentType::Declarative,
					flags: HashSet::new(),
				}));
			}
		}

		let runtime = tokio::runtime::Runtime::new()?;
		let client = Client::new();

		let project_info = runtime
			.block_on(get_cached_project(&arg.package, &storage_dirs, &client))
			.with_context(|| format!("Failed to get project {}", arg.package))?;
		let Some(project_info) = project_info else {
			return Ok(None);
		};

		let relation_sub_function = {
			let client = client.clone();
			let storage_dirs = storage_dirs.clone();
			let runtime = tokio::runtime::Runtime::new()?;

			move |relation: &str| {
				let project_info = runtime
					.block_on(get_cached_project(relation, &storage_dirs, &client))
					.context("Failed to get project")?;
				if let Some(project_info) = project_info {
					Ok(project_info.project.id)
				} else {
					// Theres a LOT of broken Modrinth projects
					Ok("none".into())
				}
			}
		};

		let package = mcvm_pkg_gen::modrinth::gen(
			project_info.project,
			&project_info.versions,
			&project_info.members,
			RelationSubMethod::Function(Box::new(relation_sub_function)),
			&[],
			true,
			true,
		)
		.context("Failed to generate MCVM package")?;
		let package =
			serde_json::to_string_pretty(&package).context("Failed to serialized package")?;

		let _ = create_leading_dirs(&path);
		let _ = std::fs::write(path, &package);

		Ok(Some(CustomRepoQueryResult {
			contents: package,
			content_type: mcvm::pkg_crate::PackageContentType::Declarative,
			flags: HashSet::new(),
		}))
	})?;

	plugin.search_custom_package_repository(|_, arg| {
		if arg.repository != "modrinth" {
			return Ok(PackageSearchResults::default());
		}

		let client = Client::new();
		let runtime = tokio::runtime::Runtime::new()?;

		let (projects, total_results) = runtime.block_on(async move {
			let results = modrinth::search_projects(arg.parameters, &client, false)
				.await
				.context("Failed to search projects from the API")?;

			let projects = results.hits.into_iter().map(|x| x.slug);

			Ok::<_, anyhow::Error>((projects, results.total_hits))
		})?;

		Ok(PackageSearchResults {
			results: projects.collect(),
			total_results,
		})
	})?;

	Ok(())
}

/// Gets a cached Modrinth project and it's versions or downloads it
async fn get_cached_project(
	project_id: &str,
	storage_dirs: &StorageDirs,
	client: &Client,
) -> anyhow::Result<Option<ProjectInfo>> {
	let project_path = storage_dirs.projects.join(project_id);
	let project_info = if project_path.exists() {
		let project_info =
			json_from_file(&project_path).context("Failed to read project info from file")?;

		project_info
	} else {
		let project_task = {
			let project = project_id.to_string();
			let client = client.clone();
			tokio::spawn(async move { modrinth::get_project_optional(&project, &client).await })
		};

		let members_task = {
			let project = project_id.to_string();
			let client = client.clone();
			tokio::spawn(async move { modrinth::get_project_team(&project, &client).await })
		};

		let versions_task = {
			let project = project_id.to_string();
			let client = client.clone();
			tokio::spawn(async move { modrinth::get_project_versions(&project, &client).await })
		};

		let (project, members, versions) = tokio::join!(project_task, members_task, versions_task);
		let project = project??;
		let project = match project {
			Some(project) => project,
			None => return Ok(None),
		};

		let members = members.context("Failed to get project members")??;
		let versions = versions.context("Failed to get project versions")??;

		let project_info = ProjectInfo {
			project: project,
			versions: versions,
			members: members,
		};

		let id_path = storage_dirs.projects.join(&project_info.project.id);
		let slug_path = storage_dirs.projects.join(&project_info.project.slug);
		let _ = create_leading_dirs(&id_path);
		let _ = json_to_file(&id_path, &project_info);
		let _ = update_hardlink(&id_path, &slug_path);

		project_info
	};

	Ok(Some(project_info))
}

/// Project data, versions, and team members for a Modrinth project
#[derive(Serialize, Deserialize)]
struct ProjectInfo {
	project: Project,
	versions: Vec<Version>,
	members: Vec<Member>,
}

/// Storage directories
#[derive(Clone)]
struct StorageDirs {
	projects: PathBuf,
	packages: PathBuf,
}

impl StorageDirs {
	pub fn new(data_dir: &Path) -> Self {
		let modrinth_dir = data_dir.join("internal/modrinth");
		Self {
			projects: modrinth_dir.join("projects"),
			packages: modrinth_dir.join("packages"),
		}
	}
}
