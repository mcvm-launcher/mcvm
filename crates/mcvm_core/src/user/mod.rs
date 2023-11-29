/// Authentication for different types of user accounts
pub mod auth;
/// Tools for working with UUIDs
pub mod uuid;

use std::collections::HashMap;

use mcvm_shared::output::MCVMOutput;
use oauth2::ClientId;
use reqwest::Client;

use crate::net::minecraft::Keypair;

/// A user account that can play the game
#[derive(Debug, Clone)]
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

/// List of users and AuthState
#[derive(Debug, Clone)]
pub struct UserManager {
	/// The current state of authentication
	pub state: AuthState,
	/// All configured / available users
	users: HashMap<String, User>,
	/// The MS client ID
	ms_client_id: ClientId,
}

/// State of authentication
#[derive(Debug, Clone)]
pub enum AuthState {
	/// No user is picked / MCVM is offline
	Offline,
	/// A user has been selected but not authenticated
	UserChosen(String),
	/// The user is authenticated
	Authed(String),
}

impl UserManager {
	/// Create a new UserManager
	pub fn new(ms_client_id: ClientId) -> Self {
		Self {
			state: AuthState::Offline,
			users: HashMap::new(),
			ms_client_id,
		}
	}

	/// Add a new user to the manager
	pub fn add_user(&mut self, user: User) {
		self.add_user_with_id(user.id.clone(), user);
	}

	/// Add a new user to the manager with a different
	/// ID than the user struct has. I don't know why you would need to do this,
	/// but it's an option anyways
	pub fn add_user_with_id(&mut self, user_id: String, user: User) {
		self.users.insert(user_id, user);
	}

	/// Get a user from the manager
	pub fn get_user(&self, user_id: &str) -> Option<&User> {
		self.users.get(user_id)
	}

	/// Get a user from the manager mutably
	pub fn get_user_mut(&mut self, user_id: &str) -> Option<&mut User> {
		self.users.get_mut(user_id)
	}

	/// Checks if a user with an ID exists
	pub fn user_exists(&self, user_id: &str) -> bool {
		self.users.contains_key(user_id)
	}

	/// Iterate over users and their IDs
	pub fn iter_users(&self) -> impl Iterator<Item = (&String, &User)> {
		self.users.iter()
	}

	/// Get the currently chosen user, if there is one
	pub fn get_chosen_user(&self) -> Option<&User> {
		match &self.state {
			AuthState::Offline => None,
			AuthState::UserChosen(user_id) => self.users.get(user_id),
			AuthState::Authed(user_id) => self.users.get(user_id),
		}
	}

	/// Ensures that the currently chosen user is authenticated
	pub async fn ensure_authenticated(
		&mut self,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		if let AuthState::UserChosen(user) = &self.state {
			let user = self
				.users
				.get_mut(user)
				.expect("User in AuthState does not exist");
			dbg!(&self.ms_client_id);
			user.authenticate(self.ms_client_id.clone(), client, o)
				.await?;
		}

		Ok(())
	}

	/// Adds users from another UserManager
	pub fn steal_users(&mut self, other: &Self) {
		self.users.extend(other.users.clone());
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
