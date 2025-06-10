use crate::download::{self, user_agent};
use anyhow::{anyhow, Context};
use mcvm_shared::{
	modifications::{Modloader, ServerType},
	pkg::PackageSearchParameters,
};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

/// A Modrinth project (mod, resource pack, etc.)
#[derive(Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct Project {
	/// The ID of the project
	#[serde(alias = "project_id")]
	pub id: String,
	/// The slug of the project
	pub slug: String,
	/// The type of this project and its files
	pub project_type: ProjectType,
	/// The ID's of the available project versions
	pub versions: Vec<String>,
	/// The Minecraft versions this project is available for
	pub game_versions: Vec<String>,
	/// The loaders this project is available for
	pub loaders: Vec<Loader>,
	/// The project's support on the client side
	pub client_side: SideSupport,
	/// The project's support on the server side
	pub server_side: SideSupport,
	/// The project's team ID
	pub team: String,
	/// The display name of the project
	pub title: String,
	/// The short description of the project
	pub description: String,
	/// The long description of the project
	pub body: Option<String>,
	/// URL to the icon
	pub icon_url: Option<String>,
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
	/// The license of the project
	pub license: License,
	/// The gallery items of the project
	pub gallery: Option<Vec<GalleryEntry>>,
}

/// The type of a Modrinth project
#[derive(Deserialize, Serialize, Copy, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
	/// A mod project
	#[default]
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

/// Get a project from the API, returning none if it does not exist
pub async fn get_project_optional(
	project_id: &str,
	client: &Client,
) -> anyhow::Result<Option<Project>> {
	let url = format_get_project_url(project_id);

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

/// Get multiple Modrinth projects
pub async fn get_multiple_projects(
	projects: &[String],
	client: &Client,
) -> anyhow::Result<Vec<Project>> {
	if projects.is_empty() {
		return Ok(Vec::new());
	}
	// Use the multiple-projects API endpoint as it's faster
	let param = serde_json::to_string(projects)
		.context("Failed to convert project list to API parameter")?;
	let url = format!("https://api.modrinth.com/v2/projects?ids={param}");
	download::json(url, client).await
}

/// Release channel for a Modrinth project version
#[derive(Deserialize, Serialize, Clone, Copy)]
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
#[derive(Deserialize, Serialize, Clone)]
pub struct Version {
	/// The ID of this version
	pub id: String,
	/// The ID of the project this version is from
	pub project_id: String,
	/// The name of this version
	pub name: String,
	/// The version number of this version
	pub version_number: String,
	/// The type / release channel of this version
	pub version_type: ReleaseChannel,
	/// The loaders that this version supports
	pub loaders: Vec<Loader>,
	/// The list of downloads for this version
	pub files: Vec<Download>,
	/// The game versions this version supports
	pub game_versions: Vec<String>,
	/// The dependencies that this version has
	pub dependencies: Vec<Dependency>,
	/// Whether this version is featured
	pub featured: bool,
	/// The date this version was published in ISO-8601
	pub date_published: String,
}

/// Loader for a Modrinth project version
#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum Loader {
	/// A loader that is known
	Known(KnownLoader),
	/// A loader that we do not know about
	Unknown(String),
}

/// A known plugin / mod loader that Modrinth supports
#[derive(Deserialize, Serialize, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum KnownLoader {
	/// The Vanilla game
	Minecraft,
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
	/// Liteloader
	Liteloader,
	/// Risugami's Modloader
	#[serde(rename = "modloader")]
	Risugamis,
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
	/// Iris shader loader
	Iris,
	/// Optifine shader loader
	Optifine,
	/// Datapack loader
	Datapack,
	/// Velocity loader
	Velocity,
	/// BungeeCord loader
	#[serde(rename = "bungeecord")]
	BungeeCord,
	/// Waterfall loader
	Waterfall,
}

impl Loader {
	/// Checks if this loader matches an mcvm modloader
	pub fn matches_modloader(&self, modloader: &Modloader) -> bool {
		match modloader {
			Modloader::Forge => matches!(self, Self::Known(KnownLoader::Forge)),
			Modloader::Fabric => matches!(self, Self::Known(KnownLoader::Fabric)),
			Modloader::Quilt => matches!(self, Self::Known(KnownLoader::Quilt)),
			_ => true,
		}
	}

