use std::fs::File;
use std::io::{BufWriter, Cursor, Write};
use std::path::Path;

use anyhow::{ensure, Context};
use mcvm_shared::output::MessageContents;
use reqwest::{IntoUrl, Url};
use serde::de::DeserializeOwned;

/// Re-export of reqwest::Client for users of this download module
pub use reqwest::Client;

/// Sensible open file descriptor limit for asynchronous transfers
#[cfg(target_os = "windows")]
const FD_SENSIBLE_LIMIT: usize = 128;
/// Sensible open file descriptor limit for asynchronous transfers
#[cfg(not(target_os = "windows"))]
const FD_SENSIBLE_LIMIT: usize = 128;

/// Get the sensible limit for asynchronous transfers
pub fn get_transfer_limit() -> usize {
	if let Ok(env) = std::env::var("MCVM_TRANSFER_LIMIT") {
		env.parse().unwrap_or_default()
	} else {
		FD_SENSIBLE_LIMIT
	}
}

/// The User-Agent header for requests
fn user_agent() -> String {
	let version = env!("CARGO_PKG_VERSION");
	format!("mcvm_core_{version}")
}

/// Downloads data from a remote location
pub async fn download(url: impl IntoUrl, client: &Client) -> anyhow::Result<reqwest::Response> {
	let resp = client
		.get(url)
		.header("User-Agent", user_agent())
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

/// A persistent single download that can be used to track progress
pub struct ProgressiveDownload<W: Write> {
	response: reqwest::Response,
	writer: W,
	content_length: u64,
	bytes_downloaded: usize,
	finished: bool,
}

impl<W: Write> ProgressiveDownload<W> {
	/// Create a new ProgressiveDownload from a response
	pub fn from_response(response: reqwest::Response, writer: W) -> Self {
		Self {
			content_length: response.content_length().unwrap_or_default(),
			response,
			writer,
			bytes_downloaded: 0,
			finished: false,
		}
	}

	/// Get the number of bytes that have been downloaded
	pub fn get_downloaded(&self) -> usize {
		self.bytes_downloaded
	}

	/// Get the total length of the content
	pub fn get_total_length(&self) -> usize {
		self.content_length as usize
	}

	/// Get the progress message corresponding to this download
	pub fn get_progress(&self) -> MessageContents {
		let current = (self.get_downloaded() / 2) as u32;
		let total = (self.get_total_length() / 2) as u32;
		MessageContents::Progress { current, total }
	}

	/// Poll the download
	pub async fn poll_download(&mut self) -> anyhow::Result<()> {
		let chunk = self
			.response
			.chunk()
			.await
			.context("Failed to download chunk")?;
		if let Some(bytes) = chunk {
			self.writer
				.write_all(&bytes)
				.context("Failed to write downloaded bytes")?;
			self.bytes_downloaded += bytes.len();
		} else {
			self.finished = true;
			// Ensure that we downloaded the correct amount
			ensure!(
				self.get_downloaded() == self.get_total_length(),
				"Bytes downloaded did not equal the amount expected"
			);
		}

		Ok(())
	}

	/// Check if the download is finished
	pub fn is_finished(&self) -> bool {
		self.finished
	}
}

impl ProgressiveDownload<BufWriter<File>> {
	/// Create a new ProgressiveDownload that downloads a file
	pub async fn file(
		url: impl IntoUrl,
		path: impl AsRef<Path>,
		client: &Client,
	) -> anyhow::Result<Self> {
		let file = BufWriter::new(File::create(path).context("Failed to open file")?);
		let response = download(url, client)
			.await
			.context("Failed to get response")?;

		Ok(Self::from_response(response, file))
	}
}

impl ProgressiveDownload<Cursor<Vec<u8>>> {
	/// Create a new ProgressiveDownload that downloads bytes
	pub async fn bytes(url: impl IntoUrl, client: &Client) -> anyhow::Result<Self> {
		let response = download(url, client)
			.await
			.context("Failed to get response")?;
		let cursor = Cursor::new(Vec::with_capacity(
			response.content_length().unwrap_or_default() as usize,
		));

		Ok(Self::from_response(response, cursor))
	}

	/// Consume the download and get the resulting bytes
	pub fn finish(self) -> Vec<u8> {
		self.writer.into_inner()
	}

	/// Consume the download into JSON
	pub fn finish_json<D: DeserializeOwned>(self) -> anyhow::Result<D> {
		simd_json::from_slice(&mut self.finish()).context("Failed to deserialize downloaded output")
	}
}

/// Validates a URL with a helpful error message
pub fn validate_url(url: &str) -> anyhow::Result<()> {
	Url::parse(url).context(
		"It may help to make sure that either http:// or https:// is before the domain name",
	)?;

	Ok(())
}
