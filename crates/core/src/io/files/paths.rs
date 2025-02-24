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
		let out = Self::new_no_create()?;
		out.create_dirs()?;

		Ok(out)
	}

	/// Create the directories on an existing set of paths
	pub fn create_dirs(&self) -> anyhow::Result<()> {
		let _ = std::fs::create_dir_all(&self.data);
		let _ = std::fs::create_dir_all(self.project.cache_dir());
		let _ = std::fs::create_dir_all(self.project.config_dir());
		let _ = std::fs::create_dir_all(&self.internal);
		let _ = std::fs::create_dir_all(&self.assets);
		let _ = std::fs::create_dir_all(&self.java);
		let _ = std::fs::create_dir_all(&self.jars);
		let _ = std::fs::create_dir_all(&self.auth);
		let _ = std::fs::create_dir_all(&self.logs);
		let _ = std::fs::create_dir_all(&self.launch_logs);
		let _ = std::fs::create_dir_all(&self.run);

		Ok(())
	}

	/// Create the paths without creating any directories
	pub fn new_no_create() -> anyhow::Result<Self> {
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
