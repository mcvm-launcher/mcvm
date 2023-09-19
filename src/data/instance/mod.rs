/// Creation of instance contents
pub mod create;
/// Launching an instance
pub mod launch;

use anyhow::{bail, ensure, Context};
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::pkg::ArcPkgReq;
use mcvm_shared::Side;
use reqwest::Client;

use crate::io::files::paths::Paths;
use crate::io::files::update_hardlink;
use crate::io::java::classpath::Classpath;
use crate::io::java::install::JavaInstallation;
use crate::io::lock::{Lockfile, LockfileAddon};
use crate::io::options::client::ClientOptions;
use crate::io::options::server::ServerOptions;
use crate::io::{files, snapshot};
use crate::net::fabric_quilt;
use crate::package::eval::{EvalData, EvalInput, Routine};
use crate::package::reg::PkgRegistry;
use mcvm_shared::later::Later;

use self::launch::LaunchOptions;

use super::addon;
use super::config::instance::ClientWindowConfig;
use super::config::package::PackageConfig;
use super::config::profile::GameModifications;
use super::config::profile::ProfilePackageConfiguration;
use super::id::InstanceID;
use super::profile::update::manager::UpdateManager;
use mcvm_shared::addon::{Addon, AddonKind};

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// An instance of the game on a profile
#[derive(Debug)]
pub struct Instance {
	/// What type of instance this is
	pub kind: InstKind,
	/// The ID of this instance
	pub id: InstanceID,
	config: InstanceStoredConfig,
	/// Directories of the instance
	pub dirs: Later<InstanceDirs>,
	java: Later<JavaInstallation>,
	classpath: Option<Classpath>,
	jar_path: Later<PathBuf>,
	main_class: Option<String>,
}

