use anyhow::Context;
use nitro_shared::{
	loaders::Loader,
	pkg::{PackageKind, PackageStability},
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

	Ok(resp.error_for_status()?.json().await?)
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
	let mut response: CurseModResponse =
		request_api(&format!("v1/mods/{id}"), api_key, client).await?;
	Ok(response.data.remove(0))
}

/// Gets a CurseForge mod with the given ID from the API
pub async fn get_mod_raw(id: &str, api_key: &str, client: &Client) -> anyhow::Result<String> {
	request_api_raw(&format!("v1/mods/{id}"), api_key, client).await
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseModResponse {
	pub data: Vec<CurseMod>,
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
#[derive(Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseFile {
	/// Unique ID of the file
	pub id: u32,
	/// Display name for the file
	pub display_name: String,
	/// File name
	pub file_name: String,
	/// Download URL for the file
	pub download_url: String,
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
	#[serde(untagged)]
	Minecraft(String),
}

/// A dependency for a CurseForge project file
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseFileDependency {
	/// ID of the dependency mod
	pub mod_id: u32,
	/// Type of dependency
	pub relation_type: u8,
}

/// A file hash for a CurseForge project
#[derive(Serialize, Deserialize)]
pub struct CurseFileHash {
	/// Hash value for the file
	pub value: String,
	/// Algorithm used for the hash
	pub algo: u8,
}

/// Author of a CurseForge project
#[derive(Serialize, Deserialize)]
pub struct CurseAuthor {
	/// Unique name of the author
	pub name: String,
}

/// Links for a CurseForge project
#[derive(Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize)]
pub struct CurseLogo {
	/// URL to the logo for the project
	pub url: String,
}

/// Screenshot for a CurseForge project
#[derive(Serialize, Deserialize)]
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

/// Parses a CurseForge release type ID into a PackageStability enum
pub fn parse_release_type(id: u8) -> PackageStability {
	match id {
		1 => PackageStability::Stable,
		_ => PackageStability::Latest,
	}
}
