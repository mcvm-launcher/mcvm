/// Creation of the client
mod client;
/// Creation of the server
mod server;

use std::collections::HashSet;
use std::fs::{self, File};
use std::path::PathBuf;

use anyhow::Context;
use mcvm_core::instance::WindowResolution;
use mcvm_core::io::java::classpath::Classpath;
use mcvm_core::launch::LaunchConfiguration;
use mcvm_core::user::uuid::hyphenate_uuid;
use mcvm_core::user::{User, UserManager};
use mcvm_core::version::InstalledVersion;
use mcvm_core::QuickPlayType;
use mcvm_mods::fabric_quilt;
use mcvm_shared::lang::translate::TranslationKey;
use mcvm_shared::modifications::Modloader;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::translate;
use mcvm_shared::Side;
use reqwest::Client;

use crate::data::config::instance::QuickPlay;
use crate::data::profile::update::manager::{UpdateManager, UpdateMethodResult, UpdateRequirement};
use crate::io::files::paths::Paths;

use super::{InstKind, Instance};

/// The default main class for the server
pub const DEFAULT_SERVER_MAIN_CLASS: &str = "net.minecraft.server.Main";
/// The main class for a Paper server
pub const PAPER_SERVER_MAIN_CLASS: &str = "io.papermc.paperclip.Main";

impl Instance {
	/// Get the requirements for this instance
	pub fn get_requirements(&self) -> HashSet<UpdateRequirement> {
		let mut out = HashSet::new();
		match self.config.modifications.get_modloader(self.kind.to_side()) {
			Modloader::Fabric => {
				out.insert(UpdateRequirement::FabricQuilt(
					fabric_quilt::Mode::Fabric,
					self.kind.to_side(),
				));
			}
			Modloader::Quilt => {
				out.insert(UpdateRequirement::FabricQuilt(
					fabric_quilt::Mode::Quilt,
					self.kind.to_side(),
				));
			}
			_ => {}
		};
		out.insert(UpdateRequirement::Options);
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

	/// Create the data for the instance.
	pub async fn create<'core>(
		&mut self,
		manager: &'core mut UpdateManager,
		paths: &Paths,
		users: &UserManager,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<UpdateMethodResult> {
		// Start by setting up custom changes
		let result = match &self.kind {
			InstKind::Client { .. } => {
				o.display(
					MessageContents::Header(translate!(o, StartUpdatingClient, "id" = &self.id)),
					MessageLevel::Important,
				);
				o.start_section();
				let result = self
					.create_client(manager, paths, users)
					.await
					.context("Failed to create client")?;
				Ok::<_, anyhow::Error>(result)
			}
			InstKind::Server { .. } => {
				o.display(
					MessageContents::Header(translate!(o, StartUpdatingServer, "id" = &self.id)),
					MessageLevel::Important,
				);
				o.start_section();
				let result = self
					.create_server(manager, paths, client, o)
					.await
					.context("Failed to create server")?;
				Ok(result)
			}
		}?;

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
		self.dirs.ensure_full(|| {
			InstanceDirs::new(paths, &self.id, &self.profile_id, &self.kind.to_side())
		});
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

	fn get_fabric_quilt(
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

	/// Create a keypair file in the instance
	fn create_keypair(&mut self, user: &User, paths: &Paths) -> anyhow::Result<()> {
		if let Some(uuid) = user.get_uuid() {
			if let Some(keypair) = user.get_keypair() {
				self.ensure_dirs(paths)?;
				let keys_dir = self.dirs.get().game_dir.join("profilekeys");
				let hyphenated_uuid = hyphenate_uuid(uuid).context("Failed to hyphenate UUID")?;
				let path = keys_dir.join(format!("{hyphenated_uuid}.json"));
				mcvm_core::io::files::create_leading_dirs(&path)?;

				let mut file = File::create(path).context("Failed to create keypair file")?;
				serde_json::to_writer(&mut file, keypair)
					.context("Failed to write keypair to file")?;
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
	pub fn new(paths: &Paths, instance_id: &str, profile_id: &str, side: &Side) -> Self {
		let prof_dir = paths.project.data_dir().join("instances").join(profile_id);

		let inst_dir = prof_dir.join(instance_id);

		let game_dir = match side {
			Side::Client => inst_dir.join(".minecraft"),
			Side::Server => inst_dir.clone(),
		};

		Self { inst_dir, game_dir }
	}

	/// Make sure the directories exist
	pub fn ensure_exist(&self) -> anyhow::Result<()> {
		mcvm_core::io::files::create_leading_dirs(&self.inst_dir)?;
		mcvm_core::io::files::create_dir(&self.inst_dir)
			.context("Failed to create instance directory")?;
		mcvm_core::io::files::create_dir(&self.game_dir)
			.context("Failed to create game directory")?;
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
