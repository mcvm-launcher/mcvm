use std::collections::HashSet;
use std::{collections::HashMap, path::PathBuf};

use mcvm::core::io::{json_from_file, json_to_file};
use mcvm::io::paths::Paths;
use serde::{Deserialize, Serialize};

/// Stored launcher data
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct LauncherData {
	/// Icons for instances
	pub instance_icons: HashMap<String, InstanceIcon>,
	/// Icons for profiles
	pub profile_icons: HashMap<String, InstanceIcon>,
	/// Set of pinned instances
	pub pinned: HashSet<String>,
	/// The currently selected user
	pub current_user: Option<String>,
}

impl LauncherData {
	/// Open the launcher data
	pub fn open(paths: &Paths) -> anyhow::Result<Self> {
		let path = Self::path(paths);
		if path.exists() {
			json_from_file(path)
		} else {
			Ok(Self::default())
		}
	}

	/// Write the launcher data
	pub fn write(&self, paths: &Paths) -> anyhow::Result<()> {
		json_to_file(Self::path(paths), &self)
	}

	/// Get the path to the launcher file
	pub fn path(paths: &Paths) -> PathBuf {
		paths.internal.join("launcher_data.json")
	}
}

/// Different icons for instances
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum InstanceIcon {
	/// A custom user icon at a path
	File(PathBuf),
}
