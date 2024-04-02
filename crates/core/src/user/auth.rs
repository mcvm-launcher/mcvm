use std::fmt::Debug;

use anyhow::Context;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::util::utc_timestamp;
use oauth2::ClientId;

use crate::net::minecraft::MinecraftUserProfile;
use crate::{net::minecraft, Paths};
use mcvm_auth::db::{AuthDatabase, DatabaseUser};
use mcvm_auth::mc as auth;
use mcvm_auth::mc::{mc_access_token_to_string, Keypair};

use super::{User, UserKind};

impl User {
	/// Authenticate the user
	pub(super) async fn authenticate(
		&mut self,
		client_id: ClientId,
		paths: &Paths,
		client: &reqwest::Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		match &mut self.kind {
			UserKind::Microsoft { xbox_uid } => {
				let user_data = update_microsoft_user_auth(&self.id, client_id, paths, client, o)
					.await
					.context("Failed to update user authentication")?;

				self.access_token = Some(user_data.access_token);
				self.name = user_data.profile.name;
				self.uuid = Some(user_data.profile.uuid);
				self.keypair = user_data.keypair;
				*xbox_uid = user_data.xbox_uid;
			}
			UserKind::Demo | UserKind::Unverified => {}
		}

		Ok(())
	}

	/// Checks if the user still has valid authentication. This does not mean that they are
	/// authenticated yet. To check if the user is authenticated and ready to be used, use the is_authenticated
	/// function instead.
	pub fn is_auth_valid(&self, paths: &Paths) -> bool {
		match &self.kind {
			UserKind::Microsoft { .. } => {
				let Ok(db) = AuthDatabase::open(&paths.auth) else {
					return false;
				};

				if let Some(user) = db.get_valid_user() {
					user.id == self.id
				} else {
					false
				}
			}
			UserKind::Demo | UserKind::Unverified => true,
		}
	}

	/// Checks if this user is currently authenticated and ready to be used
	pub fn is_authenticated(&self) -> bool {
		match &self.kind {
			UserKind::Microsoft { .. } => self.access_token.is_some() && self.uuid.is_some(),
			UserKind::Demo | UserKind::Unverified => true,
		}
	}
}

/// Data for a Microsoft user
pub struct MicrosoftUserData {
	access_token: AccessToken,
	profile: MinecraftUserProfile,
	xbox_uid: Option<String>,
	keypair: Option<Keypair>,
}

/// Updates authentication for a Microsoft user using either the database or updating from the API
pub async fn update_microsoft_user_auth(
	user_id: &str,
	client_id: ClientId,
	paths: &Paths,
	client: &reqwest::Client,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<MicrosoftUserData> {
	// Check the authentication DB
	let mut db =
		AuthDatabase::open(&paths.auth).context("Failed to open authentication database")?;
	let user_data = if let Some(db_user) = db.get_valid_user() {
		MicrosoftUserData {
			access_token: AccessToken(db_user.token.clone()),
			profile: MinecraftUserProfile {
				name: db_user.username.clone(),
				uuid: db_user.uuid.clone(),
				skins: Vec::new(),
				capes: Vec::new(),
			},
			xbox_uid: db_user.xbox_uid.clone(),
			keypair: db_user.keypair.clone(),
		}
	} else {
		// Authenticate with the server again
		let auth_result = authenticate_microsoft_user(client_id, client, o)
			.await
			.context("Failed to authenticate user")?;

		let profile_task = {
			let client = client.clone();
			let token = auth_result.access_token.0.clone();
			async move {
				let profile = crate::net::minecraft::get_user_profile(&token, &client)
					.await
					.context("Failed to get Microsoft user profile")?;

				Ok::<MinecraftUserProfile, anyhow::Error>(profile)
			}
		};

		let certificate_task = {
			let client = client.clone();
			let token = auth_result.access_token.0.clone();
			async move {
				let certificate = crate::net::minecraft::get_user_certificate(&token, &client)
					.await
					.context("Failed to get user certificate")?;

				Ok(certificate)
			}
		};

		let (profile, certificate) = tokio::try_join!(profile_task, certificate_task)?;

		// Calculate expiration time
		let now = utc_timestamp().context("Failed to get current timestamp")?;
		let expiration_time = now + auth_result.expires_in;

		// Write the new user to the database
		let db_user = DatabaseUser {
			id: user_id.to_string(),
			username: profile.name.clone(),
			uuid: profile.uuid.clone(),
			token: auth_result.access_token.0.clone(),
			expires: expiration_time,
			xbox_uid: Some(auth_result.xbox_uid.clone()),
			keypair: Some(certificate.key_pair.clone()),
		};

		db.update_user(db_user)
			.context("Failed to update user in database")?;

		MicrosoftUserData {
			access_token: auth_result.access_token,
			xbox_uid: Some(auth_result.xbox_uid),
			profile,
			keypair: Some(certificate.key_pair),
		}
	};

	Ok(user_data)
}

/// Authenticate a Microsoft user using Microsoft OAuth.
/// Will authenticate every time and will not use the database.
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

	let expires_in = response.expires_in().as_secs();

	let token = auth::get_microsoft_token(&oauth_client, response)
		.await
		.context("Failed to get Microsoft token")?;
	let mc_token = auth::auth_minecraft(token, client)
		.await
		.context("Failed to get Minecraft token")?;
	let access_token = mc_access_token_to_string(&mc_token.access_token);

	o.display(
		MessageContents::Success("Authentication successful".into()),
		MessageLevel::Important,
	);

	let out = MicrosoftAuthResult {
		access_token: AccessToken(access_token),
		xbox_uid: mc_token.username.clone(),
		expires_in,
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
	/// The Xbox UID of the user
	pub xbox_uid: String,
	/// The amount of time in seconds before the token expires
	pub expires_in: u64,
}

/// An access token for a user that will be hidden in debug messages
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AccessToken(pub String);

impl Debug for AccessToken {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "AccessToken(***)")
	}
}
