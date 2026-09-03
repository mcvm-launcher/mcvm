#[cfg(feature = "net")]
use std::path::Path;
use std::{
	io::{Read, Seek},
	path::PathBuf,
};

use anyhow::Context;
#[cfg(feature = "net")]
use nitro_net::curseforge::{CurseFile, CurseMod};
#[cfg(feature = "net")]
use nitro_shared::pkg::PackageKind;
#[cfg(feature = "net")]
use nitro_shared::{Side, versions::VersionInfo};
use serde::{Deserialize, Serialize};
use zip::ZipArchive;

#[cfg(feature = "net")]
use crate::addon::{Addon, modpack::apply_zip_override, storage};

/// CurseForge modpack
pub struct CurseForgePack<R> {
	manifest: CurseForgeManifest,
	zip: ZipArchive<R>,
}

impl<R: Read + Seek> CurseForgePack<R> {
	/// Creates a new CurseForgePack from a stream
	pub fn from_stream(r: R) -> anyhow::Result<Self> {
		let mut zip = ZipArchive::new(r).context("Failed to open pack zip file")?;
		let manifest = zip
			.by_name("manifest.json")
			.context("Failed to open CurseForge manifest")?;
		let manifest: CurseForgeManifest =
			serde_json::from_reader(manifest).context("Failed to deserialize manifest")?;

		Ok(Self { manifest, zip })
	}

	/// Gets the manifest of this modpack
	pub fn manifest(&self) -> &CurseForgeManifest {
		&self.manifest
	}

	/// Downloads all the files in this modpack to the given addons directory.
	#[cfg(feature = "net")]
	pub async fn download(
		&mut self,
		addons_dir: &Path,
		client: &nitro_net::download::Client,
		api_key: &str,
	) -> anyhow::Result<(Vec<CurseMod>, Vec<CurseFile>)> {
		use tokio::task::JoinSet;

		let projects = self
			.manifest
			.files
			.iter()
			.map(|x| x.project_id)
			.collect::<Vec<_>>();
		let files: Vec<_> = self.manifest.files.iter().map(|x| x.file_id).collect();
		let (projects, files) = tokio::try_join!(
			nitro_net::curseforge::get_mods(&projects, api_key, client),
			nitro_net::curseforge::get_many_files(&files, api_key, client)
		)
		.context("Failed to get CurseForge projects or files")?;

		let mut tasks = JoinSet::new();
		for file in &files {
			let source_path = storage::get_generic_addon_path(
				addons_dir,
				&file.mod_id.to_string(),
				Some(file.id.to_string()),
			);

			if source_path.exists() {
				continue;
			}

			if let Some(download_url) = &file.download_url {
				let download_url = download_url.clone();
				let client = client.clone();
				tasks.spawn(async move {
					if let Some(parent) = source_path.parent() {
						let _ = std::fs::create_dir_all(parent);
					}
					nitro_net::download::file(download_url, source_path, &client).await
				});
			}
		}

		while let Some(result) = tasks.join_next().await {
			result??;
		}

		Ok((projects, files))
	}

	/// Applies this modpack to the given target directory. Files must already be downloaded. Returns a list of addons.
	#[cfg(feature = "net")]
	pub fn apply(
		&mut self,
		target: &Path,
		files: &[CurseFile],
		projects: &[CurseMod],
		addons_dir: &Path,
		side: Side,
		minecraft_versions: &[String],
		mut old_pack: Option<&mut Self>,
	) -> anyhow::Result<Vec<Addon>> {
		// Link addons
		let mut addons = Vec::new();
		for file in files {
			let source_path = storage::get_generic_addon_path(
				addons_dir,
				&file.mod_id.to_string(),
				Some(file.id.to_string()),
			);
			if !source_path.exists() {
				continue;
			}
			let Some(project) = projects.iter().find(|p| p.id == file.mod_id) else {
				anyhow::bail!("Failed to find project for file {}", file.display_name);
			};
			let kind = nitro_net::curseforge::parse_class_id(project.class_id)
				.unwrap_or(PackageKind::Mod)
				.to_addon_kind()
				.unwrap();

			let mut addon = Addon {
				kind,
				file_name: file.file_name.clone(),
				original_path: None,
				target_paths: Vec::new(),
				source: Some(source_path),
				hashes: Default::default(),
			};
			let version_info = VersionInfo {
				version: self.manifest.minecraft.version.clone(),
				versions: minecraft_versions.to_vec(),
			};
			addon.get_targets(side, target, &[], None, &version_info);

			addons.push(addon);
		}

		for addon in &addons {
			addon.link().context("Failed to link addon")?;
		}

		// Apply overrides
		for i in 0..self.zip.len() {
			let file = self.zip.by_index(i)?;
			if file.is_dir() {
				continue;
			}
			let Some(name) = file.enclosed_name() else {
				continue;
			};

			let target_rel_path = if let Ok(path) = name.strip_prefix("overrides/") {
				path
			} else {
				continue;
			};

			apply_zip_override(
				file,
				target_rel_path,
				target,
				old_pack.as_mut().map(|x| &mut x.zip),
			)?;
		}

		Ok(addons)
	}

	/// Gets the overrides as relative paths
	pub fn get_overrides(&mut self) -> anyhow::Result<Vec<PathBuf>> {
		let mut out = Vec::new();
		for i in 0..self.zip.len() {
			let file = self.zip.by_index(i)?;
			if file.is_dir() {
				continue;
			}
			let Some(name) = file.enclosed_name() else {
				continue;
			};

			if let Ok(path) = name.strip_prefix("overrides/") {
				out.push(path.to_owned());
			}
		}

		Ok(out)
	}
}

/// Information file for a CurseForge pack
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeManifest {
	/// Name of the modpack
	pub name: String,
	/// Version of the modpack
	pub version: String,
	/// Files in the pack
	pub files: Vec<CurseForgePackFile>,
	/// Minecraft / mod loader dependencies
	pub minecraft: CurseForgePackDependencies,
}

/// File in the CurseForge pack index
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgePackFile {
	/// ID of the CurseForge project this file belongs to
	#[serde(rename = "projectID")]
	pub project_id: u32,
	/// ID of the CurseForge file / version
	#[serde(rename = "fileID")]
	pub file_id: u32,
}

/// CurseForge pack environment dependencies
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgePackDependencies {
	/// Required Minecraft version
	pub version: String,
	/// Required mod loaders
	pub mod_loaders: Vec<CurseForgeModLoader>,
}

/// CurseForge mod loader information
#[derive(Serialize, Deserialize)]
pub struct CurseForgeModLoader {
	/// ID of the loader
	pub id: String,
	/// Whether this is the primary loader for the pack
	pub primary: bool,
}
