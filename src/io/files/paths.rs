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
}

impl Paths {
	/// Create a new Paths object and also create all of the paths it contains on the filesystem
	pub async fn new() -> anyhow::Result<Paths> {
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

		tokio::try_join!(
			tokio::fs::create_dir_all(&data),
			tokio::fs::create_dir_all(project.cache_dir()),
			tokio::fs::create_dir_all(project.config_dir()),
			tokio::fs::create_dir_all(&internal),
			tokio::fs::create_dir_all(&addons),
			tokio::fs::create_dir_all(&pkg_cache),
			tokio::fs::create_dir_all(&pkg_index_cache),
			tokio::fs::create_dir_all(&logs),
			tokio::fs::create_dir_all(&launch_logs),
			tokio::fs::create_dir_all(&run),
			tokio::fs::create_dir_all(&snapshots),
			tokio::fs::create_dir_all(&fabric_quilt),
			tokio::fs::create_dir_all(&proxy),
		)?;

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
		})
	}
}
