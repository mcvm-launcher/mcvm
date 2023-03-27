use super::create_dir;

use anyhow::anyhow;
use directories::{BaseDirs, ProjectDirs};

use std::path::PathBuf;

/// Store for all of the paths that are used throughout the application
#[derive(Debug, Clone)]
pub struct Paths {
	/// System-wide directories
	pub base: BaseDirs,
	/// Project-specific directories
	pub project: ProjectDirs,
	/// Holds data
	pub data: PathBuf,
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
	/// Holds log files
	pub logs: PathBuf,
	/// Holds launch log files
	pub launch_logs: PathBuf,
}

impl Paths {
	pub fn new() -> anyhow::Result<Paths> {
		let base = BaseDirs::new().ok_or(anyhow!("Base directories failed"))?;
		let project = ProjectDirs::from("", "mcvm", "mcvm")
			.ok_or(anyhow!("Base directories failed"))?;

		let data = project.data_dir().to_owned();
		let internal = data.join("internal");
		let assets = internal.join("assets");
		let libraries = internal.join("libraries");
		let java = internal.join("java");
		let addons = internal.join("addons");
		let pkg_cache = project.cache_dir().join("pkg");
		let pkg_index_cache = pkg_cache.join("index");
		let jars = internal.join("jars");
		let logs = data.join("logs");
		let launch_logs = logs.join("launch");

		create_dir(&data)?;
		create_dir(project.cache_dir())?;
		create_dir(project.config_dir())?;
		create_dir(project.runtime_dir().ok_or(anyhow!("Base directories failed"))?)?;
		create_dir(&internal)?;
		create_dir(&assets)?;
		create_dir(&java)?;
		create_dir(&addons)?;
		create_dir(&pkg_cache)?;
		create_dir(&pkg_index_cache)?;
		create_dir(&jars)?;
		create_dir(&logs)?;
		create_dir(&launch_logs)?;

		Ok(Paths {
			base,
			project,
			data,
			internal,
			assets,
			libraries,
			java,
			addons,
			pkg_cache,
			pkg_index_cache,
			jars,
			logs,
			launch_logs,
		})
	}
}
