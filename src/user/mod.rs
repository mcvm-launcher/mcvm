#[derive(Debug)]
pub enum UserKind {
	Microsoft,
	Demo
}

#[derive(Debug)]
pub struct User {
	kind: UserKind,
	id: String,
	name: String,
	uuid: Option<String>
}

impl User {
	pub fn new(kind: UserKind, id: &str, name: &str) -> Self {
		Self {
			kind,
			id: id.to_owned(),
			name: name.to_owned(),
			uuid: None
		}
	}

	pub fn set_uuid(&mut self, uuid: &str) {
		self.uuid = Some(uuid.to_owned());
	}
}

#[derive(Debug)]
pub enum AuthState<'a> {
	Authed(&'a mut User),
	Offline
}
