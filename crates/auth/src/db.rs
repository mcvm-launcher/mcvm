use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::{bail, ensure, Context};
use mcvm_shared::util::utc_timestamp;
use rsa::traits::PublicKeyParts;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};

use crate::mc::Keypair;
use crate::passkey::{decrypt_chunks, encrypt_chunks};

/// The amount of time to consider a refresh token valid for. We want it to expire eventually to
/// ensure some amount of security.
// 180 days
const REFRESH_TOKEN_EXPIRATION: u64 = 15552000;

/// A handle to the authentication database where things like credentials are stored
pub struct AuthDatabase {
	/// The directory where the database is stored
	dir: PathBuf,
	/// The contents of the main database file
	contents: DatabaseContents,
}

impl AuthDatabase {
	/// Open the database in the specified directory
	pub fn open(path: &Path) -> anyhow::Result<Self> {
		std::fs::create_dir_all(path).context("Failed to ensure database directory exists")?;
		let database_path = Self::get_db_path(path);
		let contents = if database_path.exists() {
			let file = File::open(&database_path).context("Failed to open database file")?;
			serde_json::from_reader(file).context("Failed to deserialize database contents")?
		} else {
			DatabaseContents::default()
		};

		let out = Self {
			dir: path.to_owned(),
			contents,
		};

		Ok(out)
	}

	/// Write the updated contents of the database handler to the database
	pub fn write(&self) -> anyhow::Result<()> {
		let path = Self::get_db_path(&self.dir);
		let file = File::create(path).context("Failed to create database file")?;
		serde_json::to_writer_pretty(file, &self.contents)
			.context("Failed to write database contents")?;

		Ok(())
	}

	/// Get the path to the main database file
	fn get_db_path(dir: &Path) -> PathBuf {
		dir.join("db.json")
	}

	/// Get whether a user in the database is still valid and logged in
	pub fn is_user_valid(&self, user_id: &str) -> bool {
		if let Some(user) = &self.contents.users.get(user_id) {
			let Ok(now) = utc_timestamp() else {
				return false;
			};

			now < user.expires
		} else {
			false
		}
	}

	/// Update a user
	pub fn update_user(&mut self, user: DatabaseUser, user_id: &str) -> anyhow::Result<()> {
		// Update the user
		self.contents.users.insert(user_id.to_string(), user);

		// Update the DB
		self.write().context("Failed to write to database")?;
		Ok(())
	}

	/// Removes a user from the database
	pub fn remove_user(&mut self, user_id: &str) -> anyhow::Result<()> {
		self.contents.users.remove(user_id);

		self.write().context("Failed to write to database")?;
		Ok(())
	}

	/// Logs out a user from the database by removing their sensitive data, but not their passkey or user
	pub fn logout_user(&mut self, user_id: &str) -> anyhow::Result<()> {
		if let Some(user) = self.contents.users.get_mut(user_id) {
			user.sensitive = SensitiveUserInfoSerialized::None;
		}

		Ok(())
	}

	/// Gets a user from the database, if it is present
	pub fn get_user(&self, user_id: &str) -> Option<&DatabaseUser> {
		self.contents.users.get(user_id)
	}

	/// Gets a user mutably from the database, if it is present
	pub fn get_user_mut(&mut self, user_id: &str) -> Option<&mut DatabaseUser> {
		self.contents.users.get_mut(user_id)
	}

	/// Gets a user, if it is present and valid
	pub fn get_valid_user(&self, user_id: &str) -> Option<&DatabaseUser> {
		if self.is_user_valid(user_id) {
			self.get_user(user_id)
		} else {
			None
		}
	}
}

/// Structure for the auth database
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
struct DatabaseContents {
	/// The currently held users
	users: HashMap<String, DatabaseUser>,
}

/// A user in the database
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatabaseUser {
	/// A unique ID for the user
	pub id: String,
	/// The username of the user
	pub username: String,
	/// The UUID of the user
	pub uuid: String,
	/// When the refresh token will expire, as a UTC timestamp in seconds
	pub expires: u64,
	/// Sensitive info for the user, serialized into a string and encoded
	/// using the public key
	pub sensitive: SensitiveUserInfoSerialized,
	/// Passkey information for the user
	pub passkey: Option<PasskeyInfo>,
}

impl DatabaseUser {
	/// Create a new database user with sensitive info
	pub fn new(
		id: String,
		username: String,
		uuid: String,
		expires: u64,
		sensitive: SensitiveUserInfo,
	) -> anyhow::Result<Self> {
		let mut out = DatabaseUser {
			id,
			username,
			uuid,
			expires,
			sensitive: SensitiveUserInfoSerialized::Encrypted(Vec::new()),
			passkey: None,
		};
		out.set_sensitive_info(sensitive)
			.context("Failed to set sensitive information for user in database")?;
		Ok(out)
	}

	/// Checks if the user has a passkey
	pub fn has_passkey(&self) -> bool {
		self.passkey.is_some()
	}

	/// Checks if the user is logged in, where their sensitive info is present
	pub fn is_logged_in(&self) -> bool {
		!matches!(self.sensitive, SensitiveUserInfoSerialized::None)
	}

	/// Get the user's private key from their passkey. Will fail if the passkey doesn't match
	/// and return none if the user doesn't have a passkey
	pub fn get_private_key(&self, passkey: &str) -> anyhow::Result<Option<RsaPrivateKey>> {
		if self.passkey.is_some() {
			let input_key = crate::passkey::generate_keys(passkey)
				.context("Failed to generate private key from input passkey")?;
			let expected_pub_key = self
				.get_public_key()
				.context("Failed to get stored public key")?
				.expect("Passkey info should be Some");
			ensure!(
				input_key.to_public_key() == expected_pub_key,
				"Passkey did not match"
			);
			Ok(Some(input_key))
		} else {
			Ok(None)
		}
	}

