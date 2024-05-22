/// Authentication for different types of user accounts
pub mod auth;
/// Tools for working with UUIDs
pub mod uuid;

use std::{collections::HashMap, ops::Deref, sync::Arc};

use anyhow::bail;
use mcvm_auth::mc::{AccessToken, ClientId, Keypair};
use mcvm_shared::output::MCVMOutput;
use reqwest::Client;

use crate::{net::minecraft::MinecraftUserProfile, Paths};

use self::auth::AuthParameters;

/// ID for a user
pub type UserID = Arc<str>;

/// A user account that can play the game
#[derive(Debug, Clone)]
pub struct User {
	/// Type of this user
	pub(crate) kind: UserKind,
	/// This user's ID
	id: UserID,
	/// The user's username
	name: Option<String>,
	/// The user's UUID
	uuid: Option<String>,
	/// The user's access token
	access_token: Option<AccessToken>,
	/// The user's public / private key pair
	keypair: Option<Keypair>,
}

/// Type of a user
#[derive(Debug, Clone)]
pub enum UserKind {
	/// A new Microsoft user, the standard account
	Microsoft {
		/// The Xbox UID of the user
		xbox_uid: Option<String>,
	},
	/// A demo user
	Demo,
	/// An unknown user kind
	Unknown(String),
}

impl User {
	/// Create a new user
	pub fn new(kind: UserKind, id: UserID) -> Self {
		Self {
			kind,
			id,
			name: None,
			uuid: None,
			access_token: None,
			keypair: None,
		}
	}

	/// Get the ID of this user
	pub fn get_id(&self) -> &UserID {
		&self.id
	}

	/// Get the name of this user
	pub fn get_name(&self) -> Option<&String> {
		self.name.as_ref()
	}

	/// Checks if this user is a Microsoft user
	pub fn is_microsoft(&self) -> bool {
		matches!(self.kind, UserKind::Microsoft { .. })
	}

	/// Checks if this user is a demo user
	pub fn is_demo(&self) -> bool {
		matches!(self.kind, UserKind::Demo)
	}

	/// Checks if this user is an unverified user
	pub fn is_unverified(&self) -> bool {
		matches!(self.kind, UserKind::Demo)
	}

	/// Gets the kind of this user
	pub fn get_kind(&self) -> &UserKind {
		&self.kind
	}

	/// Set this user's UUID
	pub fn set_uuid(&mut self, uuid: &str) {
		self.uuid = Some(uuid.to_string());
	}

	/// Get the UUID of this user, if it exists
	pub fn get_uuid(&self) -> Option<&String> {
		self.uuid.as_ref()
	}

	/// Get the access token of this user, if it exists
	pub fn get_access_token(&self) -> Option<&AccessToken> {
		self.access_token.as_ref()
	}

	/// Get the Xbox UID of this user, if it exists
	pub fn get_xbox_uid(&self) -> Option<&String> {
		if let UserKind::Microsoft { xbox_uid } = &self.kind {
			xbox_uid.as_ref()
		} else {
			None
		}
	}

	/// Get the keypair of this user, if it exists
	pub fn get_keypair(&self) -> Option<&Keypair> {
		self.keypair.as_ref()
	}

	/// Validate the user's username. Returns true if the username is valid,
	/// and false if it isn't
	pub fn validate_username(&self) -> bool {
		if let Some(name) = &self.name {
			if name.is_empty() || name.len() > 16 {
				return false;
			}

			for c in name.chars() {
				if !c.is_ascii_alphanumeric() && c != '_' {
					return false;
				}
			}
		}

		true
	}
}

/// List of users and AuthState
#[derive(Clone)]
pub struct UserManager {
	/// The current state of authentication
	state: AuthState,
	/// All configured / available users
	users: HashMap<UserID, User>,
	/// The MS client ID
	ms_client_id: ClientId,
	/// Whether the manager has been set as offline for authentication
	offline: bool,
	/// Custom auth function for plugin injection
	custom_auth_fn: Option<CustomAuthFunction>,
}

/// State of authentication
#[derive(Debug, Clone)]
enum AuthState {
	/// No user is picked / MCVM is offline
	Offline,
	/// A default user has been selected
	UserChosen(UserID),
}

impl UserManager {
	/// Create a new UserManager
	pub fn new(ms_client_id: ClientId) -> Self {
		Self {
			state: AuthState::Offline,
			users: HashMap::new(),
			ms_client_id,
			offline: false,
			custom_auth_fn: None,
		}
	}

