use anyhow::{anyhow, Context};
use mcvm_shared::modifications::{Modloader, ServerType};
use reqwest::Client;
use serde::Deserialize;

use super::download;

/// The type of a Modrinth project
#[derive(Deserialize)]
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

/// A Modrinth project (mod, resource pack, etc.)
#[derive(Deserialize)]
pub struct Project {
	/// The type of this project and its files
	pub project_type: ProjectType,
	/// The ID's of the available project versions
	pub versions: Vec<String>,
}

/// Get a project from the API
pub async fn get_project(project_id: &str) -> anyhow::Result<Project> {
	let url = format!("https://api.modrinth.com/v2/project/{project_id}");
	let out = download::json(url, &Client::new())
		.await
		.context("Failed to download Modrinth project")?;
	Ok(out)
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

/// A known plugin / mod loader that Modrinth supports
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnownLoader {
	/// MinecraftForge
	Forge,
	/// Fabric loader
	Fabric,
	/// Quilt loader
	Quilt,
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

/// Loader for a Modrinth project version
#[derive(Deserialize)]
#[serde(untagged)]
pub enum Loader {
	/// A loader that is known
	Known(KnownLoader),
	/// A loader that we do not know about
	Unknown(String),
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

/// A Modrinth project version
#[derive(Deserialize)]
pub struct Version {
	/// The name of this version
	pub name: String,
	/// The version number of this version
	pub version_number: String,
	/// The loaders that this version supports
	pub loaders: Vec<Loader>,
	/// The list of downloads for this version
	pub downloads: Vec<Download>,
}

impl Version {
	/// Returns the primary file download for this version
	pub fn get_primary_download(&self) -> anyhow::Result<&Download> {
		let primary = self.downloads.iter().find(|x| x.primary);
		if let Some(primary) = primary {
			Ok(primary)
		} else {
			self.downloads
				.first()
				.ok_or(anyhow!("Version has no downloads"))
		}
	}
}

/// Get a Modrinth project version
pub async fn get_version(version_id: &str) -> anyhow::Result<Version> {
	let url = format!("https://api.modrinth.com/v2/version/{version_id}");
	let out = download::json(url, &Client::new())
		.await
		.context("Failed to download Modrinth version")?;
	Ok(out)
}

/// Get multiple Modrinth project versions
pub async fn get_multiple_versions(versions: &[String]) -> anyhow::Result<Vec<Version>> {
	let mut out = Vec::new();
	for version in versions {
		out.push(
			get_version(version)
				.await
				.with_context(|| format!("Failed to get version '{version}'"))?,
		);
	}
	Ok(out)
}

/// A file download from the Modrinth API
#[derive(Deserialize)]
pub struct Download {
	/// The URL to the file download
	pub url: String,
	/// The name of the file
	pub filename: String,
	/// Whether or not this is the primary file for this version
	pub primary: bool,
}
