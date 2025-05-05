/// Creation of the client
mod client;
/// Creation of the server
mod server;

use std::collections::HashSet;
use std::fs;
use std::ops::DerefMut;
use std::path::PathBuf;

use anyhow::{bail, Context};
use mcvm_config::instance::QuickPlay;
use mcvm_core::instance::WindowResolution;
use mcvm_core::io::java::classpath::Classpath;
use mcvm_core::io::json_to_file;
use mcvm_core::launch::LaunchConfiguration;
use mcvm_core::user::uuid::hyphenate_uuid;
use mcvm_core::user::{User, UserManager};
use mcvm_core::version::InstalledVersion;
use mcvm_core::QuickPlayType;
use mcvm_plugin::hooks::{OnInstanceSetup, OnInstanceSetupArg, RemoveGameModification};
use mcvm_shared::output::OutputProcess;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::translate;
use mcvm_shared::Side;

use crate::io::lock::Lockfile;
use crate::io::paths::Paths;
use crate::plugin::PluginManager;

use super::update::manager::{UpdateManager, UpdateMethodResult, UpdateRequirement};
use super::{InstKind, Instance};

/// The default main class for the server
pub const DEFAULT_SERVER_MAIN_CLASS: &str = "net.minecraft.server.Main";

impl Instance {
	/// Get the requirements for this instance
	pub fn get_requirements(&self) -> HashSet<UpdateRequirement> {
		let mut out = HashSet::new();
		match &self.kind {
			InstKind::Client { .. } => {
				if self.config.launch.use_log4j_config {
					out.insert(UpdateRequirement::ClientLoggingConfig);
				}
			}
			InstKind::Server { .. } => {}
		}
		out
	}

	/// Setup the data and folders for the instance, preparing it for launch
	pub async fn setup<'core>(
		&mut self,
		manager: &'core mut UpdateManager,
		plugins: &PluginManager,
		paths: &Paths,
		users: &UserManager,
		lock: &mut Lockfile,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<UpdateMethodResult> {
		// Start by setting up side-specific stuff
		let result = match &self.kind {
			InstKind::Client { .. } => {
				o.display(
					MessageContents::Header(translate!(o, StartUpdatingClient)),
					MessageLevel::Important,
				);
				o.start_section();
				let result = self
					.setup_client(paths, users)
					.await
					.context("Failed to create client")?;
				Ok::<_, anyhow::Error>(result)
			}
			InstKind::Server { .. } => {
				o.display(
					MessageContents::Header(translate!(o, StartUpdatingServer)),
					MessageLevel::Important,
				);
				o.start_section();
				let result = self
					.setup_server(paths)
					.await
					.context("Failed to create server")?;
				Ok(result)
			}
		}?;

		// Run plugin setup hooks
		self.ensure_dirs(paths)?;

		lock.ensure_instance_created(&self.id, &manager.version_info.get().version);
		let lock_instance = lock.get_instance(&self.id);
		let current_version = lock_instance.map(|x| x.version.clone());
		let current_game_mod_version =
			lock_instance.and_then(|x| x.game_modification_version.clone());
		let current_client_type = lock_instance
			.map(|x| x.client_type.clone())
			.unwrap_or_default();
		let current_server_type = lock_instance
			.map(|x| x.server_type.clone())
			.unwrap_or_default();

		let mut arg = OnInstanceSetupArg {
			id: self.id.to_string(),
			side: Some(self.get_side()),
			game_dir: self.dirs.get().game_dir.to_string_lossy().to_string(),
			version_info: manager.version_info.get_clone(),
			client_type: self.config.modifications.client_type(),
			server_type: self.config.modifications.server_type(),
			current_game_modification_version: current_game_mod_version,
			desired_game_modification_version: self.config.modification_version.clone(),
			custom_config: self.config.plugin_config.clone(),
			internal_dir: paths.internal.to_string_lossy().to_string(),
			update_depth: manager.settings.depth,
		};

		// Do game modification and version change checks
		let is_version_different = current_version
			.as_ref()
			.is_some_and(|x| x != &manager.version_info.get().version);
		let is_game_mod_different = self.config.modifications.client_type() != current_client_type
			|| self.config.modifications.server_type() != current_server_type;
		if is_version_different || is_game_mod_different {
			let mut process = OutputProcess::new(o);
			if is_game_mod_different {
				let message = MessageContents::StartProcess(translate!(
					process,
					StartUpdatingInstanceGameModification
				));
				process.display(message, MessageLevel::Important);
			} else if is_version_different {
				let message = MessageContents::StartProcess(translate!(
					process,
					StartUpdatingInstanceVersion,
					"version1" = &current_version.as_ref().expect("Version should exist"),
					"version2" = &manager.version_info.get().version
				));
				process.display(message, MessageLevel::Important);
			}

			// Teardown
			self.teardown(paths)
				.context("Failed to teardown instance")?;

			arg.client_type = current_client_type;
			arg.server_type = current_server_type;
			if let Some(current_version) = &current_version {
				arg.version_info.version = current_version.clone();
			}
			let results = plugins
				.call_hook(RemoveGameModification, &arg, paths, process.deref_mut())
				.context("Failed to call remove game modification hook")?;

			for result in results {
				result.result(process.deref_mut())?;
			}

			// The current game modification version is no longer valid as it is referring to the old game modification
			arg.current_game_modification_version = None;
			arg.client_type = self.config.modifications.client_type();
			arg.server_type = self.config.modifications.server_type();
			arg.version_info.version = manager.version_info.get().version.clone();

			let message =
				MessageContents::Success(translate!(process, FinishUpdatingInstanceVersion));
			process.display(message, MessageLevel::Important);
		}

		let results = plugins
			.call_hook(OnInstanceSetup, &arg, paths, o)
			.context("Failed to call instance setup hook")?;

		let mut game_mod_version_set = false;
		for result in results {
			let result = result.result(o)?;
			self.modification_data
				.classpath_extension
				.add_multiple(result.classpath_extension.iter());

			if let Some(main_class) = result.main_class_override {
				if self.modification_data.main_class_override.is_none() {
					self.modification_data.main_class_override = Some(main_class);
				} else {
					bail!("Multiple plugins overwrote the main class");
				}
			}

			if let Some(jar_path) = result.jar_path_override {
				if self.modification_data.jar_path_override.is_none() {
					self.modification_data.jar_path_override = Some(PathBuf::from(jar_path));
				} else {
					bail!("Multiple plugins overwrote the JAR path");
				}
			}

			if let Some(game_modification_version) = result.game_modification_version {
				if game_mod_version_set {
					bail!("Multiple plugins attempted to modify the game modification version");
				}
				lock.update_instance_game_modification_version(
					&self.id,
					Some(game_modification_version),
				)
				.expect("Instance should exist");
				game_mod_version_set = true;
			}
		}

		// Update the game modifications and version
		lock.update_instance_version(&self.id, &manager.version_info.get().version)
			.expect("Instance should exist");
		lock.update_instance_game_modifications(
			&self.id,
			self.config.modifications.client_type(),
			self.config.modifications.server_type(),
		)
		.expect("Instance should exist");

		lock.finish(paths)
			.context("Failed to finish using lockfile")?;

		// Make the core instance
		let mut version = manager
			.get_core_version(o)
			.await
			.context("Failed to get manager version")?;

		self.create_core_instance(&mut version, paths, o)
			.await
			.context("Failed to create core instance")?;
		o.end_section();

		Ok(result)
	}

