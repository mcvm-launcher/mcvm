//! Taken from https://github.com/KernelFreeze/minecraft-msa-auth
//! and modified to use a newer version of reqwest

use std::collections::HashMap;
use std::fmt::Debug;

use getset::{CopyGetters, Getters};
use nutype::nutype;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

const MINECRAFT_LOGIN_WITH_XBOX: &str =
	"https://api.minecraftservices.com/authentication/login_with_xbox";
const XBOX_USER_AUTHENTICATE: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XBOX_XSTS_AUTHORIZE: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";

/// Represents a Minecraft access token
#[nutype(
	validate(not_empty),
	derive(Clone, PartialEq, Eq, Hash, Deserialize, Serialize, AsRef, Into)
)]
pub struct MinecraftAccessToken(String);

impl Debug for MinecraftAccessToken {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("MinecraftAccessToken")
			.field(&"[redacted]")
			.finish()
	}
}

/// Represents the token type of a Minecraft access token
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum MinecraftTokenType {
	Bearer,
}

/// Represents an error that can occur when authenticating with Minecraft.
#[derive(Error, Debug)]
pub enum MinecraftAuthorizationError {
	/// An error occurred while sending the request
	#[error(transparent)]
	Reqwest(#[from] reqwest::Error),

	/// Claims were missing from the response
	#[error("missing claims from response")]
	MissingClaims,
}

/// The response from Minecraft when attempting to authenticate with an xbox
/// token
#[derive(Deserialize, Serialize, Debug, Getters, CopyGetters, Clone)]
pub struct MinecraftAuthenticationResponse {
	/// UUID of the Xbox account.
	/// Please note that this is not the Minecraft player's UUID
	#[getset(get = "pub")]
	username: String,

	/// The minecraft JWT access token
	#[getset(get = "pub")]
	access_token: MinecraftAccessToken,

	/// The type of access token
	#[getset(get = "pub")]
	token_type: MinecraftTokenType,

	/// How many seconds until the token expires
	#[getset(get_copy = "pub")]
	expires_in: u32,
}

/// The response from Xbox when authenticating with a Microsoft token
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct XboxLiveAuthenticationResponse {
	/// The xbox authentication token to use
	token: String,

	/// An object that contains a vec of `uhs` objects
	/// Looks like { "xui": [{"uhs": "xbl_token"}] }
	display_claims: HashMap<String, Vec<HashMap<String, String>>>,
}

/// The flow for authenticating with a Microsoft access token and getting a
/// Minecraft access token.
pub struct MinecraftAuthorizationFlow {
	http_client: Client,
}

impl MinecraftAuthorizationFlow {
	/// Creates a new [MinecraftAuthorizationFlow] using the given
	/// [Client].
	pub const fn new(http_client: Client) -> Self {
		Self { http_client }
	}

	/// Authenticates with the Microsoft identity platform using the given
	/// Microsoft access token and returns a [MinecraftAuthenticationResponse]
	/// that contains the Minecraft access token.
	pub async fn exchange_microsoft_token(
		&self,
		microsoft_access_token: impl AsRef<str>,
	) -> Result<MinecraftAuthenticationResponse, MinecraftAuthorizationError> {
		let (xbox_token, user_hash) = self.xbox_token(microsoft_access_token).await?;
		let xbox_security_token = self.xbox_security_token(xbox_token).await?;

		let response = self
			.http_client
			.post(MINECRAFT_LOGIN_WITH_XBOX)
			.json(&json!({
				"identityToken":
					format!(
						"XBL3.0 x={user_hash};{xsts_token}",
						user_hash = user_hash,
						xsts_token = xbox_security_token.token
					)
			}))
			.send()
			.await?;
		response.error_for_status_ref()?;

		let response = response.json().await?;
		Ok(response)
	}

	async fn xbox_security_token(
		&self,
		xbox_token: String,
	) -> Result<XboxLiveAuthenticationResponse, MinecraftAuthorizationError> {
		let response = self
			.http_client
			.post(XBOX_XSTS_AUTHORIZE)
			.json(&json!({
				"Properties": {
					"SandboxId": "RETAIL",
					"UserTokens": [xbox_token]
				},
				"RelyingParty": "rp://api.minecraftservices.com/",
				"TokenType": "JWT"
			}))
			.send()
			.await?;
		response.error_for_status_ref()?;
		let xbox_security_token_resp: XboxLiveAuthenticationResponse = response.json().await?;
		Ok(xbox_security_token_resp)
	}

	async fn xbox_token(
		&self,
		microsoft_access_token: impl AsRef<str>,
	) -> Result<(String, String), MinecraftAuthorizationError> {
		let xbox_authenticate_json = json!({
			"Properties": {
				"AuthMethod": "RPS",
				"SiteName": "user.auth.xboxlive.com",
				"RpsTicket": &format!("d={}", microsoft_access_token.as_ref())
			},
			"RelyingParty": "http://auth.xboxlive.com",
			"TokenType": "JWT"
		});
		let response = self
			.http_client
			.post(XBOX_USER_AUTHENTICATE)
			.json(&xbox_authenticate_json)
			.send()
			.await?;
		response.error_for_status_ref()?;
		let xbox_resp: XboxLiveAuthenticationResponse = response.json().await?;
		let xbox_token = xbox_resp.token;
		let user_hash = xbox_resp
			.display_claims
			.get("xui")
			.ok_or(MinecraftAuthorizationError::MissingClaims)?
			.get(0)
			.ok_or(MinecraftAuthorizationError::MissingClaims)?
			.get("uhs")
			.ok_or(MinecraftAuthorizationError::MissingClaims)?
			.to_owned();
		Ok((xbox_token, user_hash))
	}
}
