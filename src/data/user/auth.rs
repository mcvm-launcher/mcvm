use anyhow::Context;
use color_print::cprintln;
use reqwest::Client;

use crate::net::microsoft::auth;

/// Present the login page and secret code to the user
pub fn present_login_page_and_code(url: &str, code: &str) {
	let result = open::that_detached("url");
	if result.is_err() {
		cprintln!("<r>Failed to open link in browser");
	}
	cprintln!("<s>Open this link in your web browser if it has not opened already: <b>{url}");
	cprintln!("<s>and enter the code: <b>{code}");
}

/// Authenticate the user
pub async fn authenticate() -> anyhow::Result<()> {
	cprintln!("<y>Note: This authentication is not complete and is for debug purposes only");
	let client = auth::create_client().context("Failed to create OAuth client")?;
	let response = auth::generate_login_page(&client)
		.await
		.context("Failed to execute authorization and generate login page")?;

	present_login_page_and_code(response.verification_uri(), response.user_code().secret());

	let token = auth::get_microsoft_token(&client, response)
		.await
		.context("Failed to get Microsoft token")?;

	cprintln!("Microsoft token: {token:?}");

	let mc_token = auth::auth_microsoft(token, Client::new())
		.await
		.context("Failed to get Minecraft token")?;

	cprintln!("Minecraft token: {mc_token:?}");

	Ok(())
}
