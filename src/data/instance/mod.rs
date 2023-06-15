pub mod create;
pub mod launch;

use anyhow::{ensure, Context};
use mcvm_shared::instance::Side;
use reqwest::Client;

use crate::io::files::paths::Paths;
use crate::io::files::update_hardlink;
use crate::io::java::classpath::Classpath;
use crate::io::java::Java;
use crate::io::launch::LaunchOptions;
use crate::io::lock::{Lockfile, LockfileAddon};
use crate::io::options::client::ClientOptions;
use crate::io::options::server::ServerOptions;
use crate::io::{files, snapshot, Later};
use crate::net::fabric_quilt;
use crate::package::eval::{EvalConstants, EvalData, EvalParameters, Routine};
use crate::package::reg::{PkgRegistry, PkgRequest};
use crate::util::json;

use super::addon;
use super::config::instance::ClientWindowConfig;
use super::config::profile::GameModifications;
use super::profile::update::UpdateManager;
use mcvm_shared::addon::{Addon, AddonKind};

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum InstKind {
	Client {
		options: Option<Box<ClientOptions>>,
		window: ClientWindowConfig,
	},
	Server {
		options: Option<Box<ServerOptions>>,
	},
}

impl InstKind {
	/// Convert to the Side enum
	pub fn to_side(&self) -> Side {
		match self {
			Self::Client { .. } => Side::Client,
			Self::Server { .. } => Side::Server,
		}
	}
}

#[derive(Debug)]
pub struct Instance {
	pub kind: InstKind,
	pub id: String,
	modifications: GameModifications,
	launch: LaunchOptions,
	client_json: Later<Box<json::JsonObject>>,
	java: Later<Java>,
	classpath: Option<Classpath>,
	jar_path: Later<PathBuf>,
	main_class: Option<String>,
	datapack_folder: Option<String>,
	snapshot_config: snapshot::Config,
}

impl Instance {
	pub fn new(
		kind: InstKind,
		id: &str,
		modifications: GameModifications,
		launch: LaunchOptions,
		datapack_folder: Option<String>,
		snapshot_config: snapshot::Config,
	) -> Self {
		Self {
			kind,
			id: id.to_owned(),
			modifications,
			launch,
			client_json: Later::new(),
			java: Later::new(),
			classpath: None,
			jar_path: Later::new(),
			main_class: None,
			datapack_folder,
			snapshot_config,
		}
	}

	pub fn get_dir(&self, paths: &Paths) -> PathBuf {
		match &self.kind {
			InstKind::Client { .. } => paths.project.data_dir().join("client").join(&self.id),
			InstKind::Server { .. } => paths.project.data_dir().join("server").join(&self.id),
		}
	}

	pub fn get_subdir(&self, paths: &Paths) -> PathBuf {
		self.get_dir(paths).join(match self.kind {
			InstKind::Client { .. } => ".minecraft",
			InstKind::Server { .. } => "server",
		})
	}

	/// Set the java installation for the instance
	fn add_java(&mut self, version: &str, manager: &UpdateManager) {
		let mut java = manager.java.get().clone();
		java.add_version(version);
		self.java.fill(java);
	}

	async fn get_fabric_quilt(
		&mut self,
		paths: &Paths,
		manager: &UpdateManager,
	) -> anyhow::Result<Classpath> {
		let meta = manager.fq_meta.get();
		let classpath = fabric_quilt::get_classpath(meta, paths, self.kind.to_side());
		self.main_class = Some(
			meta.launcher_meta
				.main_class
				.get_main_class_string(self.kind.to_side())
				.to_owned(),
		);

		Ok(classpath)
	}

