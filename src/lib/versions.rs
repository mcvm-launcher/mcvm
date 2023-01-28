use core::fmt;

pub type MinecraftVersion = String;

#[derive(Debug)]
pub struct VersionNotFoundError {
	version: String
}

impl VersionNotFoundError {
	pub fn new(version: &str) -> VersionNotFoundError {
		VersionNotFoundError { version: version.to_string() }
	}
}

impl std::error::Error for VersionNotFoundError {}

impl fmt::Display for VersionNotFoundError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_fmt(format_args!("Version not found: {}", self.version))
	}
}