	/// Get the user's public key if they have one
	pub fn get_public_key(&self) -> anyhow::Result<Option<RsaPublicKey>> {
		if let Some(passkey_info) = &self.passkey {
			let key =
				hex::decode(&passkey_info.public_key).context("Failed to decode public key hex")?;
			let key = crate::passkey::recreate_public_key_bytes(&key)
				.context("Failed to recreate public key from stored data")?;
			Ok(Some(key))
		} else {
			Ok(None)
		}
	}

	/// Get the user's sensitive info if they don't have a passkey
	pub fn get_sensitive_info_no_passkey(&self) -> anyhow::Result<SensitiveUserInfo> {
		ensure!(
			self.passkey.is_none(),
			"User has a passkey that was not used"
		);
		let SensitiveUserInfoSerialized::Raw(raw) = &self.sensitive else {
			bail!("Sensitive info is encrypted, not raw");
		};
		Ok(raw.clone())
	}

	/// Get the user's sensitive info using their private key
	pub fn get_sensitive_info_with_key(
		&self,
		private_key: &RsaPrivateKey,
	) -> anyhow::Result<SensitiveUserInfo> {
		let SensitiveUserInfoSerialized::Encrypted(encrypted) = &self.sensitive else {
			bail!("Sensitive user info is raw or empty");
		};
		let mut hex_decoded = Vec::new();
		for chunk in encrypted {
			let decoded =
				hex::decode(chunk).context("Failed to deserialize hex of sensitive user info")?;
			hex_decoded.push(decoded);
		}
		let decoded = decrypt_chunks(&hex_decoded, private_key, Pkcs1v15Encrypt)
			.context("Failed to decrypt sensitive user info")?;
		let deserialized = serde_json::from_slice(&decoded)
			.context("Failed to deserialize sensitive user info")?;
		Ok(deserialized)
	}

	/// Set the user's sensitive info
	pub fn set_sensitive_info(&mut self, sensitive: SensitiveUserInfo) -> anyhow::Result<()> {
		if self.has_passkey() {
			let public_key = self
				.get_public_key()
				.context("Failed to get user public key")?
				.expect("User should have passkey");
			self.set_sensitive_info_impl(sensitive, &public_key)?;
		} else {
			self.sensitive = SensitiveUserInfoSerialized::Raw(sensitive);
		}

		Ok(())
	}

	/// Implementation for setting the user's sensitive info with the given public key
	fn set_sensitive_info_impl(
		&mut self,
		sensitive: SensitiveUserInfo,
		public_key: &RsaPublicKey,
	) -> anyhow::Result<()> {
		let serialized =
			serde_json::to_vec(&sensitive).context("Failed to serialize sensitive user info")?;
		let mut rng = rand::thread_rng();
		let encoded = encrypt_chunks(&serialized, public_key, &mut rng, Pkcs1v15Encrypt, 128)
			.context("Failed to encrypt sensitive user info")?;
		let mut hex_encoded = Vec::new();
		for chunk in encoded {
			let encoded = hex::encode(chunk);
			hex_encoded.push(encoded);
		}
		self.sensitive = SensitiveUserInfoSerialized::Encrypted(hex_encoded);
		Ok(())
	}

	/// Set the user's passkey and update their sensitive information
	pub fn update_passkey(
		&mut self,
		old_passkey: Option<&str>,
		passkey: &str,
	) -> anyhow::Result<()> {
		let old_private_key = if let Some(old_passkey) = old_passkey {
			Some(
				crate::passkey::generate_keys(old_passkey)
					.context("Failed to generate private key from old passkey")?,
			)
		} else {
			None
		};

		let private_key = crate::passkey::generate_keys(passkey)
			.context("Failed to generate private key from new passkey")?;
		let pub_key = private_key.to_public_key();

		// Update sensitive info

		// Get the current sensitive info
		let sensitive = if self.has_passkey() {
			let Some(old_private_key) = old_private_key else {
				bail!("No old passkey provided to update sensitive user data");
			};
			self.get_sensitive_info_with_key(&old_private_key)
		} else {
			self.get_sensitive_info_no_passkey()
		}
		.context("Failed to get existing sensitive user data")?;
		self.set_sensitive_info_impl(sensitive, &pub_key)
			.context("Failed to set new sensitive user data")?;

		// We only update the passkey now just in case one of the above operations failed
		let n = pub_key.n().to_bytes_le();
		let n = hex::encode(n);
		self.passkey = Some(PasskeyInfo { public_key: n });

		Ok(())
	}
}

/// Sensitive info for a user that is encoded in a string
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SensitiveUserInfo {
	/// The refresh token for the user
	pub refresh_token: Option<String>,
	/// The Xbox uid of the user, if applicable
	pub xbox_uid: Option<String>,
	/// The keypair of the user, if applicable
	pub keypair: Option<Keypair>,
	/// The Minecraft access token
	pub access_token: Option<String>,
	/// When the access token expires
	pub access_token_expires: Option<u64>,
}

/// Passkey information in the database
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PasskeyInfo {
	/// The public key that was derived from the passkey, as a hex string
	pub public_key: String,
}

/// Sensitive user data serialization format
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum SensitiveUserInfoSerialized {
	/// No info
	None,
	/// Raw info with no passkey encryption
	Raw(SensitiveUserInfo),
	/// Info encrypted with a passkey, as chunks of key-encoded hex strings
	Encrypted(Vec<String>),
}

/// Calculate the date to expire the refresh token at
pub fn calculate_expiration_date() -> u64 {
	let now = utc_timestamp().unwrap_or_default();
	now + REFRESH_TOKEN_EXPIRATION
}