	pub fn get_linked_addon_paths(
		&self,
		addon: &Addon,
		paths: &Paths,
	) -> anyhow::Result<Vec<PathBuf>> {
		let inst_dir = self.get_subdir(paths);
		Ok(match addon.kind {
			AddonKind::ResourcePack => {
				if let InstKind::Client { .. } = self.kind {
					vec![inst_dir.join("resourcepacks")]
				} else {
					vec![]
				}
			}
			AddonKind::Mod => vec![inst_dir.join("mods")],
			AddonKind::Plugin => {
				if let InstKind::Server { .. } = self.kind {
					vec![inst_dir.join("plugins")]
				} else {
					vec![]
				}
			}
			AddonKind::Shader => {
				if let InstKind::Client { .. } = self.kind {
					vec![inst_dir.join("shaderpacks")]
				} else {
					vec![]
				}
			}
			AddonKind::Datapack => {
				if let Some(datapack_folder) = &self.datapack_folder {
					vec![inst_dir.join(datapack_folder)]
				} else {
					match self.kind {
						InstKind::Client { .. } => inst_dir
							.join("saves")
							.read_dir()
							.context("Failed to read saves directory")?
							.filter_map(|world| {
								world.map(|world| world.path().join("datapacks")).ok()
							})
							.collect(),
						// TODO: Different world paths in options
						InstKind::Server { .. } => vec![inst_dir.join("world").join("datapacks")],
					}
				}
			}
		})
	}

	fn link_addon(
		dir: &Path,
		addon: &Addon,
		paths: &Paths,
		instance_id: &str,
	) -> anyhow::Result<()> {
		let link = dir.join(addon.file_name.clone());
		let addon_path = addon::get_path(addon, paths, instance_id);
		files::create_leading_dirs(&link)?;
		// These checks are to make sure that we properly link the hardlink to the right location
		// We have to remove the current link since it doesnt let us update it in place
		ensure!(addon_path.exists(), "Addon path does not exist");
		if link.exists() {
			fs::remove_file(&link).context("Failed to remove instance addon file")?;
		}
		update_hardlink(&addon_path, &link).context("Failed to create hard link")?;
		Ok(())
	}

	/// Creates an addon on the instance
	pub fn create_addon(&self, addon: &Addon, paths: &Paths) -> anyhow::Result<()> {
		let inst_dir = self.get_subdir(paths);
		files::create_leading_dirs(&inst_dir)?;
		files::create_dir(&inst_dir)?;
		for path in self
			.get_linked_addon_paths(addon, paths)
			.context("Failed to get linked directory")?
		{
			Self::link_addon(&path, addon, paths, &self.id)
				.with_context(|| format!("Failed to link addon {}", addon.id))?;
		}

		Ok(())
	}

	/// Removes an addon file from this instance
	pub fn remove_addon_file(&self, path: &Path, paths: &Paths) -> anyhow::Result<()> {
		// We check if it is a stored addon path due to the old behavior to put that path in the lockfile.
		// Also some other sanity checks
		if path.exists() && !addon::is_stored_addon_path(path, paths) && !path.is_dir() {
			fs::remove_file(path).context("Failed to remove instance addon file")?;
		}

		Ok(())
	}

	/// Removes the paper server jar file from a server instance
	pub fn remove_paper(&self, paths: &Paths, paper_file_name: String) -> anyhow::Result<()> {
		let inst_dir = self.get_subdir(paths);
		let paper_path = inst_dir.join(paper_file_name);
		if paper_path.exists() {
			fs::remove_file(paper_path).context("Failed to remove Paper jar")?;
		}

		Ok(())
	}

	/// Removes files such as the game jar for when the profile version changes
	pub fn teardown(
		&self,
		paths: &Paths,
		paper_properties: Option<(u16, String)>,
	) -> anyhow::Result<()> {
		match self.kind {
			InstKind::Client { .. } => {
				let inst_dir = self.get_dir(paths);
				let jar_path = inst_dir.join("client.jar");
				if jar_path.exists() {
					fs::remove_file(jar_path).context("Failed to remove client.jar")?;
				}
			}
			InstKind::Server { .. } => {
				let inst_dir = self.get_subdir(paths);
				let jar_path = inst_dir.join("server.jar");
				if jar_path.exists() {
					fs::remove_file(jar_path).context("Failed to remove server.jar")?;
				}

				if let Some((_, file_name)) = paper_properties {
					self.remove_paper(paths, file_name)
						.context("Failed to remove Paper")?;
				}
			}
		}

		Ok(())
	}

