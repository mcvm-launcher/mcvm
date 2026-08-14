use anyhow::Context;
use nitro_shared::{
	loaders::Loader,
	pkg::{PackageKind, PackageSearchParameters, PackageStability},
};
use reqwest::Client;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::download::user_agent;

/// Requests a sub-url from the CurseForge API
pub async fn request_api<D: DeserializeOwned>(
	url_path: &str,
	api_key: &str,
	client: &Client,
) -> anyhow::Result<D> {
	let resp = client
		.get(String::from("https://api.curseforge.com/") + url_path)
		.header("User-Agent", user_agent())
		.header("x-api-key", api_key)
		.send()
		.await
		.context("Failed to send request")?
		.error_for_status()
		.context("Server reported an error")?;

	Ok(resp.json().await.context("Failed to parse response")?)
}

/// Requests a sub-url from the CurseForge API for text
pub async fn request_api_raw(
	url_path: &str,
	api_key: &str,
	client: &Client,
) -> anyhow::Result<String> {
	let resp = client
		.get(String::from("https://api.curseforge.com/") + url_path)
		.header("x-api-key", api_key)
		.header("User-Agent", user_agent())
		.send()
		.await
		.context("Failed to send request")?
		.error_for_status()
		.context("Server reported an error")?;

	Ok(resp.error_for_status()?.text().await?)
}

/// Gets a CurseForge mod with the given ID from the API
pub async fn get_mod(id: &str, api_key: &str, client: &Client) -> anyhow::Result<CurseMod> {
	let response: CurseModResponse = request_api(&format!("v1/mods/{id}"), api_key, client).await?;
	Ok(response.data)
}

/// Gets a CurseForge mod with the given ID from the API that may not exist
pub async fn get_mod_optional(
	id: &str,
	api_key: &str,
	client: &Client,
) -> anyhow::Result<Option<CurseMod>> {
	let resp = client
		.get(format!("https://api.curseforge.com/v1/mods/{id}"))
		.header("User-Agent", user_agent())
		.header("x-api-key", api_key)
		.send()
		.await
		.context("Failed to send request")?;
	if resp.status() == reqwest::StatusCode::NOT_FOUND {
		Ok(None)
	} else {
		let resp: CurseModResponse = resp.json().await?;
		Ok(Some(resp.data))
	}
}