	/// Checks if this loader matches an mcvm plugin loader
	pub fn matches_plugin_loader(&self, plugin_loader: &ServerType) -> bool {
		match plugin_loader {
			ServerType::Paper => matches!(
				self,
				Self::Known(
					KnownLoader::Paper
						| KnownLoader::Bukkit
						| KnownLoader::Spigot
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

/// Gets all the versions for a Modrinth project
pub async fn get_project_versions(
	project_id: &str,
	client: &Client,
) -> anyhow::Result<Vec<Version>> {
	let url = format!("https://api.modrinth.com/v2/project/{project_id}/version");
	download::json(url, client).await
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
	if versions.is_empty() {
		return Ok(Vec::new());
	}

	// Use the multiple-versions API endpoint as it's faster
	let param = serde_json::to_string(versions)
		.context("Failed to convert version list to API parameter")?;
	let url = format!("https://api.modrinth.com/v2/versions?ids={param}");
	download::json(url, client).await
}

/// A file download from the Modrinth API
#[derive(Deserialize, Serialize, Clone)]
pub struct Download {
	/// The URL to the file download
	pub url: String,
	/// The name of the file
	pub filename: String,
	/// Whether or not this is the primary file for this version
	pub primary: bool,
}

/// A version dependency
#[derive(Deserialize, Serialize, Clone)]
pub struct Dependency {
	/// The ID of the project
	pub project_id: String,
	/// The ID of the version
	pub version_id: Option<String>,
	/// The type of the dependency
	pub dependency_type: DependencyType,
}

/// The type of a dependency
#[derive(Deserialize, Serialize, Clone, Copy)]
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
#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum License {
	Short(String),
	Long(LongLicense),
}

impl Default for License {
	fn default() -> Self {
		Self::Short("ARR".into())
	}
}

/// Long information about a project license
#[derive(Deserialize, Serialize, Clone)]
pub struct LongLicense {
	/// The short ID of the license
	pub id: String,
	/// The URL to a custom license
	pub url: Option<String>,
}

/// Information about a donation link
#[derive(Deserialize, Serialize, Clone)]
pub struct DonationLink {
	/// The URL of the link
	pub url: String,
}

/// An entry in a project's gallery
#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum GalleryEntry {
	Simple(String),
	Full(FullGalleryEntry),
}

impl GalleryEntry {
	/// Get the URL to this entry
	pub fn get_url(&self) -> &str {
		match self {
			Self::Simple(url) => url,
			Self::Full(entry) => &entry.url,
		}
	}
}

/// An entry in a project's gallery
#[derive(Deserialize, Serialize, Clone)]
pub struct FullGalleryEntry {
	/// The URL to the gallery image
	pub url: String,
	/// Whether the gallery image is a featured banner on the project page
	pub featured: bool,
}

/// Support status for a project on a specific side
#[derive(Deserialize, Serialize, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
pub enum SideSupport {
	/// Required to be on this side
	Required,
	/// Can optionally be on this side
	Optional,
	/// Unsupported on this side
	Unsupported,
	/// Support unknown
	#[default]
	Unknown,
}

/// Get the team members of a project
pub async fn get_project_team(project_id: &str, client: &Client) -> anyhow::Result<Vec<Member>> {
	let url = format!("https://api.modrinth.com/v2/project/{project_id}/members");
	download::json(url, client).await
}

/// Get multiple Modrinth teams
pub async fn get_multiple_teams(
	teams: &[String],
	client: &Client,
) -> anyhow::Result<Vec<Vec<Member>>> {
	if teams.is_empty() {
		return Ok(Vec::new());
	}
	// Use the multiple-teams API endpoint as it's faster
	let param =
		serde_json::to_string(teams).context("Failed to convert team list to API parameter")?;
	let url = format!("https://api.modrinth.com/v2/teams?ids={param}");
	download::json(url, client).await
}

/// A member of a project team
#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Member {
	/// The ordering of the team member
	pub ordering: i32,
	/// The user that represents this member
	pub user: User,
	/// The ID of the team this member is a part of
	pub team_id: String,
}

/// A user on the platform
#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct User {
	/// The user's username
	pub username: String,
}

/// Search projects from the Modrinth API. Note that the projects returned by this have many default fields and should NOT be used as the final projects.
pub async fn search_projects(
	params: PackageSearchParameters,
	client: &Client,
	modpacks: bool,
) -> anyhow::Result<SearchResults> {
	let limit = if params.count > 100 {
		100
	} else {
		params.count
	};
	let search = if let Some(search) = params.search {
		format!("&query={search}")
	} else {
		String::new()
	};
	let facets = format!(
		"facets=[[\"project_types{}modpack\"]]",
		if modpacks { "==" } else { "!=" }
	);
	let url = format!("https://api.modrinth.com/v2/search?limit={limit}{search}&{facets}");

	download::json(url, client).await
}

#[derive(Deserialize, Serialize)]
pub struct SearchResults {
	/// The results
	pub hits: Vec<SearchedProject>,
}

/// A project result in the search
#[derive(Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct SearchedProject {
	/// The ID of the project
	#[serde(alias = "project_id")]
	pub id: String,
	/// The slug of the project
	pub slug: String,
	/// The type of this project and its files
	pub project_type: ProjectType,
}
