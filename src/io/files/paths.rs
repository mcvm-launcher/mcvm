use super::create_dir;

use directories::{BaseDirs, ProjectDirs};

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum PathsError {
	#[error("IO operation failed:{}", .0)]
	Io(#[from] std::io::Error),
	#[error("Failed to find base directories")]
	Base,
}

/// Store for all of the paths that are used throughout the application
#[derive(Debug, Clone)]
pub struct Paths {
	/// System-wide directories
	pub base: BaseDirs,
	/// Project-specific directories
	pub project: ProjectDirs,
	/// Holds internal data
	pub internal: PathBuf,
	/// Holds game assets
	pub assets: PathBuf,
	/// Holds game libraries
	pub libraries: PathBuf,
	/// Holds Java installations
	pub java: PathBuf,
	/// Holds addons
	pub addons: PathBuf,
	/// Holds cached package scripts
	pub pkg_cache: PathBuf,
	/// Holds cached package repository indexes
	pub pkg_index_cache: PathBuf,
	/// Holds game jar files
	pub jars: PathBuf,
}

impl Paths {
	pub fn new() -> Result<Paths, PathsError> {
		let base = BaseDirs::new().ok_or(PathsError::Base)?;
		let project = ProjectDirs::from("", "mcvm", "mcvm").ok_or(PathsError::Base)?;

		let internal = project.data_dir().join("internal");
		let assets = internal.join("assets");
		let libraries = internal.join("libraries");
		let java = internal.join("java");
		let addons = internal.join("addons");
		let pkg_cache = project.cache_dir().join("pkg");
		let pkg_index_cache = pkg_cache.join("index");
		let jars = internal.join("jars");

		create_dir(project.data_dir())?;
		create_dir(project.cache_dir())?;
		create_dir(project.config_dir())?;
		create_dir(project.runtime_dir().ok_or(PathsError::Base)?)?;
		create_dir(&internal)?;
		create_dir(&assets)?;
		create_dir(&java)?;
		create_dir(&addons)?;
		create_dir(&pkg_cache)?;
		create_dir(&pkg_index_cache)?;
		create_dir(&jars)?;

		Ok(Paths {
			base,
			project,
			internal,
			assets,
			libraries,
			java,
			addons,
			pkg_cache,
			pkg_index_cache,
			jars,
		})
	}
}