/// Different kinds of instances and their associated data
#[derive(Debug, Clone)]
pub enum InstKind {
	/// A client instance
	Client {
		/// Options for the client
		options: Option<Box<ClientOptions>>,
		/// Configuration for the client window
		window: ClientWindowConfig,
	},
	/// A server instance
	Server {
		/// Options for the server
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

/// The stored configuration on an instance
#[derive(Debug)]
pub struct InstanceStoredConfig {
	/// Modifications to the instance
	pub modifications: GameModifications,
	/// Launch options for the instance
	pub launch: LaunchOptions,
	/// The instance's global datapack folder
	pub datapack_folder: Option<String>,
	/// The instance's snapshot configuration
	pub snapshot_config: snapshot::Config,
	/// The packages on the instance
	pub packages: Vec<PackageConfig>,
}

/// Directories that an instance uses
#[derive(Debug)]
pub struct InstanceDirs {
	/// The base instance directory
	pub inst_dir: PathBuf,
	/// The game directory, such as .minecraft, relative to the instance directory
	pub game_dir: PathBuf,
}

impl InstanceDirs {
	/// Create a new InstanceDirs
	pub fn new(paths: &Paths, id: &str, side: &Side) -> Self {
		let inst_dir = match side {
			Side::Client { .. } => paths.project.data_dir().join("client").join(id),
			Side::Server { .. } => paths.project.data_dir().join("server").join(id),
		};

		let game_dir = inst_dir.join(match side {
			Side::Client { .. } => ".minecraft",
			Side::Server { .. } => "server",
		});

		Self { inst_dir, game_dir }
	}

	/// Make sure the directories exist
	pub fn ensure_exist(&self) -> anyhow::Result<()> {
		files::create_leading_dirs(&self.inst_dir)?;
		files::create_dir(&self.inst_dir).context("Failed to create instance directory")?;
		files::create_dir(&self.game_dir).context("Failed to create game directory")?;
		Ok(())
	}
}

impl Instance {
	/// Create a new instance
	pub fn new(kind: InstKind, id: InstanceID, config: InstanceStoredConfig) -> Self {
		Self {
			kind,
			id,
			config,
			dirs: Later::Empty,
			java: Later::new(),
			classpath: None,
			jar_path: Later::new(),
			main_class: None,
		}
	}

	/// Ensure the directories are set and exist
	fn ensure_dirs(&mut self, paths: &Paths) -> anyhow::Result<()> {
		self.dirs
			.ensure_full(|| InstanceDirs::new(paths, &self.id, &self.kind.to_side()));
		self.dirs.get().ensure_exist()?;

		Ok(())
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
				.to_string(),
		);

		Ok(classpath)
	}

	/// Get the paths on this instance to hardlink an addon to
	pub fn get_linked_addon_paths(
		&mut self,
		addon: &Addon,
		paths: &Paths,
	) -> anyhow::Result<Vec<PathBuf>> {
		self.ensure_dirs(paths)?;
		let inst_dir = &self.dirs.get().inst_dir;
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
				if let Some(datapack_folder) = &self.config.datapack_folder {
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
	pub fn create_addon(&mut self, addon: &Addon, paths: &Paths) -> anyhow::Result<()> {
		self.ensure_dirs(paths)?;
		let game_dir = &self.dirs.get().game_dir;
		files::create_leading_dirs(game_dir)?;
		files::create_dir(game_dir)?;
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
	pub fn remove_paper(&mut self, paths: &Paths, paper_file_name: String) -> anyhow::Result<()> {
		self.ensure_dirs(paths)?;
		let game_dir = &self.dirs.get().game_dir;
		let paper_path = game_dir.join(paper_file_name);
		if paper_path.exists() {
			fs::remove_file(paper_path).context("Failed to remove Paper jar")?;
		}

		Ok(())
	}

	/// Removes files such as the game jar for when the profile version changes
	pub fn teardown(
		&mut self,
		paths: &Paths,
		paper_properties: Option<(u16, String)>,
	) -> anyhow::Result<()> {
		self.ensure_dirs(paths)?;
		match self.kind {
			InstKind::Client { .. } => {
				let inst_dir = &self.dirs.get().inst_dir;
				let jar_path = inst_dir.join("client.jar");
				if jar_path.exists() {
					fs::remove_file(jar_path).context("Failed to remove client.jar")?;
				}
			}
			InstKind::Server { .. } => {
				let game_dir = &self.dirs.get().game_dir;
				let jar_path = game_dir.join("server.jar");
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
		&mut self,
		pkg: &ArcPkgReq,
		eval_input: EvalInput<'a>,
		reg: &mut PkgRegistry,
		paths: &Paths,
		lock: &mut Lockfile,
		force: bool,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<EvalData<'a>> {
		let eval = reg
			.eval(pkg, paths, Routine::Install, eval_input, client)
			.await
			.context("Failed to evaluate package")?;

		if eval.uses_custom_instructions {
			o.display(
				MessageContents::Warning(
					"Package uses custom instructions that MCVM does not recognize".into(),
				),
				MessageLevel::Important,
			);
		}

		// Run commands
		if !eval.commands.is_empty() {
			o.display(
				MessageContents::StartProcess("Running commands".into()),
				MessageLevel::Important,
			);

			for command_and_args in &eval.commands {
				let program = command_and_args
					.first()
					.expect("Command should contain at least the program");
				let mut command = std::process::Command::new(program);
				command.args(&command_and_args[1..]);
				let mut child = command
					.spawn()
					.context("Failed to spawn command {program}")?;
				let result = child.wait()?;
				if !result.success() {
					bail!("Command {program} returned a non-zero exit code");
				}
			}

			o.display(
				MessageContents::Success("Finished running commands".into()),
				MessageLevel::Important,
			);
		}

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
			.update_package(&pkg.id, &self.id, &lockfile_addons, o)
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

	/// Collects all the configured packages for this instance
	pub fn get_configured_packages<'a>(
		&'a self,
		global: &'a [PackageConfig],
		profile: &'a ProfilePackageConfiguration,
	) -> Vec<&'a PackageConfig> {
		// We use a map so that we can override packages from more general sources
		// with those from more specific ones
		let mut map = HashMap::new();
		for pkg in global {
			map.insert(pkg.get_pkg_id(), pkg);
		}
		for pkg in profile.iter_global() {
			map.insert(pkg.get_pkg_id(), pkg);
		}
		for pkg in profile.iter_side(self.kind.to_side()) {
			map.insert(pkg.get_pkg_id(), pkg);
		}
		for pkg in &self.config.packages {
			map.insert(pkg.get_pkg_id(), pkg);
		}

		let mut out = Vec::new();
		for pkg in map.values() {
			out.push(*pkg);
		}

		out
	}

	/// Starts snapshot interactions by generating the path and opening the index
	pub fn open_snapshot_index(&self, paths: &Paths) -> anyhow::Result<(PathBuf, snapshot::Index)> {
		let snapshot_dir = snapshot::get_snapshot_directory(&self.id, paths);
		let index =
			snapshot::Index::open(&snapshot_dir).context("Failed to open snapshot index")?;
		Ok((snapshot_dir, index))
	}

	/// Creates a new snapshot for this instance
	pub fn create_snapshot(
		&mut self,
		id: String,
		kind: snapshot::SnapshotKind,
		paths: &Paths,
	) -> anyhow::Result<()> {
		self.ensure_dirs(paths)?;
		let (snapshot_dir, mut index) = self.open_snapshot_index(paths)?;

		index.create_snapshot(
			kind,
			id,
			&self.config.snapshot_config,
			&self.id,
			&self.dirs.get().game_dir,
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
	pub async fn restore_snapshot(&mut self, id: &str, paths: &Paths) -> anyhow::Result<()> {
		self.ensure_dirs(paths)?;
		let (snapshot_dir, index) = self.open_snapshot_index(paths)?;

		index
			.restore_snapshot(id, &self.id, &self.dirs.get().game_dir, paths)
			.await?;

		index.finish(&snapshot_dir)?;
		Ok(())
	}
}
