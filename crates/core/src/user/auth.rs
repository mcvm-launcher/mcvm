use std::fmt::Debug;

use anyhow::Context;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use oauth2::ClientId;

use crate::net::minecraft;
use crate::net::minecraft::MinecraftUserProfile;
use mcvm_auth::mc::{self as auth, mc_access_token_to_string};

use super::{User, UserKind};

impl User {
	/// Authenticate the user
	pub(super) async fn authenticate(
		&mut self,
		client_id: ClientId,
		client: &reqwest::Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		match &mut self.kind {
			UserKind::Microsoft { xbox_uid } => {
				let auth_result = authenticate_microsoft_user(client_id, client, o)
					.await
					.context("Failed to authenticate user")?;
				let certificate = crate::net::minecraft::get_user_certificate(
					&auth_result.access_token.0,
					client,
				)
				.await
				.context("Failed to get user certificate")?;
				self.access_token = Some(auth_result.access_token);
				self.name = auth_result.profile.name;
				self.uuid = Some(auth_result.profile.uuid);
				self.keypair = Some(certificate.key_pair);
				*xbox_uid = Some(auth_result.xbox_uid);
			}
			UserKind::Demo | UserKind::Unverified => {}
		}

		Ok(())
	}
}

/// Authenticate a Microsoft user using Microsoft OAuth
pub async fn authenticate_microsoft_user(
	client_id: ClientId,
	client: &reqwest::Client,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<MicrosoftAuthResult> {
	let oauth_client = auth::create_client(client_id).context("Failed to create OAuth client")?;
	let response = auth::generate_login_page(&oauth_client)
		.await
		.context("Failed to execute authorization and generate login page")?;

	o.display_special_ms_auth(response.verification_uri(), response.user_code().secret());

	let token = auth::get_microsoft_token(&oauth_client, response)
		.await
		.context("Failed to get Microsoft token")?;
	let mc_token = auth::auth_minecraft(token, client)
		.await
		.context("Failed to get Minecraft token")?;
	let access_token = mc_access_token_to_string(&mc_token.access_token);

	let profile = minecraft::get_user_profile(&access_token, client)
		.await
		.context("Failed to get user profile")?;

	o.display(
		MessageContents::Success("Authentication successful".into()),
		MessageLevel::Important,
	);

	let out = MicrosoftAuthResult {
		access_token: AccessToken(access_token),
		profile,
		xbox_uid: mc_token.username.clone(),
	};

	Ok(out)
}

/// Authenticate with lots of prints; used for debugging
pub async fn debug_authenticate(
	client_id: ClientId,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<()> {
	println!("Note: This authentication is not complete and is for debug purposes only");
	println!("Client ID: {}", client_id.as_str());
	let client = auth::create_client(client_id).context("Failed to create OAuth client")?;
	let req_client = reqwest::Client::new();
	let response = auth::generate_login_page(&client)
		.await
		.context("Failed to execute authorization and generate login page")?;

	o.display_special_ms_auth(response.verification_uri(), response.user_code().secret());

	let token = auth::get_microsoft_token(&client, response)
		.await
		.context("Failed to get Microsoft token")?;

	println!("Microsoft token: {token:?}");

	let mc_token = auth::auth_minecraft(token, &req_client)
		.await
		.context("Failed to get Minecraft token")?;

	println!("Minecraft token: {mc_token:?}");

	let access_token = mc_access_token_to_string(&mc_token.access_token);
	println!("Minecraft Access Token: {access_token}");

	let profile = minecraft::get_user_profile(&access_token, &req_client)
		.await
		.context("Failed to get user profile")?;
	println!("Profile: {profile:?}");

	Ok(())
}

/// Result from the Microsoft authentication function
pub struct MicrosoftAuthResult {
	/// The access token for logging into the game and other API services
	pub access_token: AccessToken,
	/// The user's Minecraft profile
	pub profile: MinecraftUserProfile,
	/// The XBox UID of the user
	pub xbox_uid: String,
}

/// An access token for a user that will be hidden in debug messages
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AccessToken(pub String);

impl Debug for AccessToken {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "AccessToken(***)")
	}
}
