use anyhow::{anyhow, Context};
use minecraft_msa_auth::{MinecraftAuthenticationResponse, MinecraftAuthorizationFlow};
use oauth2::{
	basic::{BasicClient, BasicTokenType},
	reqwest::async_http_client,
	AuthUrl, ClientId, DeviceAuthorizationUrl, EmptyExtraTokenFields, ErrorResponse,
	RequestTokenError, Scope, StandardDeviceAuthorizationResponse, StandardTokenResponse,
	TokenResponse, TokenUrl,
};

const DEVICE_CODE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const MSA_AUTHORIZE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const MSA_TOKEN_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";

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
		.add_scope(Scope::new(String::from("XboxLive.signin offline_access")))
		.request_async(async_http_client)
		.await;

	out.map_err(decorate_request_token_error)
}

pub type MicrosoftToken = StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>;

/// Get the Microsoft token. Will wait indefinitely until the user has signed in to
/// Microsoft and authenticated the application
pub async fn get_microsoft_token(
	client: &BasicClient,
	auth_response: StandardDeviceAuthorizationResponse,
) -> anyhow::Result<MicrosoftToken> {
	let out = client
		.exchange_device_access_token(&auth_response)
		.request_async(async_http_client, tokio::time::sleep, None)
		.await;

	out.map_err(decorate_request_token_error)
}

/// Authenticates with Minecraft using a Microsoft OAuth token
pub async fn auth_minecraft(
	token: MicrosoftToken,
	client: reqwest::Client,
) -> anyhow::Result<MinecraftAuthenticationResponse> {
	let mc_flow = MinecraftAuthorizationFlow::new(client);
	let mc_token = mc_flow
		.exchange_microsoft_token(token.access_token().secret())
		.await?;

	Ok(mc_token)
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
