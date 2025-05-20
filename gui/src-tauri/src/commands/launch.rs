use crate::{output::LauncherOutput, State};
use crate::{RunState, RunningInstance};
use anyhow::Context;
use itertools::Itertools;
use mcvm::instance::launch::LaunchSettings;
use mcvm::shared::id::InstanceID;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::Manager;
use tokio::task::JoinHandle;

use super::instance::InstanceInfo;
use super::{fmt_err, load_config};

#[tauri::command]
pub async fn launch_game(
	app_handle: tauri::AppHandle,
	state: tauri::State<'_, State>,
	instance_id: String,
	offline: bool,
	user: Option<&str>,
) -> Result<(), String> {
	let app_handle = Arc::new(app_handle);
	let state = Arc::new(state);
	let output = LauncherOutput::new(
		app_handle.clone(),
		state.passkeys.clone(),
		state.password_prompt.clone(),
	);

	let instance_id = InstanceID::from(instance_id);

	// Make sure the game is stopped first
	stop_game_impl(&state, &instance_id).await?;

	let launched_game = fmt_err(
		get_launched_game(
			instance_id.to_string(),
			offline,
			user,
			state.clone(),
			output,
		)
		.await
		.context("Failed to launch game"),
	)?;
	let mut lock = state.launched_games.lock().await;
	let running_instance = RunningInstance {
		id: instance_id.clone(),
		task: launched_game,
		state: RunState::NotStarted,
	};
	lock.insert(instance_id, running_instance);

	Ok(())
}

async fn get_launched_game(
	instance_id: String,
	offline: bool,
	user: Option<&str>,
	state: Arc<tauri::State<'_, State>>,
	mut o: LauncherOutput,
) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
	println!("Launching game!");

	let mut config = load_config(&state.paths, &mut o).context("Failed to load config")?;
	if let Some(user) = user {
		config.users.choose_user(user)?;
	}

	let paths = state.paths.clone();
	let plugins = config.plugins.clone();
	let instance_id = InstanceID::from(instance_id);
	o.set_instance(instance_id.clone());

	let launch_task = {
		let instance_id = instance_id.clone();
		tokio::spawn(async move {
			let mut o = o;

			let instance = config
				.instances
				.get_mut(&instance_id)
				.context("Instance does not exist")?;
			let settings = LaunchSettings {
				ms_client_id: crate::get_ms_client_id(),
				offline_auth: offline,
			};
			let handle = instance
				.launch(&paths, &mut config.users, &plugins, settings, &mut o)
				.await
				.context("Failed to launch instance")?;

			handle
				.wait(&plugins, &paths, &mut o)
				.context("Failed to wait for instance to finish")?;

			println!("Game closed");
			let app = o.get_app_handle();
			app.emit_all("game_finished", instance_id.to_string())?;

			Ok::<(), anyhow::Error>(())
		})
	};

	Ok(launch_task)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UpdateRunStateEvent {
	pub instance: String,
	pub state: RunState,
}

#[tauri::command]
pub async fn stop_game(mut state: tauri::State<'_, State>, instance: String) -> Result<(), String> {
	println!("Stopping game...");
	stop_game_impl(&mut state, &instance.into()).await?;

	Ok(())
}

async fn stop_game_impl(
	state: &tauri::State<'_, State>,
	instance: &InstanceID,
) -> Result<(), String> {
	let mut lock = state.launched_games.lock().await;
	if let Some(instance) = lock.get_mut(instance) {
		instance.task.abort();
	}
	lock.remove(instance);

	Ok(())
}

#[tauri::command]
pub async fn get_running_instances(
	state: tauri::State<'_, State>,
	app_handle: tauri::AppHandle,
) -> Result<Vec<RunningInstanceInfo>, String> {
	let app_handle = Arc::new(app_handle);

	let mut output = LauncherOutput::new(
		app_handle,
		state.passkeys.clone(),
		state.password_prompt.clone(),
	);
	let config = fmt_err(load_config(&state.paths, &mut output).context("Failed to load config"))?;

	let data = state.data.lock().await;
	let launched_games = state.launched_games.lock().await;

	let instances = launched_games
		.iter()
		.sorted_by_key(|x| x.0)
		.filter_map(|(id, instance)| {
			let configured_instance = config.instances.get(id);
			let Some(configured_instance) = configured_instance else {
				return None;
			};
			let id = id.to_string();
			Some(RunningInstanceInfo {
				info: InstanceInfo {
					icon: data.instance_icons.get(&id).cloned(),
					pinned: data.pinned.contains(&id),
					id,
					name: configured_instance.get_config().name.clone(),
					side: Some(configured_instance.get_side()),
				},
				state: instance.state,
			})
		})
		.collect();

	Ok(instances)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RunningInstanceInfo {
	pub info: InstanceInfo,
	pub state: RunState,
}

#[tauri::command]
pub async fn set_running_instance_state(
	state: tauri::State<'_, State>,
	instance: String,
	run_state: RunState,
) -> Result<(), String> {
	if let Some(instance) = state
		.launched_games
		.lock()
		.await
		.get_mut(&InstanceID::from(instance))
	{
		instance.state = run_state;
	}

	Ok(())
}

#[tauri::command]
pub async fn answer_password_prompt(
	state: tauri::State<'_, State>,
	answer: String,
) -> Result<(), String> {
	*state.password_prompt.lock().await = Some(answer);

	Ok(())
}
