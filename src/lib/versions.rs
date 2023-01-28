#[derive(Debug, thiserror::Error)]
#[error("Version not found: {}", .version.as_string())]
pub struct VersionNotFoundError {
	pub version: MinecraftVersion
}

impl VersionNotFoundError {
	pub fn new(version: MinecraftVersion) -> VersionNotFoundError {
		VersionNotFoundError{version}
	}
}

#[derive(Debug)]
pub enum MinecraftVersion {
	Unknown(String)
}

impl MinecraftVersion {
	pub fn from(string: &str) -> Self {
		Self::Unknown(string.to_string())
	}

	pub fn as_string(&self) -> &String {
		match self {
			Self::Unknown(string) => string
		}
	}
}

static _VERSION_LIST: [&str; 1] = ["1.19"];
