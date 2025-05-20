use crate::output::LauncherOutput;
use crate::State;
use anyhow::Context;
use mcvm::core::user::UserKind;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use super::{fmt_err, load_config};

#[tauri::command]
pub async fn get_users(
	state: tauri::State<'_, State>,
	app_handle: tauri::AppHandle,
) -> Result<(Option<String>, HashMap<String, UserInfo>), String> {
	let data = state.data.lock().await;

	let app_handle = Arc::new(app_handle);

	let mut output = LauncherOutput::new(
		app_handle,
		state.passkeys.clone(),
		state.password_prompt.clone(),
	);
	let mut config =
		fmt_err(load_config(&state.paths, &mut output).context("Failed to load config"))?;
	let user_ids: Vec<_> = config.users.iter_users().map(|x| x.0.clone()).collect();

	let mut users = HashMap::with_capacity(user_ids.len());
	config.users.set_offline(true);
	for id in user_ids {
		let _ = config
			.users
			.authenticate_user(&id, &state.paths.core, &state.client, &mut output)
			.await
			.context("Failed to authenticate user");

		let user = config.users.get_user(&id).expect("User should exist");

		let ty = match user.get_kind() {
			UserKind::Microsoft { .. } => UserType::Microsoft,
			UserKind::Demo => UserType::Demo,
			UserKind::Unknown(..) => UserType::Other,
		};

		let info = UserInfo {
			id: id.to_string(),
			r#type: ty,
			username: user.get_name().cloned(),
			uuid: user.get_uuid().cloned(),
		};

		users.insert(id.to_string(), info);
	}

	let current_user = data
		.current_user
		.clone()
		.filter(|x| config.users.user_exists(x));

	Ok((current_user, users))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfo {
	pub id: String,
	pub r#type: UserType,
	pub username: Option<String>,
	pub uuid: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UserType {
	Microsoft,
	Demo,
	Other,
}

#[tauri::command]
pub async fn select_user(state: tauri::State<'_, State>, user: &str) -> Result<(), String> {
	let mut data = state.data.lock().await;

	data.current_user = Some(user.to_string());
	fmt_err(data.write(&state.paths))?;

	Ok(())
}
