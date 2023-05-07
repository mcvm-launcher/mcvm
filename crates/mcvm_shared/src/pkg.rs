/// A known identifier for a package
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct PkgIdentifier {
	pub name: String,
	pub version: u32,
}

impl PkgIdentifier {
	pub fn new(name: &str, version: u32) -> Self {
		Self {
			name: name.to_owned(),
			version,
		}
	}
}
