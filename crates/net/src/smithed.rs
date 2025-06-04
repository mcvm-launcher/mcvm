use crate::download::{self, user_agent};
use anyhow::Context;
use mcvm_shared::pkg::PackageSearchParameters;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

/// Get a Smithed pack from the API
pub async fn get_pack(id: &str, client: &Client) -> anyhow::Result<Pack> {
	let url = format!("{API_URL}/packs/{id}");
	download::json(url, client).await
}

/// Get a Smithed pack from the API, returning None on 404
pub async fn get_pack_optional(id: &str, client: &Client) -> anyhow::Result<Option<Pack>> {
	let url = format!("{API_URL}/packs/{id}");

	let resp = client
		.get(url)
		.header("User-Agent", user_agent())
		.send()
		.await
		.context("Failed to send request")?;
	if resp.status() == StatusCode::NOT_FOUND {
		return Ok(None);
	}

	let resp = resp
		.error_for_status()
		.context("Server returned an error")?;

	resp.json()
		.await
		.map(Some)
		.context("Failed to deserialize JSON")
}

/// API URL
const API_URL: &str = "https://api.smithed.dev/v2";

/// A Smithed pack
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Pack {
	pub id: String,
	pub display: PackDisplay,
	pub versions: Vec<PackVersion>,
}

/// Display info for a Smithed pack
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackDisplay {
	pub name: String,
	pub description: String,
	pub icon: String,
	pub hidden: bool,
	pub web_page: Option<String>,
}

/// Version of a pack
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PackVersion {
	pub name: String,
	pub downloads: PackDownloads,
	pub supports: Vec<String>,
	pub dependencies: Vec<PackReference>,
}

/// Downloads for a pack version
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PackDownloads {
	pub datapack: Option<String>,
	pub resourcepack: Option<String>,
}

/// Reference to a pack version
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PackReference {
	pub id: String,
	pub version: String,
}

/// Get a Smithed bundle from the API
pub async fn get_bundle(id: &str, client: &Client) -> anyhow::Result<Bundle> {
	let url = format!("{API_URL}/bundles/{id}");
	download::json(url, client).await
}

/// A Smithed bundle
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Bundle {
	pub id: String,
	pub versions: Vec<BundleVersion>,
}

/// Version of a bundle
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BundleVersion {
	pub name: String,
	pub supports: Vec<String>,
	pub packs: Vec<PackReference>,
}

/// Search packs from the Smithed API
pub async fn search_packs(
	params: PackageSearchParameters,
	client: &Client,
) -> anyhow::Result<Vec<PackSearchResult>> {
	let limit = if params.count > 100 {
		100
	} else {
		params.count
	};
	let search = if let Some(search) = params.search {
		format!("&search={search}")
	} else {
		String::new()
	};
	let url = format!("{API_URL}/packs?limit={limit}{search}");

	download::json(url, client).await
}

/// A single pack search result
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PackSearchResult {
	pub id: String,
	#[serde(rename = "displayName")]
	pub display_name: String,
}
