use std::{fmt::Display, sync::Arc};

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::net::game_files::version_manifest::VersionManifest;

/// Matches for the latest Minecraft version.
/// We have to separate this so that deserialization works
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub enum MinecraftLatestVersion {
	#[serde(rename = "latest")]
	/// A release version of Minecraft
	Release,
	#[serde(rename = "latest_snapshot")]
	/// A snapshot version of Minecraft
	Snapshot,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
/// Used for deserializing a Minecraft version
pub enum MinecraftVersionDeser {
	/// One of the latest version matchers
	Latest(MinecraftLatestVersion),
	/// A generic version
	Version(VersionName),
}

impl MinecraftVersionDeser {
	/// Convert to a Minecraft version
	pub fn to_mc_version(&self) -> MinecraftVersion {
		match self {
			Self::Version(version) => MinecraftVersion::Version(version.clone()),
			Self::Latest(MinecraftLatestVersion::Release) => MinecraftVersion::Latest,
			Self::Latest(MinecraftLatestVersion::Snapshot) => MinecraftVersion::LatestSnapshot,
		}
	}
}

/// User-supplied Minecraft version pattern
#[derive(Debug, Clone)]
pub enum MinecraftVersion {
	/// A generic version
	Version(VersionName),
	/// The latest release version available
	Latest,
	/// The latest release or development version available
	LatestSnapshot,
}

impl MinecraftVersion {
	/// Get the correct version from the version manifest
	pub fn get_version(&self, manifest: &VersionManifest) -> anyhow::Result<VersionName> {
		match self {
			Self::Version(version) => Ok(version.clone()),
			Self::Latest => Ok(manifest.latest.release.clone()),
			Self::LatestSnapshot => Ok(manifest.latest.snapshot.clone()),
		}
	}

	/// Gets the serialized version of this Minecraft version
	pub fn to_serialized(self) -> MinecraftVersionDeser {
		match self {
			Self::Latest => MinecraftVersionDeser::Latest(MinecraftLatestVersion::Release),
			Self::LatestSnapshot => MinecraftVersionDeser::Latest(MinecraftLatestVersion::Snapshot),
			Self::Version(version) => MinecraftVersionDeser::Version(version),
		}
	}
}

impl Display for MinecraftVersion {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Version(version) => version,
				Self::Latest => "Latest",
				Self::LatestSnapshot => "Latest Snaphot",
			}
		)
	}
}

/// String name for a Minecraft version
pub type VersionName = Arc<str>;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_minecraft_version_deserialization() {
		#[derive(Deserialize)]
		struct Test {
			version: MinecraftVersionDeser,
		}

		assert_eq!(
			serde_json::from_str::<Test>(r#"{"version": "1.19"}"#)
				.unwrap()
				.version,
			MinecraftVersionDeser::Version("1.19".into())
		);

		assert_eq!(
			serde_json::from_str::<Test>(r#"{"version": "latest"}"#)
				.unwrap()
				.version,
			MinecraftVersionDeser::Latest(MinecraftLatestVersion::Release)
		);

		assert_eq!(
			serde_json::from_str::<Test>(r#"{"version": "latest_snapshot"}"#)
				.unwrap()
				.version,
			MinecraftVersionDeser::Latest(MinecraftLatestVersion::Snapshot)
		);
	}
}
