/// A known identifier for a package
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct PkgIdentifier {
	pub name: String,
	pub version: String,
}

impl PkgIdentifier {
	pub fn new(name: &str, version: &str) -> Self {
		Self {
			name: name.to_owned(),
			version: version.to_owned(),
		}
	}
}
