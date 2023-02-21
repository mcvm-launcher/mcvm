#[derive(Debug, thiserror::Error)]
#[error("Version not found: {}", .version.as_string())]
pub struct VersionNotFoundError {
	pub version: MinecraftVersion
}

impl VersionNotFoundError {
	pub fn new(version: &MinecraftVersion) -> VersionNotFoundError {
		VersionNotFoundError{version: version.clone()}
	}
}

#[derive(Debug, Clone)]
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

pub enum VersionPattern {
	Single(String)
}

impl VersionPattern {
	pub fn matches(&self, versions: &[String]) -> Option<String> {
		match self {
			VersionPattern::Single(version) => match versions.contains(version) {
				true => Some(version.to_string()),
				false => None
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_version_pattern() {
		let versions = vec![
			String::from("1.19.3"),
			String::from("1.18")
		];
		assert_eq!(VersionPattern::Single(String::from("1.19.3")).matches(&versions), Some(String::from("1.19.3")));
		assert_eq!(VersionPattern::Single(String::from("1.18")).matches(&versions), Some(String::from("1.18")));
		assert_eq!(VersionPattern::Single(String::from("")).matches(&versions), None);
	}
}