	/// Add a new user to the manager
	pub fn add_user(&mut self, user: User) {
		self.add_user_with_id(user.id.clone(), user);
	}

	/// Add a new user to the manager with a different
	/// ID than the user struct has. I don't know why you would need to do this,
	/// but it's an option anyways
	pub fn add_user_with_id(&mut self, user_id: UserID, user: User) {
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
	pub fn iter_users(&self) -> impl Iterator<Item = (&UserID, &User)> {
		self.users.iter()
	}

	/// Remove a user with an ID. Will unchoose the user if it is chosen.
	pub fn remove_user(&mut self, user_id: &str) {
		let is_chosen = if let Some(chosen) = self.get_chosen_user() {
			chosen.get_id().deref() == user_id
		} else {
			false
		};
		if is_chosen {
			self.unchoose_user();
		}
		self.users.remove(user_id);
	}

	/// Set the chosen user. Fails if the user does not exist.
	/// If the specified user is already chosen and authenticated, then
	/// no change will be made.
	pub fn choose_user(&mut self, user_id: &str) -> anyhow::Result<()> {
		if !self.user_exists(user_id) {
			bail!("Chosen user does not exist");
		}
		self.state = AuthState::UserChosen(user_id.into());
		Ok(())
	}

	/// Get the currently chosen user, if there is one
	pub fn get_chosen_user(&self) -> Option<&User> {
		match &self.state {
			AuthState::Offline => None,
			AuthState::UserChosen(user_id) => self.users.get(user_id),
		}
	}

	/// Get the currently chosen mutably, if there is one
	pub fn get_chosen_user_mut(&mut self) -> Option<&mut User> {
		match &self.state {
			AuthState::Offline => None,
			AuthState::UserChosen(user_id) => self.users.get_mut(user_id),
		}
	}

	/// Checks if a user is chosen
	pub fn is_user_chosen(&self) -> bool {
		matches!(self.state, AuthState::UserChosen(..))
	}

	/// Checks if a user is chosen and it is authenticated
	pub fn is_authenticated(&self) -> bool {
		let Some(user) = self.get_chosen_user() else {
			return false;
		};
		user.is_authenticated()
	}

	/// Ensures that the currently chosen user is authenticated
	pub async fn authenticate(
		&mut self,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		if let AuthState::UserChosen(user_id) = &mut self.state {
			let user = self
				.users
				.get_mut(user_id)
				.expect("User in AuthState does not exist");

			if !user.is_authenticated() || !user.is_auth_valid(paths) {
				let params = AuthParameters {
					req_client: client,
					paths,
					force: false,
					offline: self.offline,
					client_id: self.ms_client_id.clone(),
					custom_auth_fn: self.custom_auth_fn.clone(),
				};
				user.authenticate(params, o).await?;
			}
		}

		Ok(())
	}

	/// Unchooses the current user, if one is chosen
	pub fn unchoose_user(&mut self) {
		self.state = AuthState::Offline;
	}

	/// Adds users from another UserManager, and copies it's authentication state
	pub fn steal_users(&mut self, other: &Self) {
		self.users.extend(other.users.clone());
		self.state = other.state.clone();
	}

	/// Set whether the UserManager is offline. When offline, authentication won't use remote servers
	/// if possible, and error if it doesn't have enough local information to authenticate
	pub fn set_offline(&mut self, offline: bool) {
		self.offline = offline;
	}

	/// Set the manager's custom auth function
	pub fn set_custom_auth_function(&mut self, func: CustomAuthFunction) {
		self.custom_auth_fn = Some(func);
	}
}

/// Function for custom authentication handling
pub type CustomAuthFunction =
	Arc<dyn Fn(&str, &str) -> anyhow::Result<Option<MinecraftUserProfile>>>;

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

	#[test]
	fn test_user_manager() {
		let mut users = UserManager::new(ClientId::new(String::new()));
		let user = User::new(UserKind::Demo, "foo".into());
		users.add_user(user);
		users.choose_user("foo").expect("Failed to choose user");
		let user = User::new(UserKind::Demo, "bar".into());
		users.add_user(user);
		users.remove_user("foo");
		assert!(!users.is_user_chosen());
		assert!(!users.user_exists("foo"));
	}
}
