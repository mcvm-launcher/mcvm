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

		let runtime = tokio::runtime::Runtime::new()?;
		let client = Client::new();

		runtime.block_on(query_package(&arg.package, &client, &storage_dirs))
	})?;

	plugin.preload_packages(|ctx, arg| {
		if arg.repository != "modrinth" {
			return Ok(());
		}

		let data_dir = ctx.get_data_dir()?;
		let storage_dirs = StorageDirs::new(&data_dir);

		let runtime = tokio::runtime::Runtime::new()?;
		let client = Client::new();

		runtime.block_on(async move {
			let mut tasks = tokio::task::JoinSet::new();
			for package in arg.packages {
				let client = client.clone();
				let storage_dirs = storage_dirs.clone();

				tasks.spawn(async move { query_package(&package, &client, &storage_dirs).await });
			}

			while let Some(task) = tasks.join_next().await {
				let _ = task??;
			}

			Ok::<(), anyhow::Error>(())
		})?;

		Ok(())
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

/// Queries for a Modrinth package
async fn query_package(
	id: &str,
	client: &Client,
	storage_dirs: &StorageDirs,
) -> anyhow::Result<Option<CustomRepoQueryResult>> {
	let package_or_project = get_cached_package_or_project(id, storage_dirs, &client)
		.await
		.with_context(|| format!("Failed to get cached package or project '{id}'"))?;
	let Some(package_or_project) = package_or_project else {
		return Ok(None);
	};

	let package = match package_or_project {
		PackageOrProjectInfo::Package { package, .. } => package,
		PackageOrProjectInfo::ProjectInfo(project_info) => {
			let relation_sub_function = {
				let client = client.clone();
				let storage_dirs = storage_dirs.clone();

				async move |relation: &str| {
					let package_or_project =
						get_cached_package_or_project(relation, &storage_dirs, &client)
							.await
							.context("Failed to get cached data")?;
					if let Some(package_or_project) = package_or_project {
						let id = match package_or_project {
							PackageOrProjectInfo::Package { slug, .. } => slug,
							PackageOrProjectInfo::ProjectInfo(info) => info.project.slug,
						};
						Ok(id)
					} else {
						// Theres a LOT of broken Modrinth projects
						Ok("none".into())
					}
				}
			};

			let id = project_info.project.id.clone();
			let slug = project_info.project.slug.clone();

			let package = mcvm_pkg_gen::modrinth::gen(
				project_info.project,
				&project_info.versions,
				&project_info.members,
				relation_sub_function,
				&[],
				true,
				true,
			)
			.await
			.context("Failed to generate MCVM package")?;
			let package =
				serde_json::to_string_pretty(&package).context("Failed to serialized package")?;

			let package_data = format!("{id};{slug};{package}");

			let id_path = storage_dirs.packages.join(&id);
			let slug_path = storage_dirs.packages.join(&slug);
			let _ = create_leading_dirs(&id_path);
			let _ = std::fs::write(&id_path, &package_data);
			let _ = update_hardlink(&id_path, &slug_path);

			package
		}
	};

	Ok(Some(CustomRepoQueryResult {
		contents: package,
		content_type: mcvm::pkg_crate::PackageContentType::Declarative,
		flags: HashSet::new(),
	}))
}

/// Gets a cached package or project info
async fn get_cached_package_or_project(
	project_id: &str,
	storage_dirs: &StorageDirs,
	client: &Client,
) -> anyhow::Result<Option<PackageOrProjectInfo>> {
	let package_path = storage_dirs.packages.join(project_id);
	if package_path.exists() {
		if let Ok(data) = std::fs::read_to_string(&package_path) {
			let mut elems = data.splitn(3, ";");
			let id = elems.next().context("Missing")?;
			let slug = elems.next().context("Missing")?;
			let package = elems.next().context("Missing")?;
			// Remove the projects to save space, we don't need it anymore
			let _ = std::fs::remove_file(storage_dirs.projects.join(id));
			let _ = std::fs::remove_file(storage_dirs.projects.join(slug));
			return Ok(Some(PackageOrProjectInfo::Package {
				id: id.to_string(),
				slug: slug.to_string(),
				package: package.to_string(),
			}));
		}
	}

	get_cached_project(project_id, storage_dirs, client)
		.await
		.map(|x| x.map(PackageOrProjectInfo::ProjectInfo))
}

/// Gets a cached Modrinth project and it's versions or downloads it
async fn get_cached_project(
	project_id: &str,
	storage_dirs: &StorageDirs,
	client: &Client,
) -> anyhow::Result<Option<ProjectInfo>> {
	let project_path = storage_dirs.projects.join(project_id);
	// If a project does not exist, we create a dummy file so that we know not to fetch it again
	let does_not_exist_path = storage_dirs
		.projects
		.join(format!("__missing__{project_id}"));
	if does_not_exist_path.exists() {
		return Ok(None);
	}

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
		let project = project
			.context("Failed to get project")?
			.context("Failed to get project")?;
		let project = match project {
			Some(project) => project,
			None => {
				let file = std::fs::File::create(does_not_exist_path);
				std::mem::drop(file);
				return Ok(None);
			}
		};

		let members = members
			.context("Failed to get project members")?
			.context("Failed to get project members")?;
		let versions = versions
			.context("Failed to get project versions")?
			.context("Failed to get project versions")?;

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

enum PackageOrProjectInfo {
	Package {
		#[allow(dead_code)]
		id: String,
		slug: String,
		package: String,
	},
	ProjectInfo(ProjectInfo),
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
