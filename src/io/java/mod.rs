use serde::{Deserialize, Serialize};

/// Operating Java memory arguments
pub mod args;
/// Use of Java's classpath format
pub mod classpath;
/// Installation of Java for MCVM
pub mod install;

/// A major Java version (e.g. 14 or 17)
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct JavaMajorVersion(pub u16);

/// Dealing with Maven
pub mod maven {
	/// Sections of a Maven library string
	#[derive(Debug, PartialEq)]
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
		pub fn from_str(string: &str) -> Option<Self> {
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
				MavenLibraryParts::from_str("foo.bar.baz:hel.lo:wo.rld")
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
