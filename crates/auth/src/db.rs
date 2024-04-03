use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::Context;
use mcvm_shared::util::utc_timestamp;
use serde::{Deserialize, Serialize};

use crate::mc::Keypair;

/// The buffer time in seconds before a token actually expires to still consider it expired
/// because it won't be valid for very long
const EXPIRATION_BUFFER: u64 = 120;

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

	/// Get whether the current user is valid
	pub fn is_user_valid(&self) -> bool {
		if let Some(user) = &self.contents.user {
			let Ok(now) = utc_timestamp() else {
				return false;
			};

			// Handle overflow
			if user.expires < EXPIRATION_BUFFER {
				return false;
			}

			now < (user.expires - EXPIRATION_BUFFER)
		} else {
			false
		}
	}

	/// Update the current user
	pub fn update_user(&mut self, user: DatabaseUser) -> anyhow::Result<()> {
		// Update the user
		self.contents.user = Some(user);

		// Update the DB
		self.write().context("Failed to write to database")?;
		Ok(())
	}

	/// Remove the current user
	pub fn remove_user(&mut self) -> anyhow::Result<()> {
		self.contents.user = None;

		self.write().context("Failed to write to database")?;
		Ok(())
	}

	/// Get the current user, if it is present
	pub fn get_user(&self) -> Option<&DatabaseUser> {
		self.contents.user.as_ref()
	}

	/// Get the current user, if it is present and valid
	pub fn get_valid_user(&self) -> Option<&DatabaseUser> {
		if self.is_user_valid() {
			self.get_user()
		} else {
			None
		}
	}
}

/// Structure for the auth database
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
struct DatabaseContents {
	/// The currently held user
	user: Option<DatabaseUser>,
	/// Passkey information
	passkey: Option<PasskeyInfo>,
}

/// A user in the database
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DatabaseUser {
	/// A unique ID for the user
	pub id: String,
	/// The username of the user
	pub username: String,
	/// The UUID of the user
	pub uuid: String,
	/// The refresh token for the user
	pub refresh_token: Option<String>,
	/// When the refresh token will expire, as a UTC timestamp in seconds
	pub expires: u64,
	/// The Xbox uid of the user, if applicable
	pub xbox_uid: Option<String>,
	/// The keypair of the user, if applicable
	pub keypair: Option<Keypair>,
}

/// Passkey information in the database
#[derive(Serialize, Deserialize, Debug)]
pub struct PasskeyInfo {
	/// The public key that was derived from the passkey
	pub public_key: String,
	/// An encrypted check string used to make sure that the correct passkey was used
	pub check: String,
}

/// Calculate the date to expire the refresh token at
pub fn calculate_expiration_date() -> u64 {
	let now = utc_timestamp().unwrap_or_default();
	now + REFRESH_TOKEN_EXPIRATION
}
