use anyhow::{bail, Context};
use mcvm_shared::output::{MCVMOutput, MessageContents};

use crate::net::minecraft::MinecraftUserProfile;
use crate::Paths;
use mcvm_auth::db::{AuthDatabase, DatabaseUser, SensitiveUserInfo};
use mcvm_auth::mc::Keypair;
use mcvm_auth::mc::{
	self as auth, authenticate_microsoft_user, authenticate_microsoft_user_from_token, AccessToken,
	ClientId, RefreshToken,
};

use super::{User, UserKind};

impl User {
	/// Authenticate the user
	pub async fn authenticate(
		&mut self,
		force: bool,
		client_id: ClientId,
		paths: &Paths,
		client: &reqwest::Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		match &mut self.kind {
			UserKind::Microsoft { xbox_uid } => {
				let user_data =
					update_microsoft_user_auth(&self.id, force, client_id, paths, client, o)
						.await
						.context("Failed to update user authentication")?;

				self.access_token = Some(user_data.access_token);
				self.name = Some(user_data.profile.name);
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

				db.get_valid_user(&self.id).is_some()
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

	/// Updates this user's passkey using prompts
	pub fn update_passkey(&self, paths: &Paths, o: &mut impl MCVMOutput) -> anyhow::Result<()> {
		let mut db =
			AuthDatabase::open(&paths.auth).context("Failed to open authentication database")?;
		let user = db.get_user_mut(&self.id).context(
			"User does not exist in database. Try authenticating first before setting a passkey",
		)?;
		let old_passkey = if user.has_passkey() {
			Some(
				o.prompt_password(MessageContents::Simple(format!(
					"Enter the old passkey for user '{}'",
					self.id
				)))
				.context("Failed to get old passkey")?,
			)
		} else {
			None
		};
		let new_passkey = o
			.prompt_new_password(MessageContents::Simple(format!(
				"Enter the new passkey for user '{}'",
				self.id
			)))
			.context("Failed to get new passkey")?;
		user.update_passkey(old_passkey.as_deref(), &new_passkey)
			.context("Failed to update passkey for user")?;

		db.write()
			.context("Failed to write to authentication database")?;

		Ok(())
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
	force: bool,
	client_id: ClientId,
	paths: &Paths,
	client: &reqwest::Client,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<MicrosoftUserData> {
	let mut db =
		AuthDatabase::open(&paths.auth).context("Failed to open authentication database")?;

	// Force reauth if specified
	if force {
		return reauth_microsoft_user(user_id, &mut db, client_id, client, o).await;
	}

	// Check the authentication DB
	let user_data = if let Some((db_user, sensitive)) =
		get_full_user(&db, user_id, o).context("Failed to get full user from database")?
	{
		let refresh_token = RefreshToken::new(
			sensitive
				.refresh_token
				.expect("Refresh token should be present in a full valid user"),
		);
		// Get the access token using the refresh token
		let oauth_client =
			auth::create_client(client_id).context("Failed to create OAuth client")?;
		let token = auth::refresh_microsoft_token(&oauth_client, &refresh_token)
			.await
			.context("Failed to get refreshed token")?;

		let token = authenticate_microsoft_user_from_token(token, client, o)
			.await
			.context("Failed to authenticate with refreshed token")?;

		MicrosoftUserData {
			access_token: AccessToken(token.access_token.0.clone()),
			profile: MinecraftUserProfile {
				name: db_user.username.clone(),
				uuid: db_user.uuid.clone(),
				skins: Vec::new(),
				capes: Vec::new(),
			},
			xbox_uid: sensitive.xbox_uid.clone(),
			keypair: sensitive.keypair.clone(),
		}
	} else {
		// Authenticate with the server again
		reauth_microsoft_user(user_id, &mut db, client_id, client, o).await?
	};

	Ok(user_data)
}

async fn reauth_microsoft_user(
	user_id: &str,
	db: &mut AuthDatabase,
	client_id: ClientId,
	client: &reqwest::Client,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<MicrosoftUserData> {
	let auth_result = authenticate_microsoft_user(client_id, client, o)
		.await
		.context("Failed to authenticate user")?;

	let ownership_task = {
		let client = client.clone();
		let token = auth_result.access_token.0.clone();
		async move {
			let owns_game = auth::account_owns_game(&token, &client)
				.await
				.context("Failed to check for game ownership")?;

			Ok::<bool, anyhow::Error>(owns_game)
		}
	};

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

	let (owns_game, profile, certificate) =
		tokio::try_join!(ownership_task, profile_task, certificate_task)?;

	if !owns_game {
		bail!("Specified account does not own Minecraft");
	}

	// Calculate expiration time
	let expiration_time = mcvm_auth::db::calculate_expiration_date();

	// Write the new user to the database

	let sensitive = SensitiveUserInfo {
		refresh_token: auth_result.refresh_token.map(|x| x.secret().clone()),
		xbox_uid: Some(auth_result.xbox_uid.clone()),
		keypair: Some(certificate.key_pair.clone()),
	};
	let db_user = DatabaseUser::new(
		user_id.to_string(),
		profile.name.clone(),
		profile.uuid.clone(),
		expiration_time,
		sensitive,
	)
	.context("Failed to create new user in database")?;

	db.update_user(db_user, user_id)
		.context("Failed to update user in database")?;

	Ok(MicrosoftUserData {
		access_token: auth_result.access_token,
		xbox_uid: Some(auth_result.xbox_uid),
		profile,
		keypair: Some(certificate.key_pair),
	})
}

/// Tries to get a full valid user from the database along with a passkey prompt if applicable
fn get_full_user<'db>(
	db: &'db AuthDatabase,
	user_id: &str,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<Option<(&'db DatabaseUser, SensitiveUserInfo)>> {
	let Some(user) = db.get_valid_user(user_id) else {
		return Ok(None);
	};
	// Get their sensitive info
	let sensitive = if user.has_passkey() {
		let passkey = o
			.prompt_password(MessageContents::Simple(format!(
				"Please enter the passkey for the user '{user_id}'"
			)))
			.context("Passkey prompt failed")?;
		let private_key = user
			.get_private_key(&passkey)
			.context("Failed to get user private key")?
			.expect("User should have passkey");
		user.get_sensitive_info_with_key(&private_key)
			.context("Failed to get sensitive user info using key")?
	} else {
		user.get_sensitive_info_no_passkey()
			.context("Failed to get sensitive user info without key")?
	};
	if sensitive.refresh_token.is_none() {
		return Ok(None);
	}

	Ok(Some((user, sensitive)))
}
