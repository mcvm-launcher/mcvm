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
use data::LauncherData;
use mcvm::core::auth_crate::mc::ClientId;
use mcvm::core::{net::download::Client, user::UserManager};
use mcvm::io::paths::Paths;
use output::PromptResponse;
use tauri::async_runtime::Mutex;
use tokio::task::JoinHandle;

fn main() {
	let state = tauri::async_runtime::block_on(async { State::new().await })
		.expect("Error when initializing application state");
	tauri::Builder::default()
		.manage(state)
		.invoke_handler(tauri::generate_handler![
			commands::launch_game,
			commands::stop_game,
			commands::answer_password_prompt,
			commands::get_instances,
			commands::get_instance_groups,
			commands::pin_instance,
		])
		.run(tauri::generate_context!())
		.expect("Error while running tauri application");
}

/// State for the Tauri application
pub struct State {
	pub data: Mutex<LauncherData>,
	pub launched_game: Mutex<Option<JoinHandle<anyhow::Result<()>>>>,
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
			launched_game: Mutex::new(None),
			paths,
			client: Client::new(),
			user_manager: Mutex::new(UserManager::new(get_ms_client_id())),
			passkeys: Arc::new(Mutex::new(HashMap::new())),
			password_prompt: PromptResponse::new(Mutex::new(None)),
		})
	}
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
