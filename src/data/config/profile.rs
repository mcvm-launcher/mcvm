use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

use crate::{
	data::{
		addon::{Modloader, PluginLoader},
		profile::Profile,
	},
	util::versions::MinecraftVersionDeser,
};

use super::{instance::InstanceConfig, package::PackageConfig};

#[derive(Deserialize)]
pub struct ProfileConfig {
	version: MinecraftVersionDeser,
	#[serde(default)]
	pub modloader: Modloader,
	#[serde(default)]
	pub plugin_loader: PluginLoader,
	pub instances: HashMap<String, InstanceConfig>,
	#[serde(default)]
	pub packages: Vec<PackageConfig>,
}

impl ProfileConfig {
	pub fn to_profile(&self, profile_id: &str) -> Profile {
		Profile::new(
			profile_id,
			self.version.to_mc_version(),
			self.modloader.clone(),
			self.plugin_loader.clone(),
		)
	}
}

pub fn parse_profile_config(val: &Value) -> anyhow::Result<ProfileConfig> {
	let out = serde_json::from_value::<ProfileConfig>(val.clone())?;
	Ok(out)
}
