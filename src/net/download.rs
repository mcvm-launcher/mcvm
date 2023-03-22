use reqwest::Client;

// Sensible open file descriptor limit for asynchronous transfers
pub static FD_SENSIBLE_LIMIT: usize = 15;

/// Downloads a file
pub async fn download(url: &str) -> Result<reqwest::Response, reqwest::Error> {
	let resp = Client::new().get(url).send().await?.error_for_status()?;

	Ok(resp)
}

/// Downloads and returns text
pub async fn download_text(url: &str) -> Result<String, reqwest::Error> {
	let text = download(url).await?.text().await?;

	Ok(text)
}

/// Downloads and returns bytes
pub async fn download_bytes(url: &str) -> Result<bytes::Bytes, reqwest::Error> {
	let bytes = download(url).await?.bytes().await?;

	Ok(bytes)
}
