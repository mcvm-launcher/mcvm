use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::bail;
use nitrolaunch::shared::{
	lang::translate::TranslationKey,
	manual_files::ManualFile,
	output::{Message, MessageContents, MessageLevel, NitroOutput},
	pkg::{ArcPkgReq, PackageDiff, ResolutionError},
};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, broadcast, mpsc};

use crate::{ops::task::Task, state::BackEvent, util::PtrEq};

/// Response to a prompt in the frontend, shared with a mutex
pub type PromptResponse = Arc<Mutex<Option<String>>>;
/// Response to a yes/no prompt in the frontend, shared with a mutex
pub type YesNoPromptResponse = Arc<Mutex<Option<bool>>>;

pub struct LauncherOutput {
	inner: OutputInner,
	/// The task that this output is running
	task: Option<Task>,
	show_toasts: bool,
}

impl LauncherOutput {
	pub fn new(inner: &OutputInner) -> Self {
		Self {
			inner: inner.clone(),
			task: None,
			show_toasts: false,
		}
	}

	pub fn show_toasts(&mut self) {
		self.show_toasts = true;
	}

	pub fn set_task(&mut self, task: Task) {
		let _ = self
			.inner
			.event_tx
			.send(BackEvent::OutputStartTask(task.clone()));
		let _ = self.inner.logger.try_send(Message {
			contents: format!("Starting task {task:?}").into(),
			level: MessageLevel::Debug,
		});
		self.task = Some(task);
	}

	pub fn finish_task(&mut self) {
		if let Some(task) = self.task.take() {
			let _ = self.inner.logger.try_send(Message {
				contents: format!("Finished task {task:?}").into(),
				level: MessageLevel::Debug,
			});
			let _ = self.inner.event_tx.send(BackEvent::OutputEndTask {
				task,
				success: true,
			});
		}
	}
}

#[async_trait::async_trait]
impl NitroOutput for LauncherOutput {
	fn display_text(&mut self, text: String, _level: MessageLevel) {
		self.disp(text);
	}

	fn display_message(&mut self, message: Message) {
		let _ = self.inner.logger.try_send(message.clone());

		if self.show_toasts {
			match &message.contents {
				MessageContents::Error(e) => {
					let message = self
						.task
						.as_ref()
						.map(|x| x.failure_message())
						.unwrap_or_else(|| "Task failed".into());
					let _ = self
						.inner
						.event_tx
						.send(BackEvent::ErrorToast(message, Some(format!("{e:?}"))));
				}
				MessageContents::Success(msg) => {
					let _ = self
						.inner
						.event_tx
						.send(BackEvent::SuccessToast(msg.clone()));
				}
				_ => {}
			}
		}

		let _ = self.inner.event_tx.send(BackEvent::OutputMessage {
			message: message.contents,
			task: self.task.clone(),
		});
	}

	async fn prompt_yes_no(
		&mut self,
		default: bool,
		message: MessageContents,
	) -> anyhow::Result<bool> {
		println!("Starting yes no prompt");
		let _ = default;
		self.inner.yes_no_prompt.lock().await.take();
		let _ = self.inner.event_tx.send(BackEvent::ShowYesNoPrompt {
			message: message.default_format(),
		});

		// Block this thread, checking every interval if the prompt has been filled
		let result = loop {
			if let Some(answer) = self.inner.yes_no_prompt.lock().await.take() {
				break answer;
			}
			tokio::time::sleep(Duration::from_millis(50)).await;
		};

		Ok(result)
	}

	async fn prompt_special_account_passkey(
		&mut self,
		message: MessageContents,
		account_id: &str,
	) -> anyhow::Result<String> {
		{
			let passkeys = self.inner.passkeys.lock().await;
			if let Some(existing) = passkeys.get(account_id) {
				return Ok(existing.clone());
			}
		}

		let result = self.prompt_password(message).await?;
		let mut passkeys = self.inner.passkeys.lock().await;
		passkeys.insert(account_id.into(), result.clone());
		Ok(result)
	}

	async fn prompt_password(&mut self, _: MessageContents) -> anyhow::Result<String> {
		let _ = self.inner.logger.try_send(Message {
			contents: "Prompting for password".into(),
			level: MessageLevel::Debug,
		});
		let _ = self.inner.event_tx.send(BackEvent::ShowPasskeyPrompt);

		// Block this thread, checking every interval if the prompt has been filled
		let result = loop {
			if let Some(answer) = self.inner.password_prompt.lock().await.take() {
				break answer;
			}
			tokio::time::sleep(Duration::from_millis(50)).await;
		};

		Ok(result)
	}

	async fn prompt_new_password(&mut self, message: MessageContents) -> anyhow::Result<String> {
		self.prompt_password(message).await
	}

	async fn prompt_special_package_diffs(
		&mut self,
		diffs: Vec<PackageDiff>,
	) -> anyhow::Result<bool> {
		let _ = self.inner.logger.try_send(Message {
			contents: "Prompting for package diffs".into(),
			level: MessageLevel::Debug,
		});

		let _ = self.inner.event_tx.send(BackEvent::ShowPackageDiffsPrompt {
			diffs: diffs.into_iter().collect(),
		});

		let mut event_rx = self.inner.event_rx.resubscribe();
		while let Ok(ev) = event_rx.recv().await {
			if let BackEvent::ConfirmYesNoPrompt { yes } = ev {
				return Ok(yes);
			}
		}

		Ok(false)
	}

