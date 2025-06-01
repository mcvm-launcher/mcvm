use mcvm_shared::lang::Language;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Configured user preferences
#[derive(Debug)]
pub struct ConfigPreferences {
	/// The global language
	pub language: Language,
}

/// Deserialization struct for user preferences
#[derive(Deserialize, Serialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct PrefDeser {
	/// The user's configured repositories
	pub repositories: RepositoriesDeser,
	/// The user's configured language
	pub language: Language,
}

/// Deserialization struct for a package repo
#[derive(Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct RepoDeser {
	/// The ID of the repository
	pub id: String,
	/// The URL to the repository, which may not exist
	#[serde(skip_serializing_if = "Option::is_none")]
	pub url: Option<String>,
	/// The Path to the repository, which may not exist
	#[serde(skip_serializing_if = "Option::is_none")]
	pub path: Option<String>,
	/// Whether to disable the repo and not add it to the list
	#[serde(default)]
	pub disable: bool,
}

/// Deserialization struct for all configured package repositories
#[derive(Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct RepositoriesDeser {
	/// The preferred repositories over the default ones
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub preferred: Vec<RepoDeser>,
	/// The backup repositories included after the default ones
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub backup: Vec<RepoDeser>,
	/// Whether to enable the core repository
	pub enable_core: bool,
	/// Whether to enable the std repository
	pub enable_std: bool,
}

impl Default for RepositoriesDeser {
	fn default() -> Self {
		Self {
			preferred: Vec::new(),
			backup: Vec::new(),
			enable_core: true,
			enable_std: true,
		}
	}
}
