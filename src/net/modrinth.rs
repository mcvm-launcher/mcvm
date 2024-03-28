use anyhow::{anyhow, Context};
use mcvm_core::net::download;
use mcvm_shared::modifications::{Modloader, ServerType};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// A Modrinth project (mod, resource pack, etc.)
#[derive(Deserialize, Serialize)]
pub struct Project {
	/// The ID of the project
	pub id: String,
	/// The type of this project and its files
	pub project_type: ProjectType,
	/// The ID's of the available project versions
	pub versions: Vec<String>,
	/// The Minecraft versions this project is available for
	pub game_versions: Vec<String>,
	/// The display name of the project
	pub title: String,
	/// The short description of the project
	pub description: String,
	/// URL to the icon
	pub icon_url: String,
	/// URL to the issue tracker
	pub issues_url: Option<String>,
	/// URL to the source
	pub source_url: Option<String>,
	/// URL to the wiki
	pub wiki_url: Option<String>,
	/// URL to the Discord
	pub discord_url: Option<String>,
	/// Donation URLs
	pub donation_urls: Vec<DonationLink>,
	/// The loaders this project is available for
	pub loaders: Vec<Loader>,
	/// The license of the project
	pub license: License,
}

/// The type of a Modrinth project
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
	/// A mod project
	Mod,
	/// A modpack project
	Modpack,
	/// A resource pack project
	ResourcePack,
	/// A shader project
	Shader,
	/// A datapack project
	Datapack,
	/// A plugin project
	Plugin,
}

/// Get a project from the API
pub async fn get_project(project_id: &str, client: &Client) -> anyhow::Result<Project> {
	let url = format_get_project_url(project_id);
	let out = download::json(url, client)
		.await
		.context("Failed to download Modrinth project")?;
	Ok(out)
}

/// Get the raw response of a project from the API
pub async fn get_project_raw(project_id: &str, client: &Client) -> anyhow::Result<String> {
	let url = format_get_project_url(project_id);
	let out = download::text(url, client)
		.await
		.context("Failed to download Modrinth project")?;
	Ok(out)
}

/// Format the URL for the get_project API
fn format_get_project_url(project_id: &str) -> String {
	format!("https://api.modrinth.com/v2/project/{project_id}")
}

/// Release channel for a Modrinth project version
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseChannel {
	/// A finished release version
	Release,
	/// An unfinished beta version
	Beta,
	/// An unfinished alpha version
	Alpha,
}

/// A Modrinth project version
#[derive(Deserialize, Serialize)]
pub struct Version {
	/// The ID of this version
	pub id: String,
	/// The name of this version
	pub name: String,
	/// The version number of this version
	pub version_number: String,
	/// The loaders that this version supports
	pub loaders: Vec<Loader>,
	/// The list of downloads for this version
	pub files: Vec<Download>,
	/// The game versions this version supports
	pub game_versions: Vec<String>,
	/// The dependencies that this version has
	pub dependencies: Vec<Dependency>,
}

/// Loader for a Modrinth project version
#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum Loader {
	/// A loader that is known
	Known(KnownLoader),
	/// A loader that we do not know about
	Unknown(String),
}

/// A known plugin / mod loader that Modrinth supports
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum KnownLoader {
	/// MinecraftForge
	Forge,
	/// Fabric loader
	Fabric,
	/// Quilt loader
	Quilt,
	/// NeoForged loader
	#[serde(rename = "neoforge")]
	NeoForged,
	/// Rift loader
	Rift,
	/// Bukkit loaders
	Bukkit,
	/// Spigot server
	Spigot,
	/// Paper server
	Paper,
	/// Sponge server
	Sponge,
	/// Purpur server
	Purpur,
	/// Folia server
	Folia,
}

