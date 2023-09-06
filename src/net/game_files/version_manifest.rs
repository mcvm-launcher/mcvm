use anyhow::Context;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use reqwest::Client;
use serde::Deserialize;

use crate::{
	data::profile::update::manager::UpdateManager,
	io::files::{self, paths::Paths},
	net::download,
};

/// Latest available Minecraft versions in the version manifest
#[derive(Deserialize, Debug)]
pub struct LatestVersions {
	/// The latest release version
	pub release: String,
	/// The latest snapshot version
	pub snapshot: String,
}

/// Type of a version in the version manifest
#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
	/// A release version
	Release,
	/// A snapshot / development version
	Snapshot,
	/// An old alpha version
	OldAlpha,
	/// An old beta version
	OldBeta,
}

/// Entry for a version in the version manifest
#[derive(Deserialize, Debug)]
pub struct VersionEntry {
	/// The identifier for the version (e.g. "1.19.2" or "22w13a")
	pub id: String,
	/// What type of version this is
	#[serde(rename = "type")]
	pub ty: VersionType,
	/// The URL to the client version meta for this version
	pub url: String,
}

/// JSON format for the version manifest that contains all available Minecraft versions
#[derive(Deserialize, Debug)]
pub struct VersionManifest {
	/// The latest available versions
	pub latest: LatestVersions,
	/// The list of available versions, from newest to oldest
	pub versions: Vec<VersionEntry>,
}

/// Obtain the version manifest contents
async fn get_contents(
	paths: &Paths,
	manager: &UpdateManager,
	client: &Client,
	force: bool,
) -> anyhow::Result<String> {
	let mut path = paths.internal.join("versions");
	files::create_dir_async(&path).await?;
	path.push("manifest.json");

	if manager.allow_offline && !force && path.exists() {
		return tokio::fs::read_to_string(path)
			.await
			.context("Failed to read manifest contents from file");
	}

	let text = download::text(
		"https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
		client,
	)
	.await
	.context("Failed to download manifest")?;
	tokio::fs::write(&path, &text)
		.await
		.context("Failed to write manifest to a file")?;

	Ok(text)
}

/// Get the version manifest as a JSON object
pub async fn get(
	paths: &Paths,
	manager: &UpdateManager,
	client: &Client,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<VersionManifest> {
	let mut manifest_contents = get_contents(paths, manager, client, false)
		.await
		.context("Failed to get manifest contents")?;
	let manifest = match serde_json::from_str(&manifest_contents) {
		Ok(manifest) => manifest,
		Err(err) => {
			o.display(
				MessageContents::Error("Failed to obtain version manifest".to_string()),
				MessageLevel::Important,
			);
			o.display(
				MessageContents::Error(format!("{}", err)),
				MessageLevel::Important,
			);
			o.display(
				MessageContents::StartProcess("Redownloading".to_string()),
				MessageLevel::Important,
			);
			manifest_contents = get_contents(paths, manager, client, true)
				.await
				.context("Failed to donwload manifest contents")?;
			serde_json::from_str(&manifest_contents)?
		}
	};
	Ok(manifest)
}

/// Make an ordered list of versions from the manifest to use for matching
pub fn make_version_list(version_manifest: &VersionManifest) -> anyhow::Result<Vec<String>> {
	let mut out = Vec::new();
	for entry in &version_manifest.versions {
		out.push(entry.id.clone());
	}
	// We have to reverse since the version list expects oldest to newest
	out.reverse();

	Ok(out)
}
