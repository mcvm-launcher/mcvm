#![warn(missing_docs)]

//! This crate contains serde structs for MCVM configuration. It does not provide
//! any functionality to actually read the config correctly, just to create it.

use std::{collections::HashMap, sync::Arc};

use instance::InstanceConfig;
use mcvm_shared::id::{InstanceID, ProfileID};
use preferences::PrefDeser;
use profile::ProfileConfig;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use user::UserConfig;

/// Instance configuration
pub mod instance;
/// Package configuration
pub mod package;
/// Global preferences configuration
pub mod preferences;
/// Profile configuration
pub mod profile;
/// User configuration
pub mod user;

/// Deserialization struct for user configuration
#[derive(Deserialize, Serialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct ConfigDeser {
	/// The list of configured users
	pub users: HashMap<String, UserConfig>,
	/// The currently selected user
	pub default_user: Option<String>,
	/// The list of configured instances
	pub instances: HashMap<InstanceID, InstanceConfig>,
	/// The list of configured instance groups
	pub instance_groups: HashMap<Arc<str>, Vec<InstanceID>>,
	/// The list of configured profiles
	pub profiles: HashMap<ProfileID, ProfileConfig>,
	/// The global profile
	pub global_profile: Option<ProfileConfig>,
	/// The global preferences
	pub preferences: PrefDeser,
}
