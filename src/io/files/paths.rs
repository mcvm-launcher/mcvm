use super::create_leading_dirs_async;

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
	pub async fn new() -> anyhow::Result<Paths> {
		let base = BaseDirs::new().ok_or(anyhow!("Base directories failed"))?;
		let project =
			ProjectDirs::from("", "mcvm", "mcvm").ok_or(anyhow!("Base directories failed"))?;

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

		create_leading_dirs_async(&data).await?;
		create_leading_dirs_async(project.cache_dir()).await?;
		create_leading_dirs_async(project.config_dir()).await?;
		create_leading_dirs_async(
			project
				.runtime_dir()
				.ok_or(anyhow!("Base directories failed"))?,
		).await.context("Failed to create run directory")?;
		create_leading_dirs_async(&internal).await?;
		create_leading_dirs_async(&assets).await?;
		create_leading_dirs_async(&java).await?;
		create_leading_dirs_async(&addons).await?;
		create_leading_dirs_async(&pkg_cache).await?;
		create_leading_dirs_async(&pkg_index_cache).await?;
		create_leading_dirs_async(&jars).await?;
		create_leading_dirs_async(&logs).await?;
		create_leading_dirs_async(&launch_logs).await?;

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
