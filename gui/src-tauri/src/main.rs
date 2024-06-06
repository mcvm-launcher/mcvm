// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Commands for Tauri
mod commands;
/// MCVM output for the launcher frontend
mod output;

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
		])
		.run(tauri::generate_context!())
		.expect("Error while running tauri application");
}

/// State for the Tauri application
pub struct State {
	pub launched_game: Mutex<Option<JoinHandle<anyhow::Result<()>>>>,
	pub paths: Paths,
	pub client: Client,
	pub user_manager: Mutex<UserManager>,
	pub password_prompt: PromptResponse,
}

impl State {
	async fn new() -> anyhow::Result<Self> {
		Ok(Self {
			launched_game: Mutex::new(None),
			paths: Paths::new().await?,
			client: Client::new(),
			user_manager: Mutex::new(UserManager::new(get_ms_client_id())),
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
