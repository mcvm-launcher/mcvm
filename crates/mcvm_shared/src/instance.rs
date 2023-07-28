use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Minecraft game side, client or server
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
	Client,
	Server,
}

impl Side {
	pub fn parse_from_str(string: &str) -> Option<Self> {
		match string {
			"client" => Some(Self::Client),
			"server" => Some(Self::Server),
			_ => None,
		}
	}
}

impl Display for Side {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Client => "client",
				Self::Server => "server",
			}
		)
	}
}
