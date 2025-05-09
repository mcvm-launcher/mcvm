use std::path::Path;

use reqwest::Client;

use crate::download;

/// Base URL for installer versions
pub static VERSIONS_URL: &str =
	"https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge";

/// Downloads the installer for the given NeoForge version
pub async fn download_installer(
	neoforge_version: &str,
	path: &Path,
	client: &Client,
) -> anyhow::Result<()> {
	let url =
		format!("{VERSIONS_URL}/{neoforge_version}/neoforge-{neoforge_version}-installer.jar");

	download::file(&url, path, client).await
}
