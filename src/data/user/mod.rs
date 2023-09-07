/// Authentication for different types of user accounts
pub mod auth;
/// Tools for working with UUIDs
pub mod uuid;

use std::collections::HashMap;

use mcvm_shared::output::MCVMOutput;
use oauth2::ClientId;
use reqwest::Client;

use crate::net::microsoft::Keypair;

/// Type of a user
#[derive(Debug, Clone)]
pub enum UserKind {
	/// A new Microsoft user, the standard account
	Microsoft {
		/// The XBox UID of the user
		xbox_uid: Option<String>,
	},
	/// A demo user
	Demo,
	/// An unverified / not logged in user
	Unverified,
}

/// A user account that can play the game
#[derive(Debug)]
pub struct User {
	/// Type of this user
	pub kind: UserKind,
	/// This user's ID
	pub id: String,
	/// The user's username
	pub name: String,
	/// The user's UUID
	pub uuid: Option<String>,
	/// The user's access token
	pub access_token: Option<String>,
	/// The user's public / private key pair
	pub keypair: Option<Keypair>,
}

impl User {
	/// Create a new user
	pub fn new(kind: UserKind, id: &str, name: &str) -> Self {
		Self {
			kind,
			id: id.to_string(),
			name: name.to_string(),
			uuid: None,
			access_token: None,
			keypair: None,
		}
	}

	/// Set this user's UUID
	pub fn set_uuid(&mut self, uuid: &str) {
		self.uuid = Some(uuid.to_string());
	}
}

/// State of authentication
#[derive(Debug)]
pub enum AuthState {
	/// No user is picked / MCVM is offline
	Offline,
	/// A user has been selected but not authenticated
	UserChosen(String),
	/// The user is authenticated
	Authed(String),
}

/// List of users and AuthState
#[derive(Debug)]
pub struct UserManager {
	/// The current state of authentication
	pub state: AuthState,
	/// All configured / available users
	pub users: HashMap<String, User>,
}

impl UserManager {
	/// Create a new UserManager
	pub fn new() -> Self {
		Self {
			state: AuthState::Offline,
			users: HashMap::new(),
		}
	}

	/// Get the currently chosen user, if there is one
	pub fn get_user(&self) -> Option<&User> {
		match &self.state {
			AuthState::Offline => None,
			AuthState::UserChosen(user_id) => self.users.get(user_id),
			AuthState::Authed(user_id) => self.users.get(user_id),
		}
	}

	/// Ensures that the currently chosen user is authenticated
	pub async fn ensure_authenticated(
		&mut self,
		client_id: ClientId,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		if let AuthState::UserChosen(user) = &self.state {
			let user = self
				.users
				.get_mut(user)
				.expect("User in AuthState does not exist");
			user.authenticate(client_id, client, o).await?;
		}

		Ok(())
	}
}

impl Default for UserManager {
	fn default() -> Self {
		Self::new()
	}
}

/// Validate a Minecraft username
pub fn validate_username(_kind: &UserKind, name: &str) -> bool {
	if name.is_empty() || name.len() > 16 {
		return false;
	}

	for c in name.chars() {
		if !c.is_ascii_alphanumeric() && c != '_' {
			return false;
		}
	}

	true
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_username_validation() {
		assert!(validate_username(
			&UserKind::Microsoft { xbox_uid: None },
			"CarbonSmasher"
		));
		assert!(validate_username(&UserKind::Demo, "12345"));
		assert!(validate_username(
			&UserKind::Microsoft { xbox_uid: None },
			"Foo_Bar888"
		));
		assert!(!validate_username(
			&UserKind::Microsoft { xbox_uid: None },
			""
		));
		assert!(!validate_username(
			&UserKind::Microsoft { xbox_uid: None },
			"ABCDEFGHIJKLMNOPQRS"
		));
		assert!(!validate_username(
			&UserKind::Microsoft { xbox_uid: None },
			"+++"
		));
	}
}
