use anyhow::Context;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::{translate, UpdateDepth};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::io::files::{self, paths::Paths};
use crate::io::update::UpdateManager;
use crate::io::{json_from_file, json_to_file};
use crate::net::download::ProgressiveDownload;
use crate::util::versions::VersionName;

/// JSON format for the version manifest that contains all available Minecraft versions
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VersionManifest {
	/// The latest available versions
	pub latest: LatestVersions,
	/// The list of available versions, from newest to oldest
	pub versions: Vec<VersionEntry>,
}

/// Entry for a version in the version manifest
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VersionEntry {
	/// The identifier for the version (e.g. "1.19.2" or "22w13a")
	pub id: String,
	/// What type of version this is
	#[serde(rename = "type")]
	#[serde(default)]
	pub ty: VersionType,
	/// The URL to the client version meta for this version
	pub url: String,
	/// Whether the client meta needs to be unzipped first
	#[serde(default)]
	pub is_zipped: bool,
}

/// Type of a version in the version manifest
#[derive(Deserialize, Serialize, Debug, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
	/// A release version
	#[default]
	Release,
	/// A snapshot / development version
	Snapshot,
	/// An old alpha version
	OldAlpha,
	/// An old beta version
	OldBeta,
}

/// Latest available Minecraft versions in the version manifest
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LatestVersions {
	/// The latest release version
	pub release: VersionName,
	/// The latest snapshot version
	pub snapshot: VersionName,
}

/// Get the version manifest
pub async fn get(
	paths: &Paths,
	manager: &UpdateManager,
	client: &Client,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<VersionManifest> {
	let manifest = get_contents(paths, manager, client, false, o).await;
	let manifest = match manifest {
		Ok(manifest) => manifest,
		Err(err) => {
			o.display(
				MessageContents::Error("Failed to obtain version manifest".into()),
				MessageLevel::Important,
			);
			o.display(
				MessageContents::Error(format!("{}", err)),
				MessageLevel::Important,
			);
			o.display(
				MessageContents::StartProcess("Redownloading".into()),
				MessageLevel::Important,
			);
			get_contents(paths, manager, client, true, o)
				.await
				.context("Failed to download manifest contents")?
		}
	};
	Ok(manifest)
}

/// Get the version manifest with progress output around it
pub async fn get_with_output(
	paths: &Paths,
	manager: &UpdateManager,
	client: &Client,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<VersionManifest> {
	o.start_process();
	o.display(
		MessageContents::StartProcess("Obtaining version manifest".into()),
		MessageLevel::Important,
	);

	let manifest = get(paths, manager, client, o)
		.await
		.context("Failed to get version manifest")?;

	o.display(
		MessageContents::Success("Version manifest obtained".into()),
		MessageLevel::Important,
	);
	o.end_process();

	Ok(manifest)
}

/// Obtain the version manifest contents
async fn get_contents(
	paths: &Paths,
	manager: &UpdateManager,
	client: &Client,
	force: bool,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<VersionManifest> {
	let mut path = paths.internal.join("versions");
	files::create_dir(&path)?;
	path.push("manifest.json");
	if manager.update_depth < UpdateDepth::Force && !force && path.exists() {
		return json_from_file(path).context("Failed to read manifest contents from file");
	}

	let mut download = ProgressiveDownload::bytes(
		"https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
		client,
	)
	.await?;

	while !download.is_finished() {
		download.poll_download().await?;
		o.display(
			MessageContents::Associated(
				Box::new(download.get_progress()),
				Box::new(MessageContents::Simple(translate!(
					o,
					StartDownloadingVersionManifest
				))),
			),
			MessageLevel::Important,
		);
	}
	let manifest = download.finish_json()?;

	json_to_file(path, &manifest).context("Failed to write manifest to a file")?;

	Ok(manifest)
}

/// Make an ordered list of versions from the manifest to use for matching
pub fn make_version_list(version_manifest: &VersionManifest) -> Vec<String> {
	let mut out = Vec::new();
	for entry in &version_manifest.versions {
		out.push(entry.id.clone());
	}
	// We have to reverse since the version list expects oldest to newest
	out.reverse();

	out
}

/// Combination of the version manifest and version list
pub struct VersionManifestAndList {
	/// The version manifest
	pub manifest: VersionManifest,
	/// The list of versions in order, kept in sync with the manifest
	pub list: Vec<String>,
}

impl VersionManifestAndList {
	/// Construct a new VersionManifestAndList
	pub fn new(manifest: VersionManifest) -> Self {
		let list = make_version_list(&manifest);
		Self { manifest, list }
	}

	/// Change the version manifest and list
	pub fn set(&mut self, manifest: VersionManifest) -> anyhow::Result<()> {
		self.list = make_version_list(&manifest);
		self.manifest = manifest;

		Ok(())
	}
}
