pub mod eval;
pub mod repo;

use crate::io::files::{self, paths::Paths};
use crate::net::download::{Download, DownloadError};
use eval::parse::PkgAst;

use std::path::PathBuf;
use std::fs;

static PKG_EXTENSION: &str = ".pkg.txt";

#[derive(Debug)]
pub struct PkgData {
	contents: String,
	ast: Option<PkgAst>
}

#[derive(Debug, thiserror::Error)]
pub enum PkgError {
	#[error("File operation failed:\n{}", .0)]
	Io(#[from] std::io::Error),
	#[error("Download failed:\n{}", .0)]
	Download(#[from] DownloadError)
}

impl PkgData {
	pub fn new(contents: &str) -> Self {
		Self {
			contents: contents.to_owned(),
			ast: None
		}
	}
}

#[derive(Debug)]
pub enum PkgKind {
	Local(PathBuf),
	Remote(String)
}

#[derive(Debug)]
pub struct Package {
	pub name: String,
	pub version: String,
	pub kind: PkgKind,
	pub data: Option<PkgData>
}

impl Package {
	pub fn new(name: &str, version: &str, kind: PkgKind) -> Self {
		Self {
			name: name.to_owned(),
			version: version.to_owned(),
			kind,
			data: None
		}
	}

	pub fn filename(&self) -> String {
		self.name.clone() + &self.version + PKG_EXTENSION
	}

	pub fn ensure_loaded(&mut self, paths: &Paths) -> Result<(), PkgError> {
		if self.data.is_none() {
			match &self.kind {
				PkgKind::Local(path) => {
					self.data = Some(PkgData::new(&fs::read_to_string(path)?));
				}
				PkgKind::Remote(url) => {
					let cache_dir = paths.project.cache_dir().join("pkg");
					files::create_dir(&cache_dir)?;
					let path = cache_dir.join(self.filename());
					if path.exists() {
						self.data = Some(PkgData::new(&fs::read_to_string(path)?));
					} else {
						let mut dwn = Download::new();
						dwn.url(url)?;
						dwn.add_file(&path)?;
						dwn.add_str();
						dwn.perform()?;
						self.data = Some(PkgData::new(&dwn.get_str()?));
					}
				}
			};
		}
		Ok(())
	}
}