impl Loader {
	/// Checks if this loader matches an mcvm modloader
	pub fn matches_modloader(&self, modloader: Modloader) -> bool {
		match modloader {
			Modloader::Forge => matches!(self, Self::Known(KnownLoader::Forge)),
			Modloader::Fabric => matches!(self, Self::Known(KnownLoader::Fabric)),
			Modloader::Quilt => matches!(self, Self::Known(KnownLoader::Quilt)),
			_ => true,
		}
	}

	/// Checks if this loader matches an mcvm plugin loader
	pub fn matches_plugin_loader(&self, plugin_loader: ServerType) -> bool {
		match plugin_loader {
			ServerType::Paper => matches!(
				self,
				Self::Known(
					KnownLoader::Paper
						| KnownLoader::Bukkit | KnownLoader::Spigot
						| KnownLoader::Sponge
				)
			),
			_ => true,
		}
	}
}

impl Version {
	/// Returns the primary file download for this version
	pub fn get_primary_download(&self) -> anyhow::Result<&Download> {
		let primary = self.files.iter().find(|x| x.primary);
		if let Some(primary) = primary {
			Ok(primary)
		} else {
			self.files
				.first()
				.ok_or(anyhow!("Version has no downloads"))
		}
	}
}

/// Get a Modrinth project version
pub async fn get_version(version_id: &str, client: &Client) -> anyhow::Result<Version> {
	let url = format_get_version_url(version_id);
	let out = download::json(url, client)
		.await
		.context("Failed to download Modrinth version")?;
	Ok(out)
}

/// Get the raw response of a version from the API
pub async fn get_version_raw(version_id: &str, client: &Client) -> anyhow::Result<String> {
	let url = format_get_version_url(version_id);
	let out = download::text(url, client)
		.await
		.context("Failed to download Modrinth version")?;
	Ok(out)
}

/// Format the URL for the get_version API
fn format_get_version_url(version_id: &str) -> String {
	format!("https://api.modrinth.com/v2/version/{version_id}")
}

/// Get multiple Modrinth project versions
pub async fn get_multiple_versions(
	versions: &[String],
	client: &Client,
) -> anyhow::Result<Vec<Version>> {
	// Use the multiple-versions API endpoint as it's faster
	let param = serde_json::to_string(versions)
		.context("Failed to convert version list to API parameter")?;
	let url = format!("https://api.modrinth.com/v2/versions?ids={param}");
	download::json(url, client).await
}

/// A file download from the Modrinth API
#[derive(Deserialize, Serialize)]
pub struct Download {
	/// The URL to the file download
	pub url: String,
	/// The name of the file
	pub filename: String,
	/// Whether or not this is the primary file for this version
	pub primary: bool,
}

/// A version dependency
#[derive(Deserialize, Serialize)]
pub struct Dependency {
	/// The ID of the project
	pub project_id: String,
	/// The ID of the version
	pub version_id: Option<String>,
	/// The type of the dependency
	pub dependency_type: DependencyType,
}

/// The type of a dependency
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyType {
	/// A required dependency
	Required,
	/// An optional / recommended dependency
	Optional,
	/// An incompatible dependency
	Incompatible,
	/// An embedded dependency
	Embedded,
}

/// Information about a project license
#[derive(Deserialize, Serialize)]
pub struct License {
	/// The short ID of the license
	pub id: String,
}

/// Information about a donation link
#[derive(Deserialize, Serialize)]
pub struct DonationLink {
	/// The URL of the link
	pub url: String,
}

/// Get the team members of a project
pub async fn get_project_team(project_id: &str, client: &Client) -> anyhow::Result<Vec<Member>> {
	let url = format!("https://api.modrinth.com/v2/project/{project_id}/members");
	download::json(url, client).await
}

/// A member of a project team
#[derive(Deserialize, Serialize)]
pub struct Member {
	/// The user that represents this member
	pub user: User,
	/// The ordering of the team member
	pub ordering: i32,
}

/// A user on the platform
#[derive(Deserialize, Serialize)]
pub struct User {
	/// The user's username
	pub username: String,
}
