use crate::{output::LauncherOutput, State};
use anyhow::Context;
use mcvm::pkg_crate::metadata::PackageMetadata;
use mcvm::pkg_crate::properties::PackageProperties;
use mcvm::pkg_crate::repo::RepoMetadata;
use mcvm::pkg_crate::{PkgRequest, PkgRequestSource};
use mcvm::shared::output::NoOp;
use mcvm::shared::pkg::PackageSearchParameters;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::{fmt_err, load_config};

#[tauri::command]
pub async fn get_packages(
	state: tauri::State<'_, State>,
	app_handle: tauri::AppHandle,
	repo: &str,
	start: usize,
	end: usize,
	search: Option<&str>,
) -> Result<(Vec<String>, usize), String> {
	let mut output = LauncherOutput::new(state.get_output(app_handle));
	output.set_task("search_packages");
	let mut config =
		fmt_err(load_config(&state.paths, &mut NoOp).context("Failed to load config"))?;

	let params = PackageSearchParameters {
		count: (end - start) as u8,
		search: search.map(|x| x.to_string()),
		categories: Vec::new(),
	};

	let packages = fmt_err(
		config
			.packages
			.search(params, Some(repo), &state.paths, &state.client, &mut output)
			.await
			.context("Failed to get list of available packages"),
	)?;

	let packages = packages.into_iter().map(|x| x.id.to_string());

	// Add search
	let packages = packages.filter(|x| {
		if let Some(search) = &search {
			x.contains(search)
		} else {
			true
		}
	});

	let packages: Vec<_> = packages.collect();

	let available_count = packages.len();

	Ok((
		packages.into_iter().skip(start).take(end - start).collect(),
		available_count,
	))
}

#[tauri::command]
pub async fn get_package_meta(
	state: tauri::State<'_, State>,
	package: &str,
) -> Result<PackageMetadata, String> {
	let mut config =
		fmt_err(load_config(&state.paths, &mut NoOp).context("Failed to load config"))?;

	let meta = fmt_err(
		config
			.packages
			.get_metadata(
				&Arc::new(PkgRequest::any(package, PkgRequestSource::UserRequire)),
				&state.paths,
				&state.client,
				&mut NoOp,
			)
			.await
			.context("Failed to get metadata"),
	)?;

	Ok(meta.clone())
}

#[tauri::command]
pub async fn get_package_props(
	state: tauri::State<'_, State>,
	package: &str,
) -> Result<PackageProperties, String> {
	let mut config =
		fmt_err(load_config(&state.paths, &mut NoOp).context("Failed to load config"))?;

	let props = fmt_err(
		config
			.packages
			.get_properties(
				&Arc::new(PkgRequest::any(package, PkgRequestSource::UserRequire)),
				&state.paths,
				&state.client,
				&mut NoOp,
			)
			.await
			.context("Failed to get properties"),
	)?;

	Ok(props.clone())
}

#[tauri::command]
pub async fn get_package_repos(state: tauri::State<'_, State>) -> Result<Vec<RepoInfo>, String> {
	let mut config =
		fmt_err(load_config(&state.paths, &mut NoOp).context("Failed to load config"))?;

	let mut repos = Vec::new();
	for repo in &mut config.packages.repos {
		let id = repo.get_id().to_string();
		let meta = fmt_err(
			repo.get_metadata(&state.paths, &state.client, &mut NoOp)
				.await
				.context("Failed to get metadata for repository"),
		)?;
		repos.push(RepoInfo {
			id,
			meta: meta.into_owned(),
		})
	}

	Ok(repos)
}

#[derive(Serialize, Deserialize)]
pub struct RepoInfo {
	pub id: String,
	pub meta: RepoMetadata,
}
