use super::create_dir;

use directories::{BaseDirs, ProjectDirs};

use std::{path::PathBuf, fmt::Display};

// #[derive(Debug)]
// pub struct PathsError {}
// impl Display for PathsError {
// 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
// 		write!(f, "Failed to open program base directories")
// 	}
// }
// impl std::error::Error for PathsError {}

#[derive(Debug, thiserror::Error)]
pub enum PathsError {
	#[error("IO operation failed:{}", .0)]
	Io(#[from] std::io::Error),
	#[error("Failed to find base directories")]
	Base
}

#[derive(Debug)]
pub struct Paths {
	pub base: BaseDirs,
	pub project: ProjectDirs,
	pub internal: PathBuf,
	pub assets: PathBuf,
	pub libraries: PathBuf,
	pub java: PathBuf
}

impl Paths {
	pub fn new() -> Result<Paths, PathsError> {
		let base = BaseDirs::new().ok_or(PathsError::Base)?;
		let project = ProjectDirs::from("", "mcvm", "mcvm").ok_or(PathsError::Base)?;
		
		let internal= base.data_dir().join("internal");
		let assets = internal.join("assets");
		let libraries = internal.join("libraries");
		let java = internal.join("java");
		
		create_dir(project.data_dir())?;
		create_dir(project.cache_dir())?;
		create_dir(project.config_dir())?;
		create_dir(project.runtime_dir().ok_or(PathsError::Base)?)?;
		create_dir(&internal)?;
		create_dir(&assets)?;
		create_dir(&java)?;
		
		Ok(Paths {
			base,
			project,
			internal,
			assets,
			libraries,
			java,
		})
	}
}
