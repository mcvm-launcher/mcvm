#![allow(dead_code)]
use std::fs::File;

use anyhow::Context;

use crate::io::files::paths::Paths;

use super::{profile::ProfileConfig, user::UserConfig, Config, ConfigDeser};

pub enum ConfigModification {
	AddUser(String, UserConfig),
	AddProfile(String, ProfileConfig),
}

/// Applies modifications to the config
pub fn apply_modifications(
	config: &mut ConfigDeser,
	modifications: Vec<ConfigModification>,
) -> anyhow::Result<()> {
	for modification in modifications {
		match modification {
			ConfigModification::AddUser(id, user) => {
				config.users.insert(id, user);
			}
			ConfigModification::AddProfile(id, profile) => {
				config.profiles.insert(id, profile);
			}
		};
	}
	Ok(())
}

/// Applies modifications to the config and writes it to the config file
pub fn apply_modifications_and_write(
	config: &mut ConfigDeser,
	modifications: Vec<ConfigModification>,
	paths: &Paths,
) -> anyhow::Result<()> {
	apply_modifications(config, modifications)?;
	let path = Config::get_path(paths);
	let mut file = File::create(path).context("Failed to open config file")?;
	serde_json::to_writer_pretty(&mut file, config)
		.context("Failed to write default configuration")?;

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::data::config::user::UserVariant;

	use std::collections::HashMap;

	#[test]
	fn test_user_add_modification() {
		let mut config = ConfigDeser {
			users: HashMap::new(),
			default_user: None,
			profiles: HashMap::new(),
			instance_presets: HashMap::new(),
			preferences: None,
		};

		let user_config = UserConfig {
			name: String::from("Bob"),
			variant: UserVariant::Unverified {},
		};

		let modifications = vec![ConfigModification::AddUser(
			String::from("bob"),
			user_config,
		)];

		apply_modifications(&mut config, modifications).unwrap();
		assert!(config.users.contains_key(&String::from("bob")));
	}
}
