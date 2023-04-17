use std::fmt::Display;

use serde::Deserialize;

use super::json;

/// Matches for the latest Minecraft version.
/// We have to separate this so that deserialization works
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum MinecraftLatestVersion {
	#[serde(rename = "latest")]
	Release,
	#[serde(rename = "latest_snapshot")]
	Snapshot,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
/// Used for deserializing a Minecraft version
pub enum MinecraftVersionDeser {
	Latest(MinecraftLatestVersion),
	Version(String),
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
	Version(String),
	Latest,
	LatestSnapshot,
}

impl MinecraftVersion {
	/// Get the correct version from the version manifest
	pub fn get_version(&self, manifest: &json::JsonObject) -> anyhow::Result<String> {
		match self {
			Self::Version(version) => Ok(version.clone()),
			Self::Latest => {
				let latest = json::access_object(manifest, "latest")?;
				Ok(String::from(json::access_str(latest, "release")?))
			}
			Self::LatestSnapshot => {
				let latest = json::access_object(manifest, "latest")?;
				Ok(String::from(json::access_str(latest, "snapshot")?))
			}
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
			MinecraftVersionDeser::Version(String::from("1.19"))
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
