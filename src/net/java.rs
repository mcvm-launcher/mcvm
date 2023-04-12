use crate::net::download::download_json;
use crate::util::json::{self, JsonType};
use crate::util::{ARCH_STRING, OS_STRING};

use anyhow::{anyhow, Context};

pub mod adoptium {
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
		let manifest = download_json::<serde_json::Value>(&url)
			.await
			.context("Failed to download manifest of Adoptium versions")?;
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
