// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Commands for Tauri
mod commands;
/// Storage and reading for GUI-specific data
mod data;
/// MCVM output for the launcher frontend
mod output;

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Context;
use commands::launch::UpdateRunStateEvent;
use data::LauncherData;
use mcvm::core::auth_crate::mc::ClientId;
use mcvm::core::{net::download::Client, user::UserManager};
use mcvm::io::paths::Paths;
use mcvm::shared::id::InstanceID;
use output::PromptResponse;
use serde::{Deserialize, Serialize};
use tauri::async_runtime::Mutex;
use tauri::Manager;
use tokio::task::JoinHandle;

fn main() {
	let state = tauri::async_runtime::block_on(async { State::new().await })
		.expect("Error when initializing application state");
	let launched_games = state.launched_games.clone();
	tauri::Builder::default()
		.manage(state)
		.setup(move |app| {
			app.listen_global("update_run_state", move |event| {
				let payload: UpdateRunStateEvent = serde_json::from_str(
					event
						.payload()
						.expect("Update run state event should have payload"),
				)
				.expect("Failed to deserialize state update");
				let mut lock = tauri::async_runtime::block_on(launched_games.lock());
				if let Some(instance) = lock.get_mut(&InstanceID::from(payload.instance)) {
					instance.state = payload.state;
				}
			});

			Ok(())
		})
		.invoke_handler(tauri::generate_handler![
			commands::launch::launch_game,
			commands::launch::stop_game,
			commands::launch::answer_password_prompt,
			commands::instance::get_instances,
			commands::instance::get_profiles,
			commands::instance::get_instance_groups,
			commands::launch::get_running_instances,
			commands::launch::set_running_instance_state,
			commands::instance::pin_instance,
			commands::instance::get_instance_config,
			commands::instance::get_profile_config,
			commands::instance::get_global_profile,
			commands::instance::write_instance_config,
			commands::instance::write_profile_config,
			commands::instance::write_global_profile,
			commands::package::get_packages,
			commands::package::get_package_meta,
			commands::package::get_package_props,
			commands::plugin::get_plugins,
			commands::plugin::enable_disable_plugin,
			commands::plugin::install_plugin,
			commands::plugin::uninstall_plugin,
			commands::user::get_users,
			commands::user::select_user,
		])
		.run(tauri::generate_context!())
		.expect("Error while running tauri application");
}

/// State for the Tauri application
pub struct State {
	pub data: Mutex<LauncherData>,
	pub launched_games: Arc<Mutex<HashMap<InstanceID, RunningInstance>>>,
	pub paths: Paths,
	pub client: Client,
	pub user_manager: Mutex<UserManager>,
	/// Map of users to their already entered passkeys
	pub passkeys: Arc<Mutex<HashMap<String, String>>>,
	pub password_prompt: PromptResponse,
}

impl State {
	async fn new() -> anyhow::Result<Self> {
		let paths = Paths::new().await?;
		Ok(Self {
			data: Mutex::new(LauncherData::open(&paths).context("Failed to open launcher data")?),
			launched_games: Arc::new(Mutex::new(HashMap::new())),
			paths,
			client: Client::new(),
			user_manager: Mutex::new(UserManager::new(get_ms_client_id())),
			passkeys: Arc::new(Mutex::new(HashMap::new())),
			password_prompt: PromptResponse::new(Mutex::new(None)),
		})
	}
}

/// A running instance
pub struct RunningInstance {
	/// The ID of the instance
	pub id: InstanceID,
	/// The tokio task for the running instance
	pub task: JoinHandle<anyhow::Result<()>>,
	/// State of the instance in it's lifecycle
	pub state: RunState,
}

/// State of a running instance
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "snake_case")]
pub enum RunState {
	NotStarted,
	Preparing,
	Running,
}

/// Get the Microsoft client ID
pub fn get_ms_client_id() -> ClientId {
	ClientId::new(get_raw_ms_client_id().to_string())
}

const fn get_raw_ms_client_id() -> &'static str {
	if let Some(id) = option_env!("MCVM_MS_CLIENT_ID") {
		id
	} else {
		// Please don't use my client ID :)
		"402abc71-43fb-45c1-b230-e7fc9d4485fe"
	}
}
