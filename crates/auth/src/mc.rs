use super::mc_msa::{
	MinecraftAccessToken, MinecraftAuthenticationResponse, MinecraftAuthorizationFlow,
};
use anyhow::{anyhow, Context};
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::translate;
pub use oauth2::basic::{BasicClient, BasicTokenType};
pub use oauth2::reqwest::async_http_client;
pub use oauth2::{
	AuthUrl, ClientId, DeviceAuthorizationUrl, EmptyExtraTokenFields, ErrorResponse, RefreshToken,
	RequestTokenError, Scope, StandardDeviceAuthorizationResponse, StandardTokenResponse,
	TokenResponse, TokenUrl,
};
use reqwest::Response;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

const DEVICE_CODE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const MSA_AUTHORIZE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const MSA_TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";

/// Authenticate a Microsoft user using Microsoft OAuth.
/// Will authenticate every time and will not use the database.
pub async fn authenticate_microsoft_user(
	client_id: ClientId,
	client: &reqwest::Client,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<MicrosoftAuthResult> {
	let oauth_client = create_client(client_id).context("Failed to create OAuth client")?;
	let response = generate_login_page(&oauth_client)
		.await
		.context("Failed to execute authorization and generate login page")?;

	o.display_special_ms_auth(response.verification_uri(), response.user_code().secret());

	let token = get_microsoft_token(&oauth_client, response)
		.await
		.context("Failed to get Microsoft token")?;

	let result = authenticate_microsoft_user_from_token(token, client, o).await?;

	Ok(result)
}

/// Authenticate a Microsoft user from the Microsoft access token
pub async fn authenticate_microsoft_user_from_token(
	token: MicrosoftToken,
	client: &reqwest::Client,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<MicrosoftAuthResult> {
	let refresh_token = token.refresh_token().cloned();

	let mc_token = auth_minecraft(token, client)
		.await
		.context("Failed to get Minecraft token")?;

	let access_token = mc_access_token_to_string(&mc_token.access_token);

	o.display(
		MessageContents::Success(translate!(o, AuthenticationSuccessful)),
		MessageLevel::Important,
	);

	let out = MicrosoftAuthResult {
		access_token: AccessToken(access_token),
		xbox_uid: mc_token.username.clone(),
		refresh_token,
	};

	Ok(out)
}

/// Get the auth URL
fn get_auth_url() -> anyhow::Result<AuthUrl> {
	Ok(AuthUrl::new(MSA_AUTHORIZE_URL.to_string())?)
}

/// Get the token URL
fn get_token_url() -> anyhow::Result<TokenUrl> {
	Ok(TokenUrl::new(MSA_TOKEN_URL.to_string())?)
}

/// Get the device code URL
fn get_device_code_url() -> anyhow::Result<DeviceAuthorizationUrl> {
	Ok(DeviceAuthorizationUrl::new(DEVICE_CODE_URL.to_string())?)
}

/// Create the OAuth client that will be used. You will have to supply your own ClientID.
pub fn create_client(client_id: ClientId) -> anyhow::Result<BasicClient> {
	let client = BasicClient::new(
		client_id,
		None,
		get_auth_url().context("Failed to get authorization URL")?,
		Some(get_token_url().context("Failed to get token URL")?),
	)
	.set_device_authorization_url(get_device_code_url().context("Failed to get device code URL")?);

	Ok(client)
}

/// First part of the auth process
pub async fn generate_login_page(
	client: &BasicClient,
) -> anyhow::Result<StandardDeviceAuthorizationResponse> {
	let out = client
		.exchange_device_code()
		.context("Failed to exchange device code")?
		.add_scope(Scope::new("XboxLive.signin offline_access".into()))
		.request_async(async_http_client)
		.await;

	out.map_err(decorate_request_token_error)
}

/// A TokenResponse from Microsoft OAuth
pub type MicrosoftToken = StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>;

/// Get the Microsoft token. Will wait indefinitely until the user has signed in to
/// Microsoft and authenticated the application
pub async fn get_microsoft_token(
	client: &BasicClient,
	auth_response: StandardDeviceAuthorizationResponse,
) -> anyhow::Result<MicrosoftToken> {
	let out = client
		.exchange_device_access_token(&auth_response)
		.request_async(
			async_http_client,
			|x| async move { std::thread::sleep(x) },
			None,
		)
		.await;

	out.map_err(decorate_request_token_error)
}

/// Gets the access token using a refresh token
pub async fn refresh_microsoft_token(
	client: &BasicClient,
	refresh_token: &RefreshToken,
) -> anyhow::Result<MicrosoftToken> {
	let out = client
		.exchange_refresh_token(refresh_token)
		.request_async(async_http_client)
		.await;

	out.map_err(decorate_request_token_error)
}

/// Authenticates with Minecraft using a Microsoft OAuth token
pub async fn auth_minecraft(
	token: MicrosoftToken,
	client: &reqwest::Client,
) -> anyhow::Result<MinecraftAuthenticationResponse> {
	let mc_flow = MinecraftAuthorizationFlow::new(client.clone());
	let mc_token = mc_flow
		.exchange_microsoft_token(token.access_token().secret())
		.await?;

	Ok(mc_token)
}

/// Converts a Minecraft access token to a string
pub fn mc_access_token_to_string(token: &MinecraftAccessToken) -> String {
	token.clone().into_inner()
}

/// Decorates a RequestTokenError
fn decorate_request_token_error<RE: std::error::Error, T: ErrorResponse>(
	e: RequestTokenError<RE, T>,
) -> anyhow::Error {
	match e {
		RequestTokenError::ServerResponse(response) => {
			anyhow!("{response:?}").context("Server returned an error response")
		}
		e => anyhow!("{e}"),
	}
}

/// Check whether the account owns the game
pub async fn account_owns_game(
	access_token: &str,
	client: &reqwest::Client,
) -> anyhow::Result<bool> {
	let response = call_mc_api_impl(
		"https://api.minecraftservices.com/entitlements/mcstore",
		access_token,
		client,
	)
	.await
	.context("Failed to call API to check game ownership")?;
	// Instead of using the JSON format, it's faster and easier to just check for strings in the response
	let text = response.text().await?;
	let out = text.contains("product_minecraft") | text.contains("game_minecraft");
	Ok(out)
}

/// Result from the Microsoft authentication function
pub struct MicrosoftAuthResult {
	/// The access token for logging into the game and other API services
	pub access_token: AccessToken,
	/// The Xbox UID of the user
	pub xbox_uid: String,
	/// The refresh token
	pub refresh_token: Option<RefreshToken>,
}

/// An access token for a user that will be hidden in debug messages
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AccessToken(pub String);

impl Debug for AccessToken {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "AccessToken(***)")
	}
}

/// Utility function to query the Minecraft Services API with correct authorization
pub async fn call_mc_api<T: DeserializeOwned>(
	url: &str,
	access_token: &str,
	client: &reqwest::Client,
) -> anyhow::Result<T> {
	let response = call_mc_api_impl(url, access_token, client).await?;
	let response = response.json().await?;

	Ok(response)
}

async fn call_mc_api_impl(
	url: &str,
	access_token: &str,
	client: &reqwest::Client,
) -> anyhow::Result<Response> {
	let response = client
		.get(url)
		.header("Authorization", format!("Bearer {access_token}"))
		.send()
		.await?
		.error_for_status()?;

	Ok(response)
}

/// Keypair in player certificate
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Keypair {
	/// Private key
	#[serde(alias = "privateKey")]
	pub private_key: String,
	/// Public key
	#[serde(alias = "publicKey")]
	pub public_key: String,
}
