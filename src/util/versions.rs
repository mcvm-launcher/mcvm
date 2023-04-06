use std::fmt::Display;

/// Pattern matching for the version of Minecraft, a package, etc.
#[derive(Debug, Hash, Clone, PartialEq)]
pub enum VersionPattern {
	/// Matches a single version
	Single(String),
	/// Matches the latest version in the list
	Latest(Option<String>),
	/// Matches any version that is <= a version
	Before(String),
	/// Matches any version that is >= a version
	After(String),
	/// Matches any version
	Any,
}

impl VersionPattern {
	/// Finds all match in a list of versions
	pub fn _get_matches(&self, versions: &[String]) -> Vec<String> {
		match self {
			Self::Single(version) => match versions.contains(version) {
				true => vec![version.to_string()],
				false => vec![],
			},
			Self::Latest(found) => match found {
				Some(found) => vec![found.clone()],
				None => match versions.get(versions.len()).cloned() {
					Some(version) => vec![version],
					None => vec![],
				},
			},
			Self::Before(version) => match versions.iter().position(|e| e == version) {
				Some(pos) => versions[..pos + 1].to_vec(),
				None => vec![],
			},
			Self::After(version) => match versions.iter().position(|e| e == version) {
				Some(pos) => versions[pos..].to_vec(),
				None => vec![],
			},
			Self::Any => versions.to_vec(),
		}
	}

	/// Finds the newest match in a list of versions
	pub fn _get_match(&self, versions: &[String]) -> Option<String> {
		self._get_matches(versions).last().cloned()
	}

	/// Compares this pattern to a single string.
	/// For some pattern types, this may return false if it is unable to deduce an
	/// answer from the list of versions provided.
	pub fn matches_single(&self, version: &str, versions: &[String]) -> bool {
		match self {
			Self::Single(vers) => version == vers,
			Self::Latest(cached) => match cached {
				Some(vers) => version == vers,
				None => {
					if let Some(latest) = versions.last() {
						version == latest
					} else {
						false
					}
				}
			},
			Self::Before(vers) => {
				if let Some(vers_pos) = versions.iter().position(|x| x == vers) {
					if let Some(version_pos) = versions.iter().position(|x| x == version) {
						version_pos <= vers_pos
					} else {
						false
					}
				} else {
					false
				}
			}
			Self::After(vers) => {
				if let Some(vers_pos) = versions.iter().position(|x| x == vers) {
					if let Some(version_pos) = versions.iter().position(|x| x == version) {
						version_pos >= vers_pos
					} else {
						false
					}
				} else {
					false
				}
			}
			Self::Any => versions.contains(&version.to_owned()),
		}
	}

	/// Returns the union of matches for multiple patterns
	pub fn _match_union(&self, other: &Self, versions: &[String]) -> Vec<String> {
		self._get_matches(versions)
			.iter()
			.zip(other._get_matches(versions))
			.filter_map(
				|(left, right)| {
					if *left == right {
						Some(right)
					} else {
						None
					}
				},
			)
			.collect()
	}

	/// Creates a version pattern by parsing a string
	pub fn from(text: &str) -> Self {
		match text {
			"latest" => Self::Latest(None),
			"*" => Self::Any,
			text => {
				if let Some(last) = text.chars().last() {
					match last {
						'-' => return Self::Before(text[..text.len() - 1].to_string()),
						'+' => return Self::After(text[..text.len() - 1].to_string()),
						_ => {}
					}
				}
				Self::Single(text.to_owned())
			}
		}
	}

	/// Checks that a string contains no pattern-special characters
	pub fn validate(text: &str) -> bool {
		if text.contains('*') || text == "latest" {
			return false;
		}
		if let Some(last) = text.chars().last() {
			if last == '-' || last == '+' {
				return false;
			}
		}
		true
	}
}

impl Display for VersionPattern {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Single(version) => version.to_owned(),
				Self::Latest(..) => String::from("latest"),
				Self::Before(version) => version.to_owned() + "-",
				Self::After(version) => version.to_owned() + "+",
				Self::Any => String::from("*"),
			}
		)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_version_pattern() {
		let versions = vec![
			String::from("1.16.5"),
			String::from("1.17"),
			String::from("1.18"),
			String::from("1.19.3"),
		];

		assert_eq!(
			VersionPattern::Single(String::from("1.19.3"))._get_match(&versions),
			Some(String::from("1.19.3"))
		);
		assert_eq!(
			VersionPattern::Single(String::from("1.18"))._get_match(&versions),
			Some(String::from("1.18"))
		);
		assert_eq!(
			VersionPattern::Single(String::from(""))._get_match(&versions),
			None
		);
		assert_eq!(
			VersionPattern::Before(String::from("1.18"))._get_match(&versions),
			Some(String::from("1.18"))
		);
		assert_eq!(
			VersionPattern::After(String::from("1.16.5"))._get_match(&versions),
			Some(String::from("1.19.3"))
		);

		assert_eq!(
			VersionPattern::Before(String::from("1.17"))._get_matches(&versions),
			vec![String::from("1.16.5"), String::from("1.17")]
		);
		assert_eq!(
			VersionPattern::After(String::from("1.17"))._get_matches(&versions),
			vec![
				String::from("1.17"),
				String::from("1.18"),
				String::from("1.19.3")
			]
		);

		assert!(VersionPattern::Before(String::from("1.18")).matches_single("1.16.5", &versions));
		assert!(VersionPattern::After(String::from("1.18")).matches_single("1.19.3", &versions));
		assert!(VersionPattern::Latest(None).matches_single("1.19.3", &versions));
	}

	#[test]
	fn test_version_pattern_parse() {
		assert_eq!(
			VersionPattern::from("+1.19.5"),
			VersionPattern::Single(String::from("+1.19.5"))
		);
		assert_eq!(VersionPattern::from("latest"), VersionPattern::Latest(None));
		assert_eq!(
			VersionPattern::from("1.19.5-"),
			VersionPattern::Before(String::from("1.19.5"))
		);
		assert_eq!(
			VersionPattern::from("1.19.5+"),
			VersionPattern::After(String::from("1.19.5"))
		);
	}

	#[test]
	fn test_version_pattern_validation() {
		assert!(VersionPattern::validate("hello"));
		assert!(!VersionPattern::validate("latest"));
		assert!(!VersionPattern::validate("foo-"));
		assert!(!VersionPattern::validate("foo+"));
		assert!(!VersionPattern::validate("f*o"));
	}
}
