use mcvm::core::net::download;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Get a Smithed pack from the API
pub async fn get_pack(id: &str) -> anyhow::Result<Pack> {
	let url = format!("{API_URL}/packs/{id}");
	download::json(url, &Client::new()).await
}

/// API URL
const API_URL: &str = "https://api.smithed.dev/v2";

/// A Smithed pack
#[derive(Serialize, Deserialize)]
pub struct Pack {
	pub id: String,
	pub display: PackDisplay,
	pub versions: Vec<PackVersion>,
}

/// Display info for a Smithed pack
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackDisplay {
	pub name: String,
	pub description: String,
	pub icon: String,
	pub hidden: bool,
	pub web_page: Option<String>,
}

/// Version of a pack
#[derive(Serialize, Deserialize)]
pub struct PackVersion {
	pub name: String,
	pub downloads: PackDownloads,
	pub supports: Vec<String>,
	pub dependencies: Vec<PackReference>,
}

/// Downloads for a pack version
#[derive(Serialize, Deserialize)]
pub struct PackDownloads {
	pub datapack: Option<String>,
	pub resourcepack: Option<String>,
}

/// Reference to a pack version
#[derive(Serialize, Deserialize)]
pub struct PackReference {
	pub id: String,
	pub version: String,
}