/// Gets a CurseForge mod with the given ID from the API
pub async fn get_mod_raw(id: &str, api_key: &str, client: &Client) -> anyhow::Result<String> {
	request_api_raw(&format!("v1/mods/{id}"), api_key, client).await
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseModResponse {
	pub data: CurseMod,
}

/// Gets a CurseForge mod description with the given ID from the API
pub async fn get_mod_description(
	id: &str,
	api_key: &str,
	client: &Client,
) -> anyhow::Result<String> {
	let response: CurseModDescriptionResponse =
		request_api(&format!("v1/mods/{id}/description"), api_key, client).await?;
	Ok(response.data)
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseModDescriptionResponse {
	pub data: String,
}

/// Gets a CurseForge mod's files with the given ID from the API
pub async fn get_mod_files(
	id: &str,
	api_key: &str,
	client: &Client,
) -> anyhow::Result<Vec<CurseFile>> {
	let response: CurseModFilesResponse =
		request_api(&format!("v1/mods/{id}/files"), api_key, client).await?;
	Ok(response.data)
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseModFilesResponse {
	pub data: Vec<CurseFile>,
}

/// A project on CurseForge
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CurseMod {
	/// Unique ID of the mod
	pub id: u32,
	/// Game ID for the mod
	pub game_id: u32,
	/// Display name for the mod
	pub name: String,
	/// Unique slug for the mod
	pub slug: String,
	/// Short description of the mod
	pub summary: String,
	/// How many downloads the mod has
	pub download_count: u32,
	/// What type of project this is
	pub class_id: u16,
	/// Authors for the mod
	pub authors: Vec<CurseAuthor>,
	/// Links for the mod
	pub links: CurseLinks,
	/// Logo for the mod
	#[serde(default)]
	pub logo: Option<CurseLogo>,
	/// Screenshots for the mod
	#[serde(default)]
	pub screenshots: Vec<CurseScreenshot>,
	/// Latest files / versions for the mod
	pub latest_files: Vec<CurseFile>,
}

/// A file for a CurseForge project
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CurseFile {
	/// Unique ID of the file
	pub id: u32,
	/// Display name for the file
	pub display_name: String,
	/// File name
	pub file_name: String,
	/// Download URL for the file
	pub download_url: Option<String>,
	/// Things that this file supports
	pub game_versions: Vec<CurseGameVersion>,
	/// Dependencies for the file
	pub dependencies: Vec<CurseFileDependency>,
	/// Hashes for the file
	pub hashes: Vec<CurseFileHash>,
	/// Stability of the release
	pub release_type: u8,
}

/// Game version for a CurseForge project, can be loaders, sides, or Minecraft versions
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum CurseGameVersion {
	Client,
	Server,
	Forge,
	NeoForge,
	LiteLoader,
	Fabric,
	Quilt,
	Cauldron,
	OptiFine,
	#[serde(untagged)]
	Minecraft(String),
}

/// A dependency for a CurseForge project file
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CurseFileDependency {
	/// ID of the dependency mod
	pub mod_id: u32,
	/// Type of dependency
	pub relation_type: u8,
}

/// A file hash for a CurseForge project
#[derive(Serialize, Deserialize, Clone)]
pub struct CurseFileHash {
	/// Hash value for the file
	pub value: String,
	/// Algorithm used for the hash
	pub algo: u8,
}

/// Author of a CurseForge project
#[derive(Serialize, Deserialize, Clone)]
pub struct CurseAuthor {
	/// Unique name of the author
	pub name: String,
}

/// Links for a CurseForge project
#[derive(Serialize, Deserialize, Clone)]
pub struct CurseLinks {
	/// Link to website for the project
	#[serde(default)]
	pub website_url: Option<String>,
	/// Link to issues for the project
	#[serde(default)]
	pub issues_url: Option<String>,
	/// Link to source code for the project
	#[serde(default)]
	pub source_url: Option<String>,
	/// Link to wiki for the project
	#[serde(default)]
	pub wiki_url: Option<String>,
}

/// Logo for a CurseForge project
#[derive(Serialize, Deserialize, Clone)]
pub struct CurseLogo {
	/// URL to the logo for the project
	pub url: String,
}

/// Screenshot for a CurseForge project
#[derive(Serialize, Deserialize, Clone)]
pub struct CurseScreenshot {
	/// URL to the screenshot for the project
	pub url: String,
}

/// Parses a CurseForge class ID into a PackageKind enum
pub fn parse_class_id(id: u16) -> Option<PackageKind> {
	match id {
		5 => Some(PackageKind::Plugin),
		6 => Some(PackageKind::Mod),
		12 => Some(PackageKind::ResourcePack),
		4471 => Some(PackageKind::Modpack),
		4546 => Some(PackageKind::Datapack),
		6552 => Some(PackageKind::Shader),
		_ => None,
	}
}

/// Unparses a PackageKind enum into a CurseForge class ID
pub fn unparse_class_id(kind: PackageKind) -> Option<u16> {
	match kind {
		PackageKind::Plugin => Some(5),
		PackageKind::Mod => Some(6),
		PackageKind::ResourcePack => Some(12),
		PackageKind::Modpack => Some(4471),
		PackageKind::Datapack => Some(4546),
		PackageKind::Shader => Some(6552),
		_ => None,
	}
}

/// Parses a CurseForge mod loader ID into a Loader enum
pub fn parse_mod_loader(id: u8) -> Loader {
	match id {
		0 => Loader::Vanilla,
		1 => Loader::Forge,
		3 => Loader::LiteLoader,
		4 => Loader::Fabric,
		5 => Loader::Quilt,
		6 => Loader::NeoForged,
		other => Loader::Unknown(other.to_string()),
	}
}

/// Unparses a Loader enum into a CurseForge mod loader ID
pub fn unparse_mod_loader(loader: &Loader) -> Option<u8> {
	match loader {
		Loader::Vanilla | Loader::Any => Some(0),
		Loader::Forge => Some(1),
		Loader::LiteLoader => Some(3),
		Loader::Fabric => Some(4),
		Loader::Quilt => Some(5),
		Loader::NeoForged => Some(6),
		_ => None,
	}
}

/// Parses a CurseForge release type ID into a PackageStability enum
pub fn parse_release_type(id: u8) -> PackageStability {
	match id {
		1 => PackageStability::Stable,
		_ => PackageStability::Latest,
	}
}

/// Searches for CurseForge mods with the given parameters
pub async fn search_mods(
	params: PackageSearchParameters,
	api_key: &str,
	client: &Client,
) -> anyhow::Result<SearchModsResponse> {
	let mut url = format!(
		"v1/mods/search?gameId=432&index={}&pageSize={}&sortField=2&sortOrder=desc",
		params.skip, params.count
	);
	if let Some(ty) = params.types.first() {
		if let Some(class_id) = unparse_class_id(*ty) {
			url.push_str(&format!("&classId={}", class_id));
		}
	}
	if let Some(search) = &params.search {
		url.push_str(&format!("&searchFilter={}", search));
	}
	let loaders = params
		.loaders
		.iter()
		.filter_map(|l| unparse_mod_loader(l))
		.collect::<Vec<_>>();
	if !loaders.is_empty() {
		url.push_str(&format!(
			"&modLoaderType={}",
			loaders
				.iter()
				.map(|l| l.to_string())
				.collect::<Vec<_>>()
				.join(",")
		));
	}
	if !params.minecraft_versions.is_empty() {
		url.push_str(&format!(
			"&gameVersions={}",
			params
				.minecraft_versions
				.iter()
				.map(|v| v.to_string())
				.collect::<Vec<_>>()
				.join(",")
		));
	}

	request_api(&url, api_key, client).await
}

/// Response for a CurseForge search request
#[derive(Serialize, Deserialize, Clone)]
pub struct SearchModsResponse {
	/// Mods returned by the search
	pub data: Vec<CurseMod>,
	/// Pagination information for the search response
	pub pagination: Pagination,
}

/// Pagination information for a CurseForge search response
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
	pub total_count: u64,
}
