/// Addon-related functions for instances
mod addons;
/// Creation of instance contents
pub mod create;
/// Launching an instance
pub mod launch;

use anyhow::{bail, Context};
use mcvm_core::instance::WindowResolution;
use mcvm_core::io::java::classpath::Classpath;
use mcvm_core::launch::LaunchConfiguration;
use mcvm_core::version::InstalledVersion;
use mcvm_core::QuickPlayType;
use mcvm_mods::fabric_quilt;
use mcvm_options::client::ClientOptions;
use mcvm_options::server::ServerOptions;
use mcvm_shared::later::Later;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::pkg::ArcPkgReq;
use mcvm_shared::versions::VersionInfo;
use mcvm_shared::Side;
use reqwest::Client;

use crate::io::files::paths::Paths;
use crate::io::lock::{Lockfile, LockfileAddon};
use crate::io::{files, snapshot};
use crate::pkg::eval::{EvalData, EvalInput, Routine};
use crate::pkg::reg::PkgRegistry;

use self::launch::LaunchOptions;

use super::addon;
use super::config::instance::{ClientWindowConfig, QuickPlay};
use super::config::package::PackageConfig;
use super::config::profile::GameModifications;
use super::id::InstanceID;
use super::profile::update::manager::UpdateManager;

use std::fs;
use std::path::PathBuf;

/// An instance of the game on a profile
#[derive(Debug)]
pub struct Instance {
	/// What type of instance this is
	pub kind: InstKind,
	/// The ID of this instance
	pub id: InstanceID,
	/// Directories of the instance
	pub dirs: Later<InstanceDirs>,
	/// Configuration for the instance
	config: InstanceStoredConfig,
	/// Modification data
	modification_data: ModificationData,
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
	/// The packages on the instance, consolidated from all parent sources
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

/// Things that modifications for an instance change when creating it
#[derive(Debug)]
struct ModificationData {
	/// Override for the main class from modifications
	pub main_class_override: Option<String>,
	/// Override for the Jar path from modifications
	pub jar_path_override: Option<PathBuf>,
	/// Extension for the classpath from modifications
	pub classpath_extension: Classpath,
}

impl ModificationData {
	pub fn new() -> Self {
		Self {
			main_class_override: None,
			jar_path_override: None,
			classpath_extension: Classpath::new(),
		}
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
			modification_data: ModificationData::new(),
		}
	}

	/// Ensure the directories are set and exist
	pub fn ensure_dirs(&mut self, paths: &Paths) -> anyhow::Result<()> {
		self.dirs
			.ensure_full(|| InstanceDirs::new(paths, &self.id, &self.kind.to_side()));
		self.dirs.get().ensure_exist()?;

		Ok(())
	}

	/// Create the core instance
	async fn create_core_instance<'core>(
		&mut self,
		version: &'core mut InstalledVersion<'core, 'core>,
		paths: &Paths,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<mcvm_core::Instance<'core>> {
		self.ensure_dirs(paths)?;
		let side = match &self.kind {
			InstKind::Client { window, .. } => mcvm_core::InstanceKind::Client {
				window: mcvm_core::ClientWindowConfig {
					resolution: window
						.resolution
						.map(|x| WindowResolution::new(x.width, x.height)),
				},
			},
			InstKind::Server { .. } => mcvm_core::InstanceKind::Server {
				create_eula: true,
				show_gui: false,
			},
		};
		let quick_play = match self.config.launch.quick_play.clone() {
			QuickPlay::None => QuickPlayType::None,
			QuickPlay::Server { server, port } => QuickPlayType::Server { server, port },
			QuickPlay::World { world } => QuickPlayType::World { world },
			QuickPlay::Realm { realm } => QuickPlayType::Realm { realm },
		};
		let wrapper = self
			.config
			.launch
			.wrapper
			.as_ref()
			.map(|x| mcvm_core::WrapperCommand {
				cmd: x.cmd.clone(),
				args: x.args.clone(),
			});
		let launch_config = LaunchConfiguration {
			java: self.config.launch.java.clone(),
			jvm_args: self.config.launch.jvm_args.clone(),
			game_args: self.config.launch.game_args.clone(),
			min_mem: self.config.launch.min_mem.clone(),
			max_mem: self.config.launch.max_mem.clone(),
			preset: self.config.launch.preset,
			env: self.config.launch.env.clone(),
			wrappers: Vec::from_iter(wrapper),
			quick_play,
			use_log4j_config: self.config.launch.use_log4j_config,
		};
		let config = mcvm_core::InstanceConfiguration {
			side,
			path: self.dirs.get().game_dir.clone(),
			launch: launch_config,
			jar_path: self.modification_data.jar_path_override.clone(),
			main_class: self.modification_data.main_class_override.clone(),
			additional_libs: self.modification_data.classpath_extension.get_paths(),
		};
		let inst = version
			.get_instance(config, o)
			.await
			.context("Failed to initialize instance")?;
		Ok(inst)
	}

	async fn get_fabric_quilt(
		&mut self,
		paths: &Paths,
		manager: &UpdateManager,
	) -> anyhow::Result<Classpath> {
		let meta = manager.fq_meta.get();
		let classpath = fabric_quilt::get_classpath(meta, &paths.core, self.kind.to_side());
		self.modification_data.main_class_override = Some(
			meta.launcher_meta
				.main_class
				.get_main_class_string(self.kind.to_side())
				.to_string(),
		);

		Ok(classpath)
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
		let version_info = VersionInfo {
			version: eval_input.constants.version.clone(),
			versions: eval_input.constants.version_list.clone(),
		};

		// Get the configuration for the package or the default if it is not configured by the user
		let pkg_config = self
			.get_package_config(&pkg.id)
			.cloned()
			.unwrap_or_else(|| PackageConfig::Basic(pkg.id.clone()));

		let eval = reg
			.eval(pkg, paths, Routine::Install, eval_input, client, o)
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
					self.get_linked_addon_paths(
						&x.addon,
						&pkg_config.get_worlds(),
						paths,
						&version_info,
					)?
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
			self.create_addon(&addon.addon, &pkg_config.get_worlds(), paths, &version_info)
				.with_context(|| format!("Failed to install addon '{}'", addon.addon.id))?;
		}

		for path in files_to_remove {
			self.remove_addon_file(&path, paths)
				.context("Failed to remove addon file from instance")?;
		}

		Ok(eval)
	}

	/// Gets all of the configured packages for this instance
	pub fn get_configured_packages(&self) -> &Vec<PackageConfig> {
		&self.config.packages
	}

	/// Gets the configuration for a specific package on this instance
	pub fn get_package_config(&self, package: &str) -> Option<&PackageConfig> {
		let configured_packages = self.get_configured_packages();
		let package_config = configured_packages
			.into_iter()
			.find(|x| x.get_pkg_id() == package.into());
		package_config
	}
}

/// Snapshot-related functions
impl Instance {
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
