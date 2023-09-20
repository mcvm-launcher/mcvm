use anyhow::{bail, Context};
use mcvm_shared::util::DeserListOrSingle;
use reqwest::Client;
use serde::Deserialize;

use crate::data::profile::update::manager::UpdateManager;
use crate::io::files::{self, paths::Paths};
use crate::io::java::JavaMajorVersion;
use crate::net::download;

use super::version_manifest::VersionManifest;

/// The client metadata, which is used for information about
/// how to set up and launch the client for every version of the game.
#[derive(Deserialize, Debug, Clone)]
pub struct ClientMeta {
	/// Arguments for the client. Can have a different field name and format
	/// depending on how new the file is in the manifest
	#[serde(alias = "minecraftArguments")]
	pub arguments: args::Arguments,
	/// Info about the assets index
	#[serde(rename = "assetIndex")]
	pub asset_index: AssetIndexInfo,
	/// Assets version
	#[serde(rename = "assets")]
	pub assets_version: String,
	/// File downloads
	pub downloads: Downloads,
	/// Java information
	#[serde(rename = "javaVersion")]
	pub java_info: JavaInfo,
	/// Libraries to download for the client
	pub libraries: Vec<libraries::Library>,
	/// Java main class for the client
	#[serde(rename = "mainClass")]
	pub main_class: String,
	/// Logging information
	pub logging: LogInfo,
}

/// Information in the meta about the assets index
#[derive(Deserialize, Debug, Clone)]
pub struct AssetIndexInfo {
	/// The URL to the assets index for this version
	pub url: String,
}

/// Download information for different files
#[derive(Deserialize, Debug, Clone)]
pub struct Downloads {
	/// Download info for the client.jar
	pub client: DownloadInfo,
	/// Download info for the server.jar
	pub server: DownloadInfo,
}

/// Information for the downloading of a specific file
#[derive(Deserialize, Debug, Clone)]
pub struct DownloadInfo {
	/// The URL to the file
	pub url: String,
}

/// Information about Java for this version
#[derive(Deserialize, Debug, Clone)]
pub struct JavaInfo {
	/// The Java major version to use
	#[serde(rename = "majorVersion")]
	pub major_version: JavaMajorVersion,
}

/// Information about logging for this version
#[derive(Deserialize, Debug, Clone)]
pub struct LogInfo {
	/// Client logging
	pub client: ClientLogInfo,
}

/// Information about logging for the client
#[derive(Deserialize, Debug, Clone)]
pub struct ClientLogInfo {
	/// The JVM argument to use for specifying the path of the log.
	/// It contains a token '${file}' that should be replaced with the path
	/// to the file.
	pub argument: String,
	/// Download for the logging configuration file
	pub file: DownloadInfo,
}

/// Game arguments in the client meta
pub mod args {
	use super::*;
	/// The old and new formats for the game argument list
	#[derive(Deserialize, Debug, Clone)]
	#[serde(untagged)]
	pub enum Arguments {
		/// The new format with both JVM and game args
		New(NewArguments),
		/// The old format with just a list of game args in a string,
		/// separated by spaces
		Old(String),
	}

	/// Arguments for the game from the client meta, in the new format
	#[derive(Deserialize, Debug, Clone)]
	pub struct NewArguments {
		/// Arguments for the JVM
		pub jvm: Vec<ArgumentItem>,
		/// Arguments for the game
		pub game: Vec<ArgumentItem>,
	}

	/// A new argument item
	#[derive(Deserialize, Debug, Clone)]
	#[serde(untagged)]
	pub enum ArgumentItem {
		/// A simple string argument
		Simple(String),
		/// An argument or set of arguments with a condition
		Conditional(ConditionalArguments),
	}

	/// Complex arguments with conditions
	#[derive(Deserialize, Debug, Clone)]
	pub struct ConditionalArguments {
		/// Rules to check for the arguments to be applied
		pub rules: Vec<conditions::Rule>,
		/// The argument(s) to apply if the conditions succeed
		pub value: DeserListOrSingle<String>,
	}
}

/// Deserialization for libraries
pub mod libraries {
	use std::collections::HashMap;

	use super::*;

	/// A library to install
	#[derive(Deserialize, Debug, Clone)]
	pub struct Library {
		/// Downloads for this library
		#[serde(default)]
		pub downloads: Downloads,
		/// Maven name of this library
		pub name: String,
		/// Natives classifiers
		#[serde(default)]
		pub natives: HashMap<String, String>,
		/// Rules to check for this library to be downloaded
		#[serde(default)]
		pub rules: Vec<conditions::Rule>,
		/// Rules for extraction
		#[serde(default)]
		pub extract: ExtractionRules,
	}

	/// Downloads for a library
	#[derive(Deserialize, Debug, Default, Clone)]
	pub struct Downloads {
		/// Artifact for the main library file
		pub artifact: Option<Artifact>,
		/// Optional artifacts for native libraries, to be extracted.
		/// Referred to by their native classifiers.
		#[serde(rename = "classifiers")]
		#[serde(default)]
		pub native_classifiers: HashMap<String, Artifact>,
	}

	/// A single download artifact
	#[derive(Deserialize, Debug, Clone)]
	pub struct Artifact {
		/// Path to store the artifact in
		pub path: String,
		/// URL to download the artifact from
		pub url: String,
	}

