use std::path::Path;

use anyhow::Context;
use reqwest::{IntoUrl, Url};
use serde::de::DeserializeOwned;

/// Re-export of reqwest::Client for users of this download module
pub use reqwest::Client;

/// Sensible open file descriptor limit for asynchronous transfers
#[cfg(target_os = "windows")]
pub const FD_SENSIBLE_LIMIT: usize = 128;
/// Sensible open file descriptor limit for asynchronous transfers
#[cfg(not(target_os = "windows"))]
pub const FD_SENSIBLE_LIMIT: usize = 64;

/// Downloads data from a remote location
pub async fn download(url: impl IntoUrl, client: &Client) -> anyhow::Result<reqwest::Response> {
	let resp = client
		.get(url)
		.send()
		.await
		.context("Failed to send request")?
		.error_for_status()
		.context("Server reported an error")?;

	Ok(resp)
}

/// Downloads and returns text
pub async fn text(url: impl IntoUrl, client: &Client) -> anyhow::Result<String> {
	let text = download(url, client)
		.await
		.context("Failed to download")?
		.text()
		.await
		.context("Failed to convert download to text")?;

	Ok(text)
}

/// Downloads and returns bytes
pub async fn bytes(url: impl IntoUrl, client: &Client) -> anyhow::Result<bytes::Bytes> {
	let bytes = download(url, client)
		.await
		.context("Failed to download")?
		.bytes()
		.await
		.context("Failed to convert download to raw bytes")?;

	Ok(bytes)
}

/// Downloads and puts the contents in a file
pub async fn file(
	url: impl IntoUrl,
	path: impl AsRef<Path>,
	client: &Client,
) -> anyhow::Result<()> {
	let bytes = bytes(url, client)
		.await
		.context("Failed to download data")?;
	std::fs::write(path.as_ref(), bytes).with_context(|| {
		format!(
			"Failed to write downloaded contents to path {}",
			path.as_ref().display()
		)
	})?;

	Ok(())
}

/// Downloads and deserializes the contents into JSON
pub async fn json<T: DeserializeOwned>(url: impl IntoUrl, client: &Client) -> anyhow::Result<T> {
	download(url, client)
		.await
		.context("Failed to download JSON data")?
		.json()
		.await
		.context("Failed to parse JSON")
}

/// Validates a URL with a helpful error message
pub fn validate_url(url: &str) -> anyhow::Result<()> {
	Url::parse(url).context(
		"It may help to make sure that either http:// or https:// is before the domain name",
	)?;

	Ok(())
}
