use std::fmt::Display;

/// Minecraft game side, client or server
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Side {
	Client,
	Server,
}

impl Side {
	pub fn from_str(string: &str) -> Option<Self> {
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
