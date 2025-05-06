use crate::data::InstanceIcon;
use crate::{output::LauncherOutput, State};
use anyhow::Context;
use itertools::Itertools;
use mcvm::config_crate::instance::InstanceConfig;
use mcvm::shared::id::InstanceID;
use mcvm::shared::Side;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;

use super::{fmt_err, load_config};

#[tauri::command]
pub async fn get_instances(
	state: tauri::State<'_, State>,
	app_handle: tauri::AppHandle,
) -> Result<Vec<InstanceInfo>, String> {
	let app_handle = Arc::new(app_handle);

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
	let app_handle = Arc::new(app_handle);

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
pub async fn get_instance_config(
	state: tauri::State<'_, State>,
	app_handle: tauri::AppHandle,
	instance: String,
) -> Result<(InstanceConfig, InstanceConfig), String> {
	let app_handle = Arc::new(app_handle);

	let mut output = LauncherOutput::new(
		app_handle,
		state.passkeys.clone(),
		state.password_prompt.clone(),
	);

	let config = fmt_err(load_config(&state.paths, &mut output).context("Failed to load config"))?;

	let Some(instance) = config.instances.get(&InstanceID::from(instance)) else {
		return Err("Instance does not exist".into());
	};

	Ok((
		instance.get_config().original_config.clone(),
		instance.get_config().original_config_with_profiles.clone(),
	))
}
