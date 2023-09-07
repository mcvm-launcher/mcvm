#![allow(dead_code)]
use std::fs::File;

use anyhow::{anyhow, Context};

use crate::io::files::paths::Paths;

use super::{
	package::PackageConfig, profile::ProfileConfig, user::UserConfig, Config, ConfigDeser,
};

/// A modification operation that can be applied to the config
pub enum ConfigModification {
	/// Adds a new user
	AddUser(String, UserConfig),
	/// Adds a new profile
	AddProfile(String, ProfileConfig),
	/// Adds a new package to a profile
	AddPackage(String, PackageConfig),
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
			ConfigModification::AddPackage(id, package) => {
				let profile = config
					.profiles
					.get_mut(&id)
					.ok_or(anyhow!("Unknown profile '{id}'"))?;
				profile.packages.push(package);
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
	use crate::data::config::{preferences::PrefDeser, user::UserVariant};

	use std::collections::HashMap;

	#[test]
	fn test_user_add_modification() {
		let mut config = ConfigDeser {
			users: HashMap::new(),
			default_user: None,
			profiles: HashMap::new(),
			instance_presets: HashMap::new(),
			preferences: PrefDeser::default(),
		};

		let user_config = UserConfig {
			name: "Bob".into(),
			variant: UserVariant::Unverified {},
		};

		let modifications = vec![ConfigModification::AddUser("bob".into(), user_config)];

		apply_modifications(&mut config, modifications).unwrap();
		assert!(config.users.contains_key("bob"));
	}
}
