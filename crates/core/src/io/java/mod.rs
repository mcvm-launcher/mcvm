use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Operating Java memory arguments
pub mod args;
/// Use of Java's classpath format
pub mod classpath;
/// Installation of Java for MCVM
pub mod install;

/// A major Java version (e.g. 14 or 17)
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct JavaMajorVersion(pub u16);

impl Display for JavaMajorVersion {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl JavaMajorVersion {
	/// Constructs a new JavaMajorVersion
	pub fn new(version: u16) -> Self {
		Self(version)
	}

	/// Tries to parse a major version from a string
	///
	/// ```
	/// use mcvm_core::io::java::JavaMajorVersion;
	/// let string = "17";
	/// let version = JavaMajorVersion::parse(string);
	/// assert_eq!(version, Some(JavaMajorVersion(17)));
	/// ```
	pub fn parse(string: &str) -> Option<Self> {
		string.parse().map(Self::new).ok()
	}
}

/// Dealing with Maven
pub mod maven {
	/// Sections of a Maven library string
	#[derive(Debug, PartialEq, Eq, Clone)]
	pub struct MavenLibraryParts {
		/// The organizations of the package
		pub orgs: Vec<String>,
		/// The package name
		pub package: String,
		/// The version of the package
		pub version: String,
	}

	impl MavenLibraryParts {
		/// Extract the parts of a library string
		///
		/// ```
		/// use mcvm_core::io::java::maven::MavenLibraryParts;
		///
		/// let string = "foo.bar.baz:hello:world";
		/// let parts = MavenLibraryParts::parse_from_str(string).unwrap();
		/// assert_eq!(parts.orgs, vec!["foo".to_string(), "bar".to_string(), "baz".to_string()]);
		/// assert_eq!(parts.package, "hello".to_string());
		/// assert_eq!(parts.version, "world".to_string());
		/// ```
		pub fn parse_from_str(string: &str) -> Option<Self> {
			let mut parts = string.split(':');
			let orgs: Vec<String> = parts.next()?.split('.').map(|x| x.to_owned()).collect();
			let package = parts.next()?.to_owned();
			let version = parts.next()?.to_owned();
			Some(Self {
				orgs,
				package,
				version,
			})
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[test]
		fn test_maven_library_destructuring() {
			assert_eq!(
				MavenLibraryParts::parse_from_str("foo.bar.baz:hel.lo:wo.rld")
					.expect("Parts did not parse correctly"),
				MavenLibraryParts {
					orgs: vec!["foo".to_string(), "bar".to_string(), "baz".to_string()],
					package: "hel.lo".into(),
					version: "wo.rld".into()
				}
			)
		}
	}
}
