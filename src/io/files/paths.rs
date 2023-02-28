use super::create_dir;

use directories::{BaseDirs, ProjectDirs};

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum PathsError {
	#[error("IO operation failed:{}", .0)]
	Io(#[from] std::io::Error),
	#[error("Failed to find base directories")]
	Base
}

#[derive(Debug, Clone)]
pub struct Paths {
	pub base: BaseDirs,
	pub project: ProjectDirs,
	pub internal: PathBuf,
	pub assets: PathBuf,
	pub libraries: PathBuf,
	pub java: PathBuf,
	pub mcvm_assets: PathBuf,
	pub pkg_cache: PathBuf,
	pub pkg_index_cache: PathBuf
}

impl Paths {
	pub fn new() -> Result<Paths, PathsError> {
		let base = BaseDirs::new().ok_or(PathsError::Base)?;
		let project = ProjectDirs::from("", "mcvm", "mcvm").ok_or(PathsError::Base)?;
		
		let internal = project.data_dir().join("internal");
		let assets = internal.join("assets");
		let libraries = internal.join("libraries");
		let java = internal.join("java");
		let mcvm_assets = internal.join("mcvm_assets");
		let pkg_cache = project.cache_dir().join("pkg");
		let pkg_index_cache = pkg_cache.join("index");
		
		create_dir(project.data_dir())?;
		create_dir(project.cache_dir())?;
		create_dir(project.config_dir())?;
		create_dir(project.runtime_dir().ok_or(PathsError::Base)?)?;
		create_dir(&internal)?;
		create_dir(&assets)?;
		create_dir(&java)?;
		create_dir(&mcvm_assets)?;
		create_dir(&pkg_cache)?;
		create_dir(&pkg_index_cache)?;
		
		Ok(Paths {
			base,
			project,
			internal,
			assets,
			libraries,
			java,
			mcvm_assets,
			pkg_cache,
			pkg_index_cache
		})
	}
}

// Replace tilde in a string with the home directory
pub fn _replace_home(string: &str, paths: &Paths) -> String {
	string.to_owned().replace(
		'~',
		paths.base.home_dir().to_str().expect("Failed to convert home directory to a string")
	)
}