	/// Installs a package on this instance
	pub async fn install_package<'a>(
		&self,
		pkg: &PkgRequest,
		pkg_version: u32,
		constants: &'a EvalConstants,
		params: EvalParameters,
		reg: &mut PkgRegistry,
		paths: &Paths,
		lock: &mut Lockfile,
		force: bool,
		client: &Client,
	) -> anyhow::Result<EvalData<'a>> {
		let eval = reg
			.eval(pkg, paths, Routine::Install, constants, params, client)
			.await
			.context("Failed to evaluate package")?;

		let lockfile_addons = eval
			.addon_reqs
			.iter()
			.map(|x| {
				Ok(LockfileAddon::from_addon(
					&x.addon,
					self.get_linked_addon_paths(&x.addon, paths)?
						.iter()
						.map(|y| y.join(x.addon.file_name.clone()))
						.collect(),
				))
			})
			.collect::<anyhow::Result<Vec<LockfileAddon>>>()
			.context("Failed to convert addons to the lockfile format")?;

		let files_to_remove = lock
			.update_package(&pkg.name, &self.id, pkg_version, &lockfile_addons)
			.context("Failed to update package in lockfile")?;

		for addon in eval.addon_reqs.iter() {
			if addon::should_update(&addon.addon, paths, &self.id) || force {
				addon
					.acquire(paths, &self.id, client)
					.await
					.with_context(|| format!("Failed to acquire addon '{}'", addon.addon.id))?;
			}
			self.create_addon(&addon.addon, paths)
				.with_context(|| format!("Failed to install addon '{}'", addon.addon.id))?;
		}

		for path in files_to_remove {
			self.remove_addon_file(&path, paths)
				.context("Failed to remove addon file from instance")?;
		}

		Ok(eval)
	}

	/// Starts snapshot interactions by generating the path and opening the index
	pub fn open_snapshot_index(&self, paths: &Paths) -> anyhow::Result<(PathBuf, snapshot::Index)> {
		let snapshot_dir = snapshot::get_snapshot_directory(&self.id, paths);
		let index = snapshot::Index::open(&snapshot_dir)?;
		Ok((snapshot_dir, index))
	}

	/// Creates a new snapshot for this instance
	pub fn create_snapshot(
		&self,
		id: String,
		kind: snapshot::SnapshotKind,
		paths: &Paths,
	) -> anyhow::Result<()> {
		let (snapshot_dir, mut index) = self.open_snapshot_index(paths)?;

		index.create_snapshot(
			kind,
			id,
			&self.snapshot_config,
			&self.id,
			&self.get_subdir(paths),
			paths,
		)?;

		index.finish(&snapshot_dir)?;
		Ok(())
	}

	/// Removes a snapshot from this instance
	pub fn remove_snapshot(&self, id: &str, paths: &Paths) -> anyhow::Result<()> {
		let (snapshot_dir, mut index) = self.open_snapshot_index(paths)?;

		index.remove_snapshot(id, &self.id, paths)?;

		index.finish(&snapshot_dir)?;
		Ok(())
	}

	/// Restores a snapshot for this instance
	pub async fn restore_snapshot(&self, id: &str, paths: &Paths) -> anyhow::Result<()> {
		let (snapshot_dir, index) = self.open_snapshot_index(paths)?;

		index
			.restore_snapshot(id, &self.id, &self.get_subdir(paths), paths)
			.await?;

		index.finish(&snapshot_dir)?;
		Ok(())
	}
}
