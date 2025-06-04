use std::{
	collections::HashSet,
	path::{Path, PathBuf},
};

use anyhow::Context;
use mcvm_core::io::{files::create_leading_dirs, json_from_file, json_to_file};
use mcvm_net::{
	download::Client,
	modrinth::{self, Member, Project, Version},
};
use mcvm_pkg_gen::relation_substitution::RelationSubMethod;
use mcvm_plugin::{api::CustomPlugin, hooks::CustomRepoQueryResult};

fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::from_manifest_file("modrinth", include_str!("plugin.json"))?;

	plugin.query_custom_package_repository(|ctx, arg| {
		if arg.repository != "modrinth" {
			return Ok(None);
		}

		let runtime = tokio::runtime::Runtime::new()?;
		let client = Client::new();

		let data_dir = ctx.get_data_dir()?;
		let storage_dirs = StorageDirs::new(&data_dir);
		let project_info = runtime
			.block_on(get_cached_project(&arg.package, &storage_dirs, &client))
			.context("Failed to get project")?;
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
					.context("Failed to get project")?
					.context("Project does not exist")?;

				Ok(project_info.project.id)
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

		Ok(Some(CustomRepoQueryResult {
			contents: package,
			content_type: mcvm::pkg_crate::PackageContentType::Declarative,
			flags: HashSet::new(),
		}))
	})?;

	Ok(())
}

/// Gets a cached Modrinth project and it's versions or downloads it
async fn get_cached_project(
	project: &str,
	storage_dirs: &StorageDirs,
	client: &Client,
) -> anyhow::Result<Option<ProjectInfo>> {
	let project_path = storage_dirs.projects.join(project);
	let members_path = storage_dirs.members.join(project);
	let (project, members) = if project_path.exists() {
		let project = json_from_file(&project_path).context("Failed to read project from file")?;
		let members =
			json_from_file(&members_path).context("Failed to read project members from file")?;

		(project, members)
	} else {
		let project_task = {
			let project = project.to_string();
			let client = client.clone();
			tokio::spawn(async move { modrinth::get_project_optional(&project, &client).await })
		};

		let members_task = {
			let project = project.to_string();
			let client = client.clone();
			tokio::spawn(async move { modrinth::get_project_team(&project, &client).await })
		};

		let (project, members) = tokio::try_join!(project_task, members_task)
			.context("Failed to get project and members")?;
		let project = project?;
		let members = members?;

		let project = match project {
			Some(project) => project,
			None => return Ok(None),
		};

		// Get a list of missing versions
		let mut missing = Vec::new();
		for version in &project.versions {
			let path = storage_dirs.versions.join(version);
			if !path.exists() {
				missing.push(version);
			}
		}

		if !missing.is_empty() {
			let versions = modrinth::get_project_versions(&project.id, client)
				.await
				.context("Failed to get project versions")?;

			for version in versions {
				let path = storage_dirs.versions.join(&version.id);
				let _ = create_leading_dirs(&path);
				json_to_file(path, &version).context("Failed to write version to file")?;
			}
		}

		let _ = create_leading_dirs(&project_path);
		// TODO: Store both the id and slug together, hardlinked to each other, to cache no matter which method is used to request
		let _ = json_to_file(&project_path, &project);

		let _ = create_leading_dirs(&members_path);
		let _ = json_to_file(&members_path, &members);

		(project, members)
	};

	let mut versions = Vec::with_capacity(project.versions.len());
	for version in &project.versions {
		let path = storage_dirs.versions.join(version);
		let version = json_from_file(path).context("Failed to read version from file")?;

		versions.push(version);
	}

	Ok(Some(ProjectInfo {
		project,
		versions,
		members,
	}))
}

/// Project data, versions, and team members for a Modrinth project
struct ProjectInfo {
	project: Project,
	versions: Vec<Version>,
	members: Vec<Member>,
}

/// Storage directories
#[derive(Clone)]
struct StorageDirs {
	projects: PathBuf,
	versions: PathBuf,
	members: PathBuf,
}

impl StorageDirs {
	pub fn new(data_dir: &Path) -> Self {
		let modrinth_dir = data_dir.join("internal/modrinth");
		Self {
			projects: modrinth_dir.join("projects"),
			versions: modrinth_dir.join("versions"),
			members: modrinth_dir.join("members"),
		}
	}
}
