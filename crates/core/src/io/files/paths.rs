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
	/// Holds game jar files
	pub jars: PathBuf,
	/// Holds authentication data
	pub auth: PathBuf,
	/// Holds log files
	pub logs: PathBuf,
	/// Holds launch log files
	pub launch_logs: PathBuf,
	/// Used for runtime info like PIDs
	pub run: PathBuf,
}

impl Paths {
	/// Create a new Paths object. This will create all of the directories
	/// referenced in the paths if they do not already exist.
	pub fn new() -> anyhow::Result<Paths> {
		let base = BaseDirs::new().ok_or(anyhow!("Failed to create base directories"))?;
		let project = ProjectDirs::from("", "mcvm", "mcvm")
			.ok_or(anyhow!("Failed to create project directories"))?;

		let data = project.data_dir().to_owned();
		let internal = data.join("internal");
		let assets = internal.join("assets");
		let libraries = internal.join("libraries");
		let java = internal.join("java");
		let jars = internal.join("jars");
		let auth = internal.join("auth");
		let logs = data.join("logs");
		let launch_logs = logs.join("launch");
		let run = project
			.runtime_dir()
			.map(|x| x.to_path_buf())
			.unwrap_or(internal.join("run"));

		std::fs::create_dir_all(&data)?;
		std::fs::create_dir_all(project.cache_dir())?;
		std::fs::create_dir_all(project.config_dir())?;
		std::fs::create_dir_all(&internal)?;
		std::fs::create_dir_all(&assets)?;
		std::fs::create_dir_all(&java)?;
		std::fs::create_dir_all(&jars)?;
		std::fs::create_dir_all(&auth)?;
		std::fs::create_dir_all(&logs)?;
		std::fs::create_dir_all(&launch_logs)?;
		std::fs::create_dir_all(&run)?;

		Ok(Paths {
			base,
			project,
			data,
			internal,
			assets,
			libraries,
			java,
			jars,
			auth,
			logs,
			launch_logs,
			run,
		})
	}
}
