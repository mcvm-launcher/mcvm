use serde::{Deserialize, Serialize};

use crate::data::user::{User, UserKind};

#[derive(Deserialize, Serialize, Clone)]
/// Configuration for a user
pub struct UserConfig {
	/// The username of the user
	pub name: String,
	/// Configuration for the different user variants
	#[serde(flatten)]
	pub variant: UserVariant,
}

/// Different variants of users for configuration
#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum UserVariant {
	/// A Microsoft user
	Microsoft {
		/// The UUID of the user
		uuid: Option<String>,
	},
	/// A demo user
	Demo {
		/// The UUID of the user
		uuid: Option<String>,
	},
	/// An unverified user
	Unverified {},
}

impl UserVariant {
	fn to_user_kind(&self) -> UserKind {
		match self {
			Self::Microsoft { .. } => UserKind::Microsoft { xbox_uid: None },
			Self::Demo { .. } => UserKind::Demo,
			Self::Unverified {} => UserKind::Unverified,
		}
	}
}

impl UserConfig {
	/// Creates a user from this user config
	pub fn to_user(&self, id: &str) -> User {
		let mut user = User::new(self.variant.to_user_kind(), id, &self.name);
		match &self.variant {
			UserVariant::Microsoft { uuid } | UserVariant::Demo { uuid } => {
				if let Some(uuid) = uuid {
					user.set_uuid(uuid);
				}
			}
			_ => {}
		}

		user
	}
}