	async fn prompt_special_manual_files(&mut self, files: Vec<ManualFile>) -> anyhow::Result<()> {
		let _ = self.inner.logger.try_send(Message {
			contents: "Prompting for manual files".into(),
			level: MessageLevel::Debug,
		});

		let _ = self.inner.event_tx.send(BackEvent::ShowManualFilesPrompt {
			files: PtrEq(files.into_iter().collect()),
		});

		let mut event_rx = self.inner.event_rx.resubscribe();
		while let Ok(ev) = event_rx.recv().await {
			if let BackEvent::ConfirmYesNoPrompt { yes } = ev {
				if yes {
					return Ok(());
				} else {
					bail!("Manual files prompt was rejected by user");
				}
			}
		}

		bail!("Manual files prompt was closed without a response")
	}

	fn display_special_ms_auth(&mut self, url: &str, code: &str) {
		let _ = self.inner.logger.try_send(Message {
			contents: "Prompting for Microsoft auth".into(),
			level: MessageLevel::Debug,
		});
		let _ = self.inner.event_tx.send(BackEvent::ShowAuthPrompt {
			url: url.into(),
			device_code: code.into(),
		});
	}

	fn display_special_resolution_error(&mut self, error: ResolutionError, instance_id: &str) {
		let _ = self.inner.logger.try_send(Message {
			contents: format!("Package resolution error: {error}").into(),
			level: MessageLevel::Important,
		});
		let _ = self.inner.event_tx.send(BackEvent::OutputResolutionError {
			error: Arc::new(SerializableResolutionError::from_err(error)),
			instance_id: instance_id.to_string(),
		});
	}

	fn translate(&self, key: TranslationKey) -> &str {
		// Emit an event for certain keys as they notify us of progress in the launch
		if let TranslationKey::AuthenticationSuccessful = key {
			let _ = self.inner.event_tx.send(BackEvent::CloseAuthPrompt);
		}

		key.get_default()
	}

	fn end_process(&mut self) {
		let _ = self
			.inner
			.event_tx
			.send(BackEvent::OutputEndProcess(self.task.clone()));
	}

	fn end_section(&mut self) {
		let _ = self
			.inner
			.event_tx
			.send(BackEvent::OutputEndSection(self.task.clone()));
	}

	fn get_lesser_copy(&self) -> Box<dyn NitroOutput + Sync> {
		Box::new(Self::new(&self.inner))
	}
}

impl LauncherOutput {
	fn disp(&mut self, text: String) {
		println!("{text}");
		let _ = self.inner.event_tx.send(BackEvent::OutputMessage {
			message: MessageContents::Simple(text),
			task: self.task.clone(),
		});
	}
}

impl Drop for LauncherOutput {
	fn drop(&mut self) {
		self.finish_task();
	}
}

#[derive(Clone)]
pub struct OutputInner {
	pub event_tx: broadcast::Sender<BackEvent>,
	pub event_rx: Arc<broadcast::Receiver<BackEvent>>,
	pub password_prompt: PromptResponse,
	pub yes_no_prompt: YesNoPromptResponse,
	pub passkeys: Arc<Mutex<HashMap<String, String>>>,
	pub logger: mpsc::Sender<Message>,
}

/// A serializable ResolutionError
#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", content = "data")]
pub enum SerializableResolutionError {
	PackageContext(ArcPkgReq, Box<SerializableResolutionError>),
	FailedToPreload(String),
	FailedToGetProperties(ArcPkgReq, String),
	NoValidVersionsFound(ArcPkgReq, Vec<String>),
	ExtensionNotFulfilled(Option<ArcPkgReq>, ArcPkgReq),
	ExplicitRequireNotFulfilled(ArcPkgReq, ArcPkgReq),
	IncompatiblePackage(ArcPkgReq, Vec<Arc<str>>),
	FailedToEvaluate(ArcPkgReq, String),
	Misc(String),
}

impl SerializableResolutionError {
	pub fn from_err(err: ResolutionError) -> Self {
		match err {
			ResolutionError::PackageContext(req, resolution_error) => {
				SerializableResolutionError::PackageContext(
					req,
					Box::new(SerializableResolutionError::from_err(*resolution_error)),
				)
			}
			ResolutionError::FailedToPreload(error) => {
				SerializableResolutionError::FailedToPreload(error.to_string())
			}
			ResolutionError::FailedToGetProperties(req, error) => {
				SerializableResolutionError::FailedToGetProperties(req, format!("{error:?}"))
			}
			ResolutionError::NoValidVersionsFound(req, constraints) => {
				SerializableResolutionError::NoValidVersionsFound(
					req,
					constraints.into_iter().map(|x| x.to_string()).collect(),
				)
			}
			ResolutionError::ExtensionNotFulfilled(req1, req2) => {
				SerializableResolutionError::ExtensionNotFulfilled(req1, req2)
			}
			ResolutionError::ExplicitRequireNotFulfilled(req1, req2) => {
				SerializableResolutionError::ExplicitRequireNotFulfilled(req1, req2)
			}
			ResolutionError::IncompatiblePackage(req, items) => {
				SerializableResolutionError::IncompatiblePackage(req, items)
			}
			ResolutionError::FailedToEvaluate(req, error) => {
				SerializableResolutionError::FailedToEvaluate(req, format!("{error:?}"))
			}
			ResolutionError::Misc(error) => SerializableResolutionError::Misc(format!("{error:?}")),
		}
	}
}
