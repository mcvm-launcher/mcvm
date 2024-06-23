use crate::data::InstanceIcon;
use crate::{output::LauncherOutput, State};
use anyhow::Context;
use itertools::Itertools;
use mcvm::config::{plugin::PluginManager, Config};
use mcvm::instance::launch::LaunchSettings;
use mcvm::io::paths::Paths;
use mcvm::shared::id::InstanceID;
use mcvm::shared::Side;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use tauri::Manager;
use tokio::task::JoinHandle;

#[tauri::command]
pub async fn launch_game(
	app_handle: tauri::AppHandle,
	mut state: tauri::State<'_, State>,
	instance_id: String,
	offline: bool,
) -> Result<(), String> {
	let output = LauncherOutput::new(
		app_handle,
		state.passkeys.clone(),
		state.password_prompt.clone(),
	);

	// Make sure the game is stopped first
	stop_game_impl(&mut state).await?;

	let launched_game = fmt_err(
		get_launched_game(instance_id, offline, &mut state, output)
			.await
			.context("Failed to launch game"),
	)?;
	let mut lock = state.launched_game.lock().await;
	*lock = Some(launched_game);

	Ok(())
}

async fn get_launched_game(
	instance_id: String,
	offline: bool,
	state: &mut tauri::State<'_, State>,
	mut o: LauncherOutput,
) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
	println!("Launching game!");

	let mut config = load_config(&state.paths, &mut o).context("Failed to load config")?;

	let paths = state.paths.clone();
	// let mut users = state.user_manager.lock().await.clone();
	let plugins = config.plugins.clone();

	let task_handle = tokio::spawn(async move {
		let mut o = o;
		let instance_id = InstanceID::from(instance_id);
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
		app.emit_all("game_finished", ())?;

		Ok::<(), anyhow::Error>(())
	});

	Ok(task_handle)
}

#[tauri::command]
pub async fn stop_game(mut state: tauri::State<'_, State>) -> Result<(), String> {
	println!("Stopping game...");
	stop_game_impl(&mut state).await?;

	Ok(())
}

async fn stop_game_impl(state: &mut tauri::State<'_, State>) -> Result<(), String> {
	let mut lock = state.launched_game.lock().await;
	lock.as_mut().map(|game| game.abort());
	lock.take();

	Ok(())
}

#[tauri::command]
pub async fn get_instances(
	state: tauri::State<'_, State>,
	app_handle: tauri::AppHandle,
) -> Result<Vec<InstanceInfo>, String> {
	let mut output = LauncherOutput::new(
		app_handle,
		state.passkeys.clone(),
		state.password_prompt.clone(),
	);
	let config = fmt_err(load_config(&state.paths, &mut output).context("Failed to load config"))?;

	let data = state.data.lock().await;

	let instances = config
		.instances
		.iter()
		.sorted_by_key(|x| x.0)
		.map(|(id, instance)| {
			let id = id.to_string();
			InstanceInfo {
				icon: data.instance_icons.get(&id).cloned(),
				pinned: data.pinned.contains(&id),
				id,
				name: instance.get_config().name.clone(),
				side: instance.get_side(),
			}
		})
		.collect();

	Ok(instances)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstanceInfo {
	pub id: String,
	pub name: Option<String>,
	pub side: Side,
	pub icon: Option<InstanceIcon>,
	pub pinned: bool,
}

#[tauri::command]
pub async fn pin_instance(
	state: tauri::State<'_, State>,
	instance_id: String,
	pin: bool,
) -> Result<(), String> {
	let mut data = state.data.lock().await;
	if pin {
		data.pinned.insert(instance_id);
	} else {
		data.pinned.remove(&instance_id);
	}
	fmt_err(data.write(&state.paths).context("Failed to write data"))?;

	Ok(())
}

#[tauri::command]
pub async fn get_instance_groups(
	state: tauri::State<'_, State>,
	app_handle: tauri::AppHandle,
) -> Result<Vec<InstanceGroupInfo>, String> {
	let mut output = LauncherOutput::new(
		app_handle,
		state.passkeys.clone(),
		state.password_prompt.clone(),
	);
	let config = fmt_err(load_config(&state.paths, &mut output).context("Failed to load config"))?;

	let groups = config
		.instance_groups
		.iter()
		.sorted_by_key(|x| x.0)
		.map(|(id, instances)| InstanceGroupInfo {
			id: id.to_string(),
			contents: instances.iter().map(ToString::to_string).collect(),
		})
		.collect();

	Ok(groups)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstanceGroupInfo {
	pub id: String,
	pub contents: Vec<String>,
}

#[tauri::command]
pub async fn answer_password_prompt(
	state: tauri::State<'_, State>,
	answer: String,
) -> Result<(), String> {
	*state.password_prompt.lock().await = Some(answer);

	Ok(())
}

fn load_config(paths: &Paths, o: &mut LauncherOutput) -> anyhow::Result<Config> {
	let plugins = PluginManager::load(paths, o).context("Failed to load plugin manager")?;
	Config::load(&Config::get_path(paths), plugins, true, paths, o).context("Failed to load config")
}

/// Error formatting for results
fn fmt_err<T, E: Debug>(r: Result<T, E>) -> Result<T, String> {
	r.map_err(|x| format!("{x:?}"))
}