	/// Extraction rules for a library
	#[derive(Deserialize, Debug, Clone, Default)]
	#[serde(default)]
	pub struct ExtractionRules {
		/// Files to exclude from the extraction
		pub exclude: Vec<String>,
	}
}

/// Facilities for conditions in the meta
pub mod conditions {
	use std::fmt::Display;

	use super::*;

	/// A rule condition
	#[derive(Deserialize, Debug, Clone)]
	pub struct Rule {
		/// Action for inverting the rule
		pub action: RuleAction,
		/// Features to check for the condition
		#[serde(default)]
		pub features: RuleFeatures,
		/// OS properties to check for the condition
		#[serde(default)]
		pub os: OSConditions,
	}

	/// Used in argument rules to invert a condition
	#[derive(Deserialize, Debug, Clone)]
	#[serde(rename_all = "snake_case")]
	pub enum RuleAction {
		/// Allow the arguments if the conditions are met
		Allow,
		/// Remove the arguments if the conditions are met
		Disallow,
	}

	impl RuleAction {
		/// Check if this rule is allowed
		pub fn is_allowed(&self) -> bool {
			matches!(&self, Self::Allow)
		}

		/// Check if the allowance of this rule matches a condition.
		/// If the rule is allowed, but the condition fails, then the return is false.
		/// If the rule is not allowed, but the condition succeeds, then the return is also false.
		pub fn is_allowed_with_condition(&self, condition: bool) -> bool {
			self.is_allowed() == condition
		}
	}

	/// Features that can be checked for a conditional argument rule
	#[derive(Deserialize, Debug, Default, Clone)]
	pub struct RuleFeatures {
		/// Feature for if the user is a demo user. Should be checked if present
		pub is_demo_user: Option<bool>,
		/// Feature for if a custom window resolution is set. Should be checked if present
		pub has_custom_resolution: Option<bool>,
		/// Feature for if QuickPlay is enabled. Should be checked if present
		#[serde(alias = "has_quick_plays_support")]
		pub has_quick_play_support: Option<bool>,
		/// Feature for if QuickPlay singleplayer is enabled. Should be checked if present
		pub is_quick_play_singleplayer: Option<bool>,
		/// Feature for if QuickPlay multiplayer is enabled. Should be checked if present
		pub is_quick_play_multiplayer: Option<bool>,
		/// Feature for if QuickPlay Realms is enabled. Should be checked if present
		pub is_quick_play_realms: Option<bool>,
	}

	/// Operating-system related conditions for argument rules
	#[derive(Deserialize, Debug, Default, Clone)]
	pub struct OSConditions {
		/// Condition for the type of OS. Should be checked if present
		pub name: Option<OSName>,
		/// Condition for the target architecture. Should be checked if present
		pub arch: Option<OSArch>,
	}

	/// Operating systems for OS conditions
	#[derive(Deserialize, Debug, Clone)]
	#[serde(rename_all = "snake_case")]
	pub enum OSName {
		/// Windows operating system
		Windows,
		/// MacOS operating system
		#[serde(alias = "osx")]
		MacOS,
		/// Linux operating system
		Linux,
	}

	impl Display for OSName {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(
				f,
				"{}",
				match self {
					Self::Windows => "windows",
					Self::MacOS => "macos",
					Self::Linux => "linux",
				}
			)
		}
	}

	/// Architecture for OS conditions
	#[derive(Deserialize, Debug, Clone)]
	#[serde(rename_all = "snake_case")]
	pub enum OSArch {
		/// x86 architecture
		X86,
		/// x86_64 architecture
		X86_64,
		/// ARM architecture
		Arm,
	}

	impl Display for OSArch {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(
				f,
				"{}",
				match self {
					Self::X86 => "x86",
					Self::X86_64 => "x86_64",
					Self::Arm => "arm",
				}
			)
		}
	}
}

/// Gets the specific client info JSON file for a Minecraft version
pub async fn get(
	version: &str,
	version_manifest: &VersionManifest,
	paths: &Paths,
	manager: &UpdateManager,
	client: &Client,
) -> anyhow::Result<ClientMeta> {
	let version_string = version.to_owned();

	let mut version_url = None;
	for entry in &version_manifest.versions {
		if entry.id == version_string {
			version_url = Some(entry.url.clone());
		}
	}
	if version_url.is_none() {
		bail!("Minecraft version does not exist or was not found in the manifest");
	}

	let client_meta_name: String = version_string.clone() + ".json";
	let version_dir = paths.internal.join("versions").join(version_string);
	files::create_dir_async(&version_dir).await?;
	let path = version_dir.join(client_meta_name);
	let text = if manager.allow_offline && path.exists() {
		tokio::fs::read_to_string(path)
			.await
			.context("Failed to read client meta from file")?
	} else {
		let text = download::text(version_url.expect("Version does not exist"), client)
			.await
			.context("Failed to download client meta")?;
		tokio::fs::write(path, &text)
			.await
			.context("Failed to write client meta to a file")?;

		text
	};

	let version_doc = serde_json::from_str(&text).context("Failed to parse client meta")?;

	Ok(version_doc)
}
