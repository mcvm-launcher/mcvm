use std::{fmt::Display, str::FromStr};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

/// Minecraft game side, client or server
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
	/// The default game
	Client,
	/// A dedicated server
	Server,
}

impl Side {
	/// Parse a Side from a string
	pub fn parse_from_str(string: &str) -> Option<Self> {
		match string {
			"client" => Some(Self::Client),
			"server" => Some(Self::Server),
			_ => None,
		}
	}
}

impl FromStr for Side {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::parse_from_str(s).ok_or(anyhow!("Not a valid side"))
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
