use anyhow::{bail, Context};
use mcvm_auth::RsaPrivateKey;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::translate;

use crate::net::minecraft::MinecraftUserProfile;
use crate::Paths;
use mcvm_auth::db::{AuthDatabase, DatabaseUser, SensitiveUserInfo};
use mcvm_auth::mc::Keypair;
use mcvm_auth::mc::{
	self as auth, authenticate_microsoft_user, authenticate_microsoft_user_from_token, AccessToken,
	ClientId, RefreshToken,
};

use super::{CustomAuthFunction, User, UserKind};

impl User {
	/// Authenticate the user
	pub(crate) async fn authenticate(
		&mut self,
		params: AuthParameters<'_>,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		match &mut self.kind {
			UserKind::Microsoft { xbox_uid } => {
				if params.offline {
					let db = AuthDatabase::open(&params.paths.auth)
						.context("Failed to open authentication database")?;
					let Some((user, sensitive)) = get_full_user(&db, &self.id, o)
						.await
						.context("Failed to get user from database")?
					else {
						bail!("User not present in database. Make sure to authenticate at least once before logging in in offline mode");
					};

					self.name = Some(user.username.clone());
					self.uuid = Some(user.uuid.clone());
					self.keypair = sensitive.keypair.clone();
					*xbox_uid = sensitive.xbox_uid.clone();
				} else {
					let user_data = update_microsoft_user_auth(&self.id, params, o)
						.await
						.context("Failed to update user authentication")?;

					self.access_token = Some(user_data.access_token);
					self.name = Some(user_data.profile.name);
					self.uuid = Some(user_data.profile.uuid);
					self.keypair = user_data.keypair;
					*xbox_uid = user_data.xbox_uid;
				}
			}
			UserKind::Demo => {}
			UserKind::Unknown(other) => {
				if let Some(func) = params.custom_auth_fn {
					o.display(
						MessageContents::Simple(
							"Handling custom user type with authentication function".into(),
						),
						MessageLevel::Debug,
					);
					let profile = func(&self.id, other).context("Custom auth function failed")?;
					if let Some(profile) = profile {
						self.name = Some(profile.name);
						self.uuid = Some(profile.uuid);
					}
				} else {
					o.display(
						MessageContents::Simple(
							"Authentication for custom user type not handled".into(),
						),
						MessageLevel::Debug,
					);
				}
			}
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
			UserKind::Demo => true,
			UserKind::Unknown(..) => true,
		}
	}

	/// Checks if this user is currently authenticated and ready to be used
	pub fn is_authenticated(&self) -> bool {
		match &self.kind {
			UserKind::Microsoft { .. } => self.access_token.is_some() && self.uuid.is_some(),
			UserKind::Demo => true,
			UserKind::Unknown(..) => true,
		}
	}

	/// Updates this user's passkey using prompts
	pub async fn update_passkey(
		&self,
		paths: &Paths,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
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
				.await
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
			.await
			.context("Failed to get new passkey")?;
		user.update_passkey(old_passkey.as_deref(), &new_passkey)
			.context("Failed to update passkey for user")?;

		db.write()
			.context("Failed to write to authentication database")?;

		Ok(())
	}

	/// Logs out this user and removes their data from the auth database (not including passkey)
	pub fn logout(&mut self, paths: &Paths) -> anyhow::Result<()> {
		let mut db =
			AuthDatabase::open(&paths.auth).context("Failed to open authentication database")?;
		db.logout_user(&self.id)
			.context("Failed to logout user in database")?;

		db.write()
			.context("Failed to write authentication database")?;

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
async fn update_microsoft_user_auth(
	user_id: &str,
	params: AuthParameters<'_>,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<MicrosoftUserData> {
	let mut db =
		AuthDatabase::open(&params.paths.auth).context("Failed to open authentication database")?;

	// Force reauth if specified
	if params.force {
		return reauth_microsoft_user(user_id, &mut db, params.client_id, params.req_client, o)
			.await;
	}

	// Check the authentication DB
	let user_data = if let Some((db_user, sensitive)) = get_full_user(&db, user_id, o)
		.await
		.context("Failed to get full user from database")?
	{
		let refresh_token = RefreshToken::new(
			sensitive
				.refresh_token
				.expect("Refresh token should be present in a full valid user"),
		);
		// Get the access token using the refresh token
		let oauth_client =
			auth::create_client(params.client_id).context("Failed to create OAuth client")?;
		let token = auth::refresh_microsoft_token(&oauth_client, &refresh_token)
			.await
			.context("Failed to get refreshed token")?;

		let token = authenticate_microsoft_user_from_token(token, params.req_client, o)
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
		reauth_microsoft_user(user_id, &mut db, params.client_id, params.req_client, o).await?
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
async fn get_full_user<'db>(
	db: &'db AuthDatabase,
	user_id: &str,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<Option<(&'db DatabaseUser, SensitiveUserInfo)>> {
	let Some(user) = db.get_valid_user(user_id) else {
		return Ok(None);
	};
	// We have to reauthenticate non-logged-in users
	if !user.is_logged_in() {
		return Ok(None);
	}

	// Get their sensitive info
	let sensitive = if user.has_passkey() {
		let private_key = get_private_key(
			user,
			MessageContents::Simple(format!("Please enter the passkey for the user '{user_id}'")),
			o,
		)
		.await
		.context("Failed to get key")?;

		let out = user
			.get_sensitive_info_with_key(&private_key)
			.context("Failed to get sensitive user info using key")?;
		o.display(
			MessageContents::Success(translate!(o, PasskeyAccepted)),
			MessageLevel::Important,
		);

		out
	} else {
		user.get_sensitive_info_no_passkey()
			.context("Failed to get sensitive user info without key")?
	};
	if sensitive.refresh_token.is_none() {
		return Ok(None);
	}

	Ok(Some((user, sensitive)))
}

/// Gets the user's private key with a repeating passkey prompt.
/// The user must have a passkey available.
async fn get_private_key(
	user: &DatabaseUser,
	message: MessageContents,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<RsaPrivateKey> {
	const MAX_ATTEMPTS: u8 = 3;

	for _ in 0..MAX_ATTEMPTS {
		let result = o
			.prompt_special_user_passkey(message.clone(), &user.id)
			.await;
		if let Ok(passkey) = result {
			let result = user.get_private_key(&passkey);
			match result {
				Ok(private_key) => {
					return Ok(private_key.expect("User should have passkey"));
				}
				Err(e) => {
					o.display(
						MessageContents::Error(format!("{e:?}")),
						MessageLevel::Important,
					);
				}
			}
		}
	}

	bail!("Passkey authentication failed; max attempts exceeded")
}

/// Container struct for parameters for authenticating a user
pub(crate) struct AuthParameters<'a> {
	pub force: bool,
	pub offline: bool,
	pub client_id: ClientId,
	pub paths: &'a Paths,
	pub req_client: &'a reqwest::Client,
	pub custom_auth_fn: Option<CustomAuthFunction>,
}
