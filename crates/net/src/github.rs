use reqwest::Client;
use serde::Deserialize;

/// A single GitHub release
#[derive(Deserialize)]
pub struct GithubRelease {
	pub id: u32,
	pub tag_name: String,
	pub name: String,
	pub body: Option<String>,
	pub assets: Vec<GithubAsset>,
}

/// An asset for a GitHub release
#[derive(Deserialize)]
pub struct GithubAsset {
	pub name: String,
	pub content_type: String,
	pub browser_download_url: String,
}

/// Get the list of releases for a GitHub project
pub async fn get_github_releases(
	owner: &str,
	repo: &str,
	client: &Client,
) -> anyhow::Result<Vec<GithubRelease>> {
	crate::download::json(
		&format!("https://api.github.com/repos/{owner}/{repo}/releases"),
		client,
	)
	.await
}
