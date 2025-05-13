use crate::{output::LauncherOutput, State};
use anyhow::Context;
use mcvm::pkg_crate::metadata::PackageMetadata;
use mcvm::pkg_crate::{PkgRequest, PkgRequestSource};
use std::sync::Arc;

use super::{fmt_err, load_config};

#[tauri::command]
pub async fn get_packages(
	state: tauri::State<'_, State>,
	app_handle: tauri::AppHandle,
	start: usize,
	end: usize,
) -> Result<Vec<String>, String> {
	let app_handle = Arc::new(app_handle);

	let mut output = LauncherOutput::new(
		app_handle,
		state.passkeys.clone(),
		state.password_prompt.clone(),
	);
	let mut config =
		fmt_err(load_config(&state.paths, &mut output).context("Failed to load config"))?;

	let mut packages = fmt_err(
		config
			.packages
			.get_all_available_packages(&state.paths, &state.client, &mut output)
			.await
			.context("Failed to get list of available packages"),
	)?;
	packages.sort();

	println!("Packages got");

	Ok(packages
		.into_iter()
		.skip(start)
		.take(end - start)
		.map(|x| x.id.to_string())
		.collect())
}

#[tauri::command]
pub async fn get_package_meta(
	state: tauri::State<'_, State>,
	app_handle: tauri::AppHandle,
	package: &str,
) -> Result<PackageMetadata, String> {
	let app_handle = Arc::new(app_handle);

	let mut output = LauncherOutput::new(
		app_handle,
		state.passkeys.clone(),
		state.password_prompt.clone(),
	);
	let mut config =
		fmt_err(load_config(&state.paths, &mut output).context("Failed to load config"))?;

	let meta = fmt_err(
		config
			.packages
			.get_metadata(
				&Arc::new(PkgRequest::any(package, PkgRequestSource::UserRequire)),
				&state.paths,
				&state.client,
				&mut output,
			)
			.await
			.context("Failed to get metadata"),
	)?;

	Ok(meta.clone())
}
