use crate::{output::LauncherOutput, State};
use anyhow::Context;
use mcvm::{
	plugin_crate::hooks::{AddSupportedGameModifications, SupportedGameModifications},
};
use std::sync::Arc;

use super::{fmt_err, load_config};

#[tauri::command]
pub async fn get_supported_game_modifications(
	state: tauri::State<'_, State>,
	app_handle: tauri::AppHandle,
) -> Result<Vec<SupportedGameModifications>, String> {
	let app_handle = Arc::new(app_handle);

	let mut output = LauncherOutput::new(
		app_handle,
		state.passkeys.clone(),
		state.password_prompt.clone(),
	);
	let config = fmt_err(load_config(&state.paths, &mut output).context("Failed to load config"))?;

	let results = fmt_err(
		config
			.plugins
			.call_hook(
				AddSupportedGameModifications,
				&(),
				&state.paths,
				&mut output,
			)
			.context("Failed to get supported game modifications from plugins"),
	)?;
	let mut out = Vec::with_capacity(results.len());
	for result in results {
		let result = fmt_err(result.result(&mut output))?;
		out.push(result);
	}

	Ok(out)
}