	/// Ensure the directories are set and exist
	pub fn ensure_dirs(&mut self, paths: &Paths) -> anyhow::Result<()> {
		self.dirs
			.ensure_full(|| InstanceDirs::new(paths, &self.id, &self.kind.to_side()));
		self.dirs.get().ensure_exist()?;

		Ok(())
	}

	/// Create the core instance
	pub(super) async fn create_core_instance<'core>(
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

	/// Removes files such as the game jar for when the profile version changes
	pub fn teardown(&mut self, paths: &Paths) -> anyhow::Result<()> {
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
			}
		}

		Ok(())
	}

	/// Create a keypair file in the instance
	fn create_keypair(&mut self, user: &User, paths: &Paths) -> anyhow::Result<()> {
		if let Some(uuid) = user.get_uuid() {
			if let Some(keypair) = user.get_keypair() {
				self.ensure_dirs(paths)?;
				let keys_dir = self.dirs.get().game_dir.join("profilekeys");
				let hyphenated_uuid = hyphenate_uuid(uuid).context("Failed to hyphenate UUID")?;
				let path = keys_dir.join(format!("{hyphenated_uuid}.json"));
				mcvm_core::io::files::create_leading_dirs(&path)?;

				json_to_file(path, keypair).context("Failed to write keypair to file")?;
			}
		}

		Ok(())
	}
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
	pub fn new(paths: &Paths, instance_id: &str, side: &Side) -> Self {
		let inst_dir = paths.project.data_dir().join("instances").join(instance_id);

		let game_dir = match side {
			Side::Client => inst_dir.join(".minecraft"),
			Side::Server => inst_dir.clone(),
		};

		Self { inst_dir, game_dir }
	}

	/// Make sure the directories exist
	pub fn ensure_exist(&self) -> anyhow::Result<()> {
		std::fs::create_dir_all(&self.inst_dir).context("Failed to create instance directory")?;
		std::fs::create_dir_all(&self.game_dir)
			.context("Failed to create instance game directory")?;
		Ok(())
	}
}

/// Things that modifications for an instance change when creating it
#[derive(Debug)]
pub struct ModificationData {
	/// Override for the main class from modifications
	pub main_class_override: Option<String>,
	/// Override for the Jar path from modifications
	pub jar_path_override: Option<PathBuf>,
	/// Extension for the classpath from modifications
	pub classpath_extension: Classpath,
}

impl ModificationData {
	/// Create a new ModificationData with default parameters
	pub fn new() -> Self {
		Self {
			main_class_override: None,
			jar_path_override: None,
			classpath_extension: Classpath::new(),
		}
	}
}

impl Default for ModificationData {
	fn default() -> Self {
		Self::new()
	}
}
