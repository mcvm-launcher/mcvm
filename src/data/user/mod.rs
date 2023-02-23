use std::collections::HashMap;

#[derive(Debug)]
pub enum UserKind {
	Microsoft,
	Demo
}

#[derive(Debug)]
pub struct User {
	pub kind: UserKind,
	pub id: String,
	pub name: String,
	pub uuid: Option<String>,
	pub access_token: Option<String>
}

impl User {
	pub fn new(kind: UserKind, id: &str, name: &str) -> Self {
		Self {
			kind,
			id: id.to_owned(),
			name: name.to_owned(),
			uuid: None,
			access_token: None
		}
	}

	pub fn set_uuid(&mut self, uuid: &str) {
		self.uuid = Some(uuid.to_string());
	}
}

#[derive(Debug)]
pub enum AuthState {
	Authed(String),
	Offline
}

#[derive(Debug)]
pub struct Auth {
	pub state: AuthState,
	pub users: HashMap<String, User>
}

impl Auth {
	pub fn new() -> Self {
		Self {
			state: AuthState::Offline,
			users: HashMap::new()
		}
	}

	pub fn get_user(&self) -> Option<&User> {
		match &self.state {
			AuthState::Authed(user_id) => self.users.get(user_id),
			AuthState::Offline => None
		}
	}
}
