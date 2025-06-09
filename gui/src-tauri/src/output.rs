use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::Context;
use mcvm::shared::{
	id::InstanceID,
	lang::translate::TranslationKey,
	output::{MCVMOutput, Message, MessageContents, MessageLevel},
};
use serde::Serialize;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

use crate::{commands::launch::UpdateRunStateEvent, RunState};

/// Response to a prompt in the frontend, shared with a mutex
pub type PromptResponse = Arc<Mutex<Option<String>>>;

pub struct LauncherOutput {
	inner: OutputInner,
	/// The task that this output is running
	task: Option<String>,
	/// The instance launch associated with this specific output
	instance: Option<InstanceID>,
}

impl LauncherOutput {
	pub fn new(inner: &OutputInner) -> Self {
		Self {
			inner: inner.clone(),
			task: None,
			instance: None,
		}
	}

	pub fn set_task(&mut self, task: &str) {
		let _ = self.inner.app.emit_all("mcvm_output_create_task", task);
		self.task = Some(task.to_string());
	}

	pub fn get_app_handle(self) -> Arc<AppHandle> {
		self.inner.app.clone()
	}

	pub fn set_instance(&mut self, instance: InstanceID) {
		self.instance = Some(instance);
	}

	pub fn finish_task(&self) {
		if let Some(task) = &self.task {
			let _ = self.inner.app.emit_all("mcvm_output_finish_task", task);
		}
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
					let _ = self.inner.app.emit_all(
						"mcvm_output_progress",
						AssociatedProgressEvent {
							current,
							total,
							message: msg.default_format(),
							task: self.task.clone(),
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
				let _ = self.inner.app.emit_all(
					"mcvm_output_header",
					MessageEvent {
						message: text,
						ty: MessageType::Header,
						task: self.task.clone(),
					},
				);
			}
			msg => self.disp(msg.default_format()),
		}
	}

	async fn prompt_special_user_passkey(
		&mut self,
		message: MessageContents,
		user_id: &str,
	) -> anyhow::Result<String> {
		{
			let passkeys = self.inner.passkeys.lock().await;
			if let Some(existing) = passkeys.get(user_id) {
				return Ok(existing.clone());
			}
		}

		let result = self.prompt_password(message).await?;
		let mut passkeys = self.inner.passkeys.lock().await;
		passkeys.insert(user_id.into(), result.clone());
		Ok(result)
	}

	async fn prompt_password(&mut self, message: MessageContents) -> anyhow::Result<String> {
		println!("Starting password prompt");
		self.inner
			.app
			.emit_all("mcvm_display_password_prompt", message.default_format())
			.context("Failed to display password prompt to user")?;

		// Block this thread, checking every interval if the prompt has been filled
		// Weird lint
		#[allow(unused_assignments)]
		let mut result = None;
		loop {
			if let Some(answer) = self.inner.password_prompt.lock().await.take() {
				result = Some(answer);
				break;
			}
			tokio::time::sleep(Duration::from_millis(50)).await;
		}

		Ok(result.unwrap())
	}

	async fn prompt_new_password(&mut self, message: MessageContents) -> anyhow::Result<String> {
		self.prompt_password(message).await
	}

	fn display_special_ms_auth(&mut self, url: &str, code: &str) {
		self.display_text("Showing auth info".into(), MessageLevel::Important);
		let _ = self.inner.app.emit_all(
			"mcvm_display_auth_info",
			AuthDisplayEvent {
				url: url.to_owned(),
				device_code: code.to_owned(),
			},
		);
	}

	fn translate(&self, key: TranslationKey) -> &str {
		// Emit an event for certain keys as they notify us of progress in the launch
		if let TranslationKey::PreparingLaunch = key {
			if let Some(instance) = &self.instance {
				let _ = self.inner.app.emit_all(
					"update_run_state",
					UpdateRunStateEvent {
						instance: instance.to_string(),
						state: RunState::Preparing,
					},
				);
			}
		}
		if let TranslationKey::AuthenticationSuccessful = key {
			let _ = self.inner.app.emit_all("mcvm_close_auth_info", ());
		}
		if let TranslationKey::Launch = key {
			if let Some(instance) = &self.instance {
				let _ = self.inner.app.emit_all(
					"update_run_state",
					UpdateRunStateEvent {
						instance: instance.to_string(),
						state: RunState::Running,
					},
				);
			}
		}

		key.get_default()
	}

	fn start_process(&mut self) {
		let _ = self.inner.app.emit_all("mcvm_output_start_process", ());
	}

	fn end_process(&mut self) {
		let _ = self.inner.app.emit_all("mcvm_output_end_process", ());
	}

	fn start_section(&mut self) {
		let _ = self.inner.app.emit_all("mcvm_output_start_section", ());
	}

	fn end_section(&mut self) {
		let _ = self.inner.app.emit_all("mcvm_output_end_section", ());
	}
}

impl LauncherOutput {
	fn disp(&mut self, text: String) {
		println!("{text}");
		let _ = self.inner.app.emit_all(
			"mcvm_output_message",
			MessageEvent {
				message: text,
				ty: MessageType::Simple,
				task: self.task.clone(),
			},
		);
	}
}

impl Drop for LauncherOutput {
	fn drop(&mut self) {
		self.finish_task();
	}
}

#[derive(Clone)]
pub struct OutputInner {
	pub app: Arc<AppHandle>,
	pub password_prompt: PromptResponse,
	pub passkeys: Arc<Mutex<HashMap<String, String>>>,
}

/// Event for a simple text message
#[derive(Clone, Serialize)]
pub struct MessageEvent {
	pub message: String,
	#[serde(rename = "type")]
	pub ty: MessageType,
	pub task: Option<String>,
}

/// Event for an associated progressbar
#[derive(Clone, Serialize)]
pub struct AssociatedProgressEvent {
	pub current: u32,
	pub total: u32,
	pub message: String,
	pub task: Option<String>,
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

#[derive(Clone, Serialize, Copy)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
	Simple,
	Header,
}
