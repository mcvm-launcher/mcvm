use crate::State;
use anyhow::Context;
use mcvm::{
	plugin_crate::hooks::{AddSupportedGameModifications, SupportedGameModifications},
	shared::output::NoOp,
};

use super::{fmt_err, load_config};

#[tauri::command]
pub async fn get_supported_game_modifications(
	state: tauri::State<'_, State>,
) -> Result<Vec<SupportedGameModifications>, String> {
	let config = fmt_err(load_config(&state.paths, &mut NoOp).context("Failed to load config"))?;

	let results = fmt_err(
		config
			.plugins
			.call_hook(AddSupportedGameModifications, &(), &state.paths, &mut NoOp)
			.context("Failed to get supported game modifications from plugins"),
	)?;
	let mut out = Vec::with_capacity(results.len());
	for result in results {
		let result = fmt_err(result.result(&mut NoOp))?;
		out.push(result);
	}

	Ok(out)
}
