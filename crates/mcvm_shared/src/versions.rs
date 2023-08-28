use std::fmt::Display;

use serde::Deserialize;

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
	/// Matches any versions between an inclusive range
	Range(String, String),
	/// Matches any version
	Any,
}

impl VersionPattern {
	/// Finds all match in a list of versions
	pub fn get_matches(&self, versions: &[String]) -> Vec<String> {
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
				Some(pos) => versions[..=pos].to_vec(),
				None => vec![],
			},
			Self::After(version) => match versions.iter().position(|e| e == version) {
				Some(pos) => versions[pos..].to_vec(),
				None => vec![],
			},
			Self::Range(start, end) => match versions.iter().position(|e| e == start) {
				Some(start_pos) => match versions.iter().position(|e| e == end) {
					Some(end_pos) => versions[start_pos..=end_pos].to_vec(),
					None => vec![],
				},
				None => vec![],
			},
			Self::Any => versions.to_vec(),
		}
	}

	/// Finds the newest match in a list of versions
	pub fn get_match(&self, versions: &[String]) -> Option<String> {
		self.get_matches(versions).last().cloned()
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
			Self::Range(start, end) => {
				if let Some(start_pos) = versions.iter().position(|x| x == start) {
					if let Some(end_pos) = versions.iter().position(|x| x == end) {
						if let Some(version_pos) = versions.iter().position(|x| x == version) {
							(version_pos >= start_pos) && (version_pos <= end_pos)
						} else {
							false
						}
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

	/// Compares this pattern to a version supplied in a VersionInfo
	pub fn matches_info(&self, version_info: &VersionInfo) -> bool {
		self.matches_single(&version_info.version, &version_info.versions)
	}

	/// Returns the union of matches for multiple patterns
	pub fn match_union(&self, other: &Self, versions: &[String]) -> Vec<String> {
		self.get_matches(versions)
			.iter()
			.zip(other.get_matches(versions))
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

				let range_split: Vec<_> = text.split("..").collect();
				if range_split.len() == 2 {
					let start = range_split
						.first()
						.expect("First element in range split should exist");
					let end = range_split
						.get(1)
						.expect("Second element in range split should exist");
					return Self::Range(start.to_string(), end.to_string());
				}

				Self::Single(text.to_owned())
			}
		}
	}

	/// Checks that a string contains no pattern-special characters
	#[cfg(test)]
	pub fn validate(text: &str) -> bool {
		if text.contains('*') || text.contains("..") || text == "latest" {
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
				Self::Range(start, end) => start.to_owned() + ".." + end,
				Self::Any => String::from("*"),
			}
		)
	}
}

impl<'de> Deserialize<'de> for VersionPattern {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let string = String::deserialize(deserializer)?;
		Ok(Self::from(&string))
	}
}


/// Utility struct that contains the version and version list
#[derive(Debug)]
pub struct VersionInfo {
	pub version: String,
	pub versions: Vec<String>,
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
			VersionPattern::Single(String::from("1.19.3")).get_match(&versions),
			Some(String::from("1.19.3"))
		);
		assert_eq!(
			VersionPattern::Single(String::from("1.18")).get_match(&versions),
			Some(String::from("1.18"))
		);
		assert_eq!(
			VersionPattern::Single(String::from("")).get_match(&versions),
			None
		);
		assert_eq!(
			VersionPattern::Before(String::from("1.18")).get_match(&versions),
			Some(String::from("1.18"))
		);
		assert_eq!(
			VersionPattern::After(String::from("1.16.5")).get_match(&versions),
			Some(String::from("1.19.3"))
		);

		assert_eq!(
			VersionPattern::Before(String::from("1.17")).get_matches(&versions),
			vec![String::from("1.16.5"), String::from("1.17")]
		);
		assert_eq!(
			VersionPattern::After(String::from("1.17")).get_matches(&versions),
			vec![
				String::from("1.17"),
				String::from("1.18"),
				String::from("1.19.3")
			]
		);
		assert_eq!(
			VersionPattern::Range(String::from("1.16.5"), String::from("1.18"))
				.get_matches(&versions),
			vec![
				String::from("1.16.5"),
				String::from("1.17"),
				String::from("1.18"),
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
		assert_eq!(
			VersionPattern::from("1.17.1..1.19.3"),
			VersionPattern::Range(String::from("1.17.1"), String::from("1.19.3"))
		);
	}

	#[test]
	fn test_version_pattern_validation() {
		assert!(VersionPattern::validate("hello"));
		assert!(!VersionPattern::validate("latest"));
		assert!(!VersionPattern::validate("foo-"));
		assert!(!VersionPattern::validate("foo+"));
		assert!(!VersionPattern::validate("f*o"));
		assert!(!VersionPattern::validate("f..o"));
	}
}
