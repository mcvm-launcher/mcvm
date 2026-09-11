use std::collections::HashMap;

use nitrolaunch::shared::output::MessageContents;

use crate::{
	components::misc::progress_bar,
	data::LauncherData,
	ops::task::{KillTask, Task},
	prelude::*,
	state::BackEvent,
};

#[derive(PartialEq)]
pub struct OutputIndicator;

impl Component for OutputIndicator {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		let back_state = use_consume::<BackState>();
		let kill_task = use_mutation(Mutation::new(KillTask::new(back_state.clone())));

		let mut tasks = use_state::<HashMap<Task, TaskData>>(HashMap::new);
		let mut is_open = use_state(|| false);

		use_side_effect(move || {
			if tasks.read().is_empty() {
				is_open.set(false);
			}
		});

		let front_state2 = front_state.clone();
		let back_state2 = back_state.clone();
		use_future(move || {
			let front_state2 = front_state2.clone();
			let back_state = back_state2.clone();
			async move {
				let mut event_rx = front_state2.read().subscribe_events();
				while let Ok(ev) = event_rx.recv().await {
					match ev {
						BackEvent::OutputStartTask(task) => {
							if !tasks.read().contains_key(&task) {
								tasks.write().insert(task, TaskData::new());
							}
						}
						BackEvent::OutputEndTask { task, .. } => {
							tasks.write().remove(&task);
						}
						BackEvent::OutputMessage {
							message,
							task: Some(task),
						} => {
							if let Some(task) = tasks.write().get_mut(&task) {
								match message {
									MessageContents::StartProcess(msg) => task.process = Some(msg),
									MessageContents::Success(..) => task.process = None,
									MessageContents::Header(header) => task.section = Some(header),
									other => task.messages.push(other),
								}
							}
						}
						BackEvent::OutputEndProcess(Some(task)) => {
							if let Some(task) = tasks.write().get_mut(&task) {
								task.process = None;
							}
						}
						BackEvent::OutputEndSection(Some(task)) => {
							if let Some(task) = tasks.write().get_mut(&task) {
								task.section = None;
							}
						}
						BackEvent::OutputResolutionError { error, instance_id } => {
							front_state2.write().toast(Toast::error(
								"Package resolution failed",
								Some(format!("Instance {instance_id} failed to resolve packages. Check the instance for more information.").into_element()),
							));

							if let Ok(mut data) = LauncherData::open(&back_state.paths) {
								data.last_resolution_errors
									.insert(instance_id, (*error).clone());
								let _ = data.write(&back_state.paths);
								front_state2.write().invalidate(FrontChannel::Data);
							}
						}
						_ => {}
					}
				}
			}
		});

		let temp = tasks.read();
		let current_task = temp
			.iter()
			.max_by_key(|x| x.1.messages.len())
			.map(|x| x.0.clone());

		let indicator_text = match tasks.read().len() {
			0 => "No tasks running".into(),
			1 => current_task.as_ref().unwrap().to_string(),
			other => format!("{other} tasks running"),
		};

		let current_task2 = current_task.clone();
		let indicator = rect()
			.width(Size::fill())
			.height(Size::px(36.0))
			.horizontal()
			.center()
			.spacing(theme.gap)
			.panel_colorway(&theme, false, !tasks.read().is_empty())
			.background(theme.bg)
			.corner_radius(theme.round)
			.on_press(move |_| {
				if current_task2.is_some() {
					is_open.toggle();
				}
			})
			.maybe(current_task.is_some(), |this| {
				this.child(CircularLoader::new().size(24.0))
			})
			.child(indicator_text);

		let popout = if *is_open.read() {
			let current_task2 = current_task.clone();
			let can_cancel = current_task.as_ref().is_some_and(|x| x.can_cancel());
			let task = current_task
				.as_ref()
				.and_then(|x| tasks.read().get(x).cloned());
			let current_message = task
				.as_ref()
				.and_then(|x| x.messages.last())
				.map(|x| format_message(x, &theme))
				.unwrap_or("Running".into_element());
			let indicator = if can_cancel {
				icon("delete", 16.0).into_element()
			} else {
				CircularLoader::new().size(24.0).into_element()
			};

			let actual = rect()
				.height(Size::fill())
				.horizontal()
				.cross_align(Alignment::Center)
				.spacing(theme.gap)
				.padding(theme.gap2)
				.panel_colorway(&theme, false, true)
				.corner_radius(theme.round)
				.overlay_shadow()
				.maybe(can_cancel, |this| this.tip(&front_state, "Click to cancel"))
				.on_press(move |_| kill_task.mutate(current_task2.clone().unwrap()))
				.child(indicator)
				.child(current_message);

			Some(
				rect()
					.width(Size::fill())
					.height(Size::px(36.0))
					.margin((0.0, 0.0, theme.gap2, 0.0))
					.center()
					.child(actual),
			)
		} else {
			None
		};

		Attached::new(indicator).top().maybe_child(popout)
	}
}

#[derive(Clone)]
struct TaskData {
	messages: Vec<MessageContents>,
	process: Option<String>,
	section: Option<String>,
}

impl TaskData {
	fn new() -> Self {
		Self {
			messages: Vec::new(),
			process: None,
			section: None,
		}
	}
}

fn format_message(message: &MessageContents, theme: &Theme) -> Element {
	match message {
		MessageContents::Simple(text) => label().text(text.clone()).max_lines(1).into_element(),
		MessageContents::Warning(text) => label()
			.text(text.clone())
			.max_lines(1)
			.color(theme.warning)
			.into_element(),
		MessageContents::Error(text) => {
			label().text(text.clone()).color(theme.error).into_element()
		}
		MessageContents::Success(text) => label()
			.text(text.clone())
			.max_lines(1)
			.color(theme.success)
			.into_element(),
		MessageContents::Header(text) => label()
			.text(text.clone())
			.max_lines(1)
			.font_weight(FontWeight::BOLD)
			.into_element(),
		MessageContents::Progress { current, total } => rect()
			.width(Size::px(240.0))
			.child(progress_bar(theme, *current as f32 / *total as f32))
			.into_element(),
		MessageContents::Associated(msg1, msg2) => rect()
			.horizontal()
			.center()
			.spacing(theme.gap)
			.child(format_message(msg1, theme))
			.child(format_message(msg2, theme))
			.into_element(),
		other => other.clone().default_format().into_element(),
	}
}
