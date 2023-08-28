pub mod auth;
pub mod uuid;

use std::collections::HashMap;

use anyhow::Context;
use oauth2::ClientId;
use reqwest::Client;

use crate::net::microsoft::Keypair;

#[derive(Debug, Copy, Clone)]
pub enum UserKind {
	Microsoft,
	Demo,
	Unverified,
}

/// A user account that can play the game
#[derive(Debug)]
pub struct User {
	pub kind: UserKind,
	pub id: String,
	pub name: String,
	pub uuid: Option<String>,
	pub access_token: Option<String>,
	pub keypair: Option<Keypair>,
	pub xbox_uid: Option<String>,
}

impl User {
	pub fn new(kind: UserKind, id: &str, name: &str) -> Self {
		Self {
			kind,
			id: id.to_owned(),
			name: name.to_owned(),
			uuid: None,
			access_token: None,
			keypair: None,
			xbox_uid: None,
		}
	}

	pub fn set_uuid(&mut self, uuid: &str) {
		self.uuid = Some(uuid.to_string());
	}
}

/// State of authentication
#[derive(Debug)]
pub enum AuthState {
	Offline,
	UserChosen(String),
	Authed(String),
}

/// List of users and AuthState
#[derive(Debug)]
pub struct UserManager {
	pub state: AuthState,
	pub users: HashMap<String, User>,
}

impl UserManager {
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
	pub async fn ensure_authenticated(&mut self, client_id: ClientId) -> anyhow::Result<()> {
		if let AuthState::UserChosen(user) = &self.state {
			let user = self
				.users
				.get_mut(user)
				.expect("User in AuthState does not exist");
			if let UserKind::Microsoft = &user.kind {
				let client = Client::new();
				let auth_result = crate::data::user::auth::authenticate(client_id, &client)
					.await
					.context("Failed to authenticate user")?;
				let certificate =
					crate::net::microsoft::get_user_certificate(&auth_result.access_token, &client)
						.await
						.context("Failed to get user certificate")?;
				user.access_token = Some(auth_result.access_token);
				user.uuid = Some(auth_result.profile.uuid);
				user.keypair = Some(certificate.key_pair);
				user.xbox_uid = Some(auth_result.xbox_uid);
			}
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
pub fn validate_username(kind: UserKind, name: &str) -> bool {
	match kind {
		UserKind::Microsoft | UserKind::Demo | UserKind::Unverified => {
			if name.is_empty() || name.len() > 16 {
				return false;
			}

			for c in name.chars() {
				if !c.is_ascii_alphanumeric() && c != '_' {
					return false;
				}
			}
		}
	}

	true
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_username_validation() {
		assert!(validate_username(UserKind::Microsoft, "CarbonSmasher"));
		assert!(validate_username(UserKind::Demo, "12345"));
		assert!(validate_username(UserKind::Microsoft, "Foo_Bar888"));
		assert!(!validate_username(UserKind::Microsoft, ""));
		assert!(!validate_username(
			UserKind::Microsoft,
			"ABCDEFGHIJKLMNOPQRS"
		));
		assert!(!validate_username(UserKind::Microsoft, "+++"));
	}
}
