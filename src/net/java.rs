use crate::util::{ARCH_STRING, OS_STRING};
use crate::util::json::{self, JsonType};

use anyhow::{anyhow, Context};

pub mod adoptium {
	use crate::net::download::download_bytes;

use super::*;

	/// Gets the URL to the JSON file for a major Java version
	fn json_url(major_version: &str) -> String {
		format!(
			"https://api.adoptium.net/v3/assets/latest/{}/hotspot?image_type=jre&vendor=eclipse&architecture={}&os={}",
			major_version,
			ARCH_STRING,
			OS_STRING
		)
	}

	/// Gets the newest Adoptium binaries download for a major Java version
	pub async fn get_latest(major_version: &str) -> anyhow::Result<json::JsonObject> {
		let url = json_url(major_version);
		let manifest = serde_json::from_slice::<serde_json::Value>(&download_bytes(&url).await?)?;
		let manifest = json::ensure_type(manifest.as_array(), JsonType::Arr)
			.context("Expected manifest to be an array of versions")?;
		let version = json::ensure_type(
			manifest
				.get(0)
				.ok_or(anyhow!("A valid installation was not found"))?
				.as_object(),
			JsonType::Obj,
		)?;

		Ok(version.to_owned())
	}
}