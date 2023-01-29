use std::path::PathBuf;

pub struct PkgData {
	contents: Option<String>
}

pub enum PkgType {
	Local(PathBuf),
	Remote(String)
}

pub struct Package {
	pub name: String,
	pkg_type: PkgType
}
