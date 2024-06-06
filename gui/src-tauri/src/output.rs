use std::{sync::Arc, time::Duration};

use anyhow::Context;
use mcvm::shared::output::{MCVMOutput, Message, MessageContents, MessageLevel};
use serde::Serialize;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

/// Response to a prompt in the frontend, shared with a mutex
pub type PromptResponse = Arc<Mutex<Option<String>>>;

pub struct LauncherOutput {
	app: AppHandle,
	password_prompt: PromptResponse,
}

impl LauncherOutput {
	pub fn new(app: AppHandle, password_prompt: PromptResponse) -> Self {
		Self {
			app,
			password_prompt,
		}
	}

	pub fn get_app_handle(self) -> AppHandle {
		self.app
	}
}

#[async_trait::async_trait]
impl MCVMOutput for LauncherOutput {
	fn display_text(&mut self, text: String, _level: MessageLevel) {
		self.disp(text);
	}

	fn display_message(&mut self, message: Message) {
		if !message.level.at_least(&MessageLevel::Extra) {
			return;
		}
		match message.contents {
			MessageContents::Associated(assoc, msg) => match *assoc {
				MessageContents::Progress { current, total } => {
					let _ = self.app.emit_all(
						"mcvm_output_progress",
						AssociatedProgressEvent {
							current,
							total,
							message: msg.default_format(),
						},
					);
				}
				_ => self.disp(format!(
					"({}) {}",
					assoc.default_format(),
					msg.default_format()
				)),
			},
			MessageContents::Header(text) => {
				let _ = self.app.emit_all("mcvm_output_header", MessageEvent(text));
			}
			msg => self.disp(msg.default_format()),
		}
	}

	async fn prompt_password(&mut self, message: MessageContents) -> anyhow::Result<String> {
		println!("Starting password prompt");
		self.app
			.emit_all("mcvm_display_password_prompt", message.default_format())
			.context("Failed to display password prompt to user")?;

		// Block this thread, checking every interval if the prompt has been filled
		// Weird lint
		#[allow(unused_assignments)]
		let mut result = None;
		loop {
			println!("Waiting for password...");
			if let Some(answer) = self.password_prompt.lock().await.take() {
				result = Some(answer);
				break;
			}
			tokio::time::sleep(Duration::from_millis(200)).await;
		}

		Ok(result.unwrap())
	}

	async fn prompt_new_password(&mut self, message: MessageContents) -> anyhow::Result<String> {
		self.prompt_password(message).await
	}

	fn display_special_ms_auth(&mut self, url: &str, code: &str) {
		self.display_text("Showing auth info".into(), MessageLevel::Important);
		let _ = self.app.emit_all(
			"mcvm_display_auth_info",
			AuthDisplayEvent {
				url: url.to_owned(),
				device_code: code.to_owned(),
			},
		);
	}
}

impl LauncherOutput {
	fn disp(&mut self, text: String) {
		println!("{text}");
		let _ = self.app.emit_all("mcvm_output_message", MessageEvent(text));
	}
}

/// Event for a message
#[derive(Clone, Serialize)]
pub struct MessageEvent(String);

/// Event for an associated progressbar
#[derive(Clone, Serialize)]
pub struct AssociatedProgressEvent {
	pub current: u32,
	pub total: u32,
	pub message: String,
}

/// Event for the auth display
#[derive(Clone, Serialize)]
pub struct AuthDisplayEvent {
	url: String,
	device_code: String,
}

/// Event for a yes-no prompt
#[derive(Clone, Serialize)]
pub struct YesNoPromptEvent {
	default: bool,
	message: String,
}
