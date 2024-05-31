use std::fmt::Display;

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use mcvm_core::user::{User, UserKind};

#[derive(Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
/// Configuration for a user
pub struct UserConfig {
	/// Configuration for the different user variants
	#[serde(flatten)]
	pub variant: UserVariant,
}

/// Different variants of users for configuration
#[derive(Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum UserVariant {
	/// A Microsoft user
	Microsoft {},
	/// A demo user
	Demo {},
	/// An unknown user
	#[cfg_attr(not(feature = "schema"), serde(untagged))]
	Unknown(String),
}

impl UserVariant {
	fn to_user_kind(&self) -> UserKind {
		match self {
			Self::Microsoft { .. } => UserKind::Microsoft { xbox_uid: None },
			Self::Demo { .. } => UserKind::Demo,
			Self::Unknown(id) => UserKind::Unknown(id.clone()),
		}
	}
}

impl UserConfig {
	/// Creates a user from this user config
	pub fn to_user(&self, id: &str) -> User {
		User::new(self.variant.to_user_kind(), id.into())
	}
}

impl Display for UserVariant {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Microsoft {} => write!(f, "microsoft"),
			Self::Demo {} => write!(f, "demo"),
			Self::Unknown(other) => write!(f, "{other}"),
		}
	}
}
