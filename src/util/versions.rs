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

// Pattern matching for the version of Minecraft or a package
#[derive(Debug, Hash, Clone)]
pub enum VersionPattern {
	Single(String),
	Latest(Option<String>)
}

impl VersionPattern {
	// Finds a match in a list of versions
	pub fn matches(&self, versions: &[String]) -> Option<String> {
		match self {
			VersionPattern::Single(version) => match versions.contains(version) {
				true => Some(version.to_string()),
				false => None
			},
			VersionPattern::Latest(found) => match found {
				Some(found) => Some(found.clone()),
				None => versions.get(versions.len()).cloned()
			}
		}
	}

	pub fn as_string(&self) -> &str {
		match self {
			Self::Single(version) => version,
			Self::Latest(..) => "latest"
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
