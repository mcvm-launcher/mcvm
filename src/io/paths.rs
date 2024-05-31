use anyhow::{anyhow, Context};
use directories::{BaseDirs, ProjectDirs};

use std::path::PathBuf;

/// Store for all of the paths that are used throughout the application
#[derive(Debug, Clone)]
pub struct Paths {
	/// System-wide directories
	pub base: BaseDirs,
	/// Project-specific directories
	pub project: ProjectDirs,
	/// Paths object from core
	pub core: mcvm_core::Paths,
	/// Holds program data
	pub data: PathBuf,
	/// Holds internal data
	pub internal: PathBuf,
	/// Holds addons
	pub addons: PathBuf,
	/// Holds cached package scripts
	pub pkg_cache: PathBuf,
	/// Holds cached package repository indexes
	pub pkg_index_cache: PathBuf,
	/// Holds log files
	pub logs: PathBuf,
	/// Holds launch log files
	pub launch_logs: PathBuf,
	/// Used for runtime info like PIDs
	pub run: PathBuf,
	/// Storing instance snapshots
	pub snapshots: PathBuf,
	/// Storing Fabric and Quilt data
	pub fabric_quilt: PathBuf,
	/// Storing proxy data
	pub proxy: PathBuf,
	/// Holding user plugins
	pub plugins: PathBuf,
}

impl Paths {
	/// Create a new Paths object and also create all of the paths it contains on the filesystem
	pub async fn new() -> anyhow::Result<Paths> {
		let out = Self::new_no_create()?;
		out.create_dirs().await?;

		Ok(out)
	}

	/// Create the directories on an existing set of paths
	pub async fn create_dirs(&self) -> anyhow::Result<()> {
		tokio::try_join!(
			tokio::fs::create_dir_all(&self.data),
			tokio::fs::create_dir_all(self.project.cache_dir()),
			tokio::fs::create_dir_all(self.project.config_dir()),
			tokio::fs::create_dir_all(&self.internal),
			tokio::fs::create_dir_all(&self.addons),
			tokio::fs::create_dir_all(&self.pkg_cache),
			tokio::fs::create_dir_all(&self.pkg_index_cache),
			tokio::fs::create_dir_all(&self.logs),
			tokio::fs::create_dir_all(&self.launch_logs),
			tokio::fs::create_dir_all(&self.run),
			tokio::fs::create_dir_all(&self.snapshots),
			tokio::fs::create_dir_all(&self.fabric_quilt),
			tokio::fs::create_dir_all(&self.proxy),
			tokio::fs::create_dir_all(&self.plugins),
		)?;
		self.core.create_dirs()?;

		Ok(())
	}

	/// Create the paths without creating any directories
	pub fn new_no_create() -> anyhow::Result<Self> {
		let base = BaseDirs::new().ok_or(anyhow!("Base directories failed"))?;
		let project =
			ProjectDirs::from("", "mcvm", "mcvm").ok_or(anyhow!("Base directories failed"))?;

		let data = project.data_dir().to_owned();
		let internal = data.join("internal");
		let addons = internal.join("addons");
		let pkg_cache = project.cache_dir().join("pkg");
		let pkg_index_cache = pkg_cache.join("index");
		let logs = data.join("logs");
		let launch_logs = logs.join("launch");
		let run = project
			.runtime_dir()
			.map(|x| x.to_path_buf())
			.unwrap_or(internal.join("run"));
		let snapshots = internal.join("snapshots");
		let fabric_quilt = internal.join("fabric_quilt");
		let proxy = data.join("proxy");
		let plugins = data.join("plugins");

		let core_paths = mcvm_core::Paths::new().context("Failed to create core paths")?;

		Ok(Paths {
			base,
			project,
			core: core_paths,
			data,
			internal,
			addons,
			pkg_cache,
			pkg_index_cache,
			logs,
			launch_logs,
			run,
			snapshots,
			fabric_quilt,
			proxy,
			plugins,
		})
	}
}
