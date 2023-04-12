use color_print::cprintln;
use serde::Deserialize;

use crate::data::user::{User, UserKind};

#[derive(Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum UserVariant {
	Microsoft { uuid: Option<String> },
	Demo { uuid: Option<String> },
	Unverified {},
}

impl UserVariant {
	fn to_user_kind(&self) -> UserKind {
		match self {
			Self::Microsoft { .. } => UserKind::Microsoft,
			Self::Demo { .. } => UserKind::Demo,
			Self::Unverified {} => UserKind::Unverified,
		}
	}
}

#[derive(Deserialize)]
/// Configuration for a user
pub struct UserConfig {
	pub name: String,
	#[serde(flatten)]
	pub variant: UserVariant,
}

impl UserConfig {
	/// Creates a user from this user config
	pub fn to_user(&self, id: &str) -> User {
		let mut user = User::new(self.variant.to_user_kind(), id, &self.name);
		match &self.variant {
			UserVariant::Microsoft { uuid } | UserVariant::Demo { uuid } => {
				match uuid {
					Some(uuid) => user.set_uuid(uuid),
					None => {
						cprintln!("<y>Warning: It is recommended to have your uuid in the configuration for user {}", id);
					}
				};
			}
			_ => {}
		}

		user
	}
}
