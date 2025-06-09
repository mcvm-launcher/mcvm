use crate::output::LauncherOutput;
use crate::State;
use anyhow::Context;
use mcvm::plugin::PluginManager;
use mcvm::plugin_crate::hooks::{
	AddSidebarButtons, GetPage, InjectPageScript, InjectPageScriptArg, SidebarButton,
};
use mcvm::shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm::{plugin::install::get_verified_plugins, shared::output::NoOp};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use super::{fmt_err, load_config};

#[tauri::command]
pub async fn get_plugins(
	state: tauri::State<'_, State>,
	app_handle: tauri::AppHandle,
) -> Result<Vec<PluginInfo>, String> {
	let config =
		fmt_err(PluginManager::open_config(&state.paths).context("Failed to open plugin config"))?;

	let downloaded = fmt_err(
		PluginManager::get_available_plugins(&state.paths)
			.context("Failed to get available plugins"),
	)?;

	let downloaded = downloaded.into_iter().filter_map(|x| {
		let id = x.0;
		let manifest = PluginManager::read_plugin_manifest(&id, &state.paths).ok()?;

		Some(PluginInfo {
			enabled: config.plugins.contains(&id),
			id,
			name: manifest.name,
			description: manifest.description,
			installed: true,
		})
	});

	let mut output = LauncherOutput::new(state.get_output(app_handle));
	output.set_task("get_plugins");
	let mut process = output.get_process();
	process.display(
		MessageContents::StartProcess("Getting plugins".into()),
		MessageLevel::Important,
	);
	let verified_plugins = fmt_err(
		get_verified_plugins(&state.client)
			.await
			.context("Failed to get verified plugins"),
	)?;
	process.display(
		MessageContents::Success("Plugins Acquired".into()),
		MessageLevel::Important,
	);

	let verified_plugins = verified_plugins.into_values().map(|x| PluginInfo {
		id: x.id,
		name: x.name,
		description: Some(x.description),
		enabled: false,
		installed: false,
	});

	let mut all: Vec<_> = downloaded.chain(verified_plugins).collect();
	all.sort_by_cached_key(|x| x.id.clone());

	Ok(all)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PluginInfo {
	pub id: String,
	pub name: Option<String>,
	pub description: Option<String>,
	pub enabled: bool,
	pub installed: bool,
}

#[tauri::command]
pub async fn enable_disable_plugin(
	state: tauri::State<'_, State>,
	plugin: &str,
	enabled: bool,
) -> Result<(), String> {
	if enabled {
		fmt_err(
			PluginManager::enable_plugin(plugin, &state.paths).context("Failed to enable plugin"),
		)?;
	} else {
		fmt_err(
			PluginManager::disable_plugin(plugin, &state.paths).context("Failed to disable plugin"),
		)?;
	}

	Ok(())
}

#[tauri::command]
pub async fn install_plugin(state: tauri::State<'_, State>, plugin: &str) -> Result<(), String> {
	let verified_list = fmt_err(
		get_verified_plugins(&state.client)
			.await
			.context("Failed to get verified plugin list"),
	)?;

	let Some(plugin) = verified_list.get(plugin) else {
		return Err(format!("Unknown plugin '{plugin}'"));
	};

	fmt_err(
		plugin
			.install(None, &state.paths, &state.client, &mut NoOp)
			.await
			.context("Failed to install plugin"),
	)?;

	Ok(())
}

#[tauri::command]
pub async fn uninstall_plugin(state: tauri::State<'_, State>, plugin: &str) -> Result<(), String> {
	fmt_err(
		PluginManager::uninstall_plugin(plugin, &state.paths).context("Failed to uninstall plugin"),
	)?;

	Ok(())
}

#[tauri::command]
pub async fn get_page_inject_script(
	state: tauri::State<'_, State>,
	app_handle: tauri::AppHandle,
	page: String,
	object: Option<String>,
) -> Result<Option<String>, String> {
	let mut output = LauncherOutput::new(state.get_output(app_handle));

	let config = fmt_err(load_config(&state.paths, &mut NoOp).context("Failed to load config"))?;

	let arg = InjectPageScriptArg { page, object };
	let results = fmt_err(config.plugins.call_hook(
		InjectPageScript,
		&arg,
		&state.paths,
		&mut output,
	))?;

	let mut out = String::new();
	for result in results {
		let result = fmt_err(result.result(&mut output))?;
		out.push_str(&result);
	}

	Ok(Some(out))
}

#[tauri::command]
pub async fn get_sidebar_buttons(
	state: tauri::State<'_, State>,
	app_handle: tauri::AppHandle,
) -> Result<Vec<SidebarButton>, String> {
	let mut output = LauncherOutput::new(state.get_output(app_handle));

	let config = fmt_err(load_config(&state.paths, &mut NoOp).context("Failed to load config"))?;

	let results = fmt_err(config.plugins.call_hook(
		AddSidebarButtons,
		&(),
		&state.paths,
		&mut output,
	))?;

	let mut out = Vec::new();
	for result in results {
		let result = fmt_err(result.result(&mut output))?;
		out.extend(result);
	}

	Ok(out)
}

#[tauri::command]
pub async fn get_plugin_page(
	state: tauri::State<'_, State>,
	app_handle: tauri::AppHandle,
	page: &str,
) -> Result<Option<String>, String> {
	let mut output = LauncherOutput::new(state.get_output(app_handle));

	let config = fmt_err(load_config(&state.paths, &mut NoOp).context("Failed to load config"))?;

	let results = fmt_err(config.plugins.call_hook(
		GetPage,
		&page.to_string(),
		&state.paths,
		&mut output,
	))?;

	for result in results {
		let result = fmt_err(result.result(&mut output))?;
		if let Some(result) = result {
			return Ok(Some(result));
		}
	}

	Ok(None)
}
