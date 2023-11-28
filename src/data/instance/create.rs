use std::collections::{HashMap, HashSet};
use std::fs::File;

use anyhow::Context;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel, OutputProcess};
use reqwest::Client;

use crate::data::profile::update::manager::{UpdateManager, UpdateMethodResult, UpdateRequirement};
use crate::data::user::uuid::hyphenate_uuid;
use crate::data::user::{AuthState, User, UserManager};
use crate::io::files::update_hardlink;
use crate::io::files::{self, paths::Paths};
use crate::io::java::classpath::Classpath;
use crate::io::java::install::JavaInstallationKind;
use crate::io::options::{self, client::write_options_txt, server::write_server_properties};
use crate::net::{fabric_quilt, game_files, paper};
use mcvm_shared::later::Later;
use mcvm_shared::modifications::{Modloader, ServerType};

use super::{InstKind, Instance};

/// The default main class for the server
pub const DEFAULT_SERVER_MAIN_CLASS: &str = "net.minecraft.server.Main";
/// The main class for a Paper server
pub const PAPER_SERVER_MAIN_CLASS: &str = "io.papermc.paperclip.Main";

impl Instance {
	/// Get the requirements for this instance
	pub fn get_requirements(&self) -> HashSet<UpdateRequirement> {
		let mut out = HashSet::new();
		// Even though it is the client meta it also contains the server download link
		// so we need it for both.
		out.insert(UpdateRequirement::ClientMeta);

		let java_kind = match &self.config.launch.java {
			JavaInstallationKind::Adoptium(..) => JavaInstallationKind::Adoptium(Later::Empty),
			JavaInstallationKind::Zulu(..) => JavaInstallationKind::Zulu(Later::Empty),
			x => x.clone(),
		};
		out.insert(UpdateRequirement::Java(java_kind));
		out.insert(UpdateRequirement::GameJar(self.kind.to_side()));
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
				out.insert(UpdateRequirement::ClientAssets);
				out.insert(UpdateRequirement::ClientLibraries);
				if self.config.launch.use_log4j_config {
					out.insert(UpdateRequirement::ClientLoggingConfig);
				}
			}
			InstKind::Server { .. } => {}
		}
		out
	}

	/// Create the data for the instance.
	pub async fn create(
		&mut self,
		manager: &UpdateManager,
		paths: &Paths,
		users: &UserManager,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<UpdateMethodResult> {
		match &self.kind {
			InstKind::Client { .. } => {
				o.display(
					MessageContents::Header(format!("Updating client {}", self.id)),
					MessageLevel::Important,
				);
				o.start_section();
				let result = self
					.create_client(manager, paths, users)
					.await
					.context("Failed to create client")?;
				o.end_section();
				Ok(result)
			}
			InstKind::Server { .. } => {
				o.display(
					MessageContents::Header(format!("Updating server {}", self.id)),
					MessageLevel::Important,
				);
				o.start_section();
				let result = self
					.create_server(manager, paths, client, o)
					.await
					.context("Failed to create server")?;
				o.end_section();
				Ok(result)
			}
		}
	}

	/// Create a client
	pub async fn create_client(
		&mut self,
		manager: &UpdateManager,
		paths: &Paths,
		users: &UserManager,
	) -> anyhow::Result<UpdateMethodResult> {
		debug_assert!(matches!(self.kind, InstKind::Client { .. }));

		let out = UpdateMethodResult::new();
		let version = &manager.version_info.get().version;
		self.ensure_dirs(paths)?;
		let jar_path =
			crate::io::minecraft::game_jar::get_path(self.kind.to_side(), version, paths);

		let client_meta = manager.client_meta.get();

		let mut classpath = Classpath::new();
		let lib_classpath = game_files::libraries::get_classpath(client_meta, paths)
			.context("Failed to extract classpath from game library list")?;
		classpath.extend(lib_classpath);

		let java_vers = client_meta.java_info.major_version;
		self.add_java(&java_vers.0.to_string(), manager);

		self.main_class = Some(client_meta.main_class.clone());

		if let Modloader::Fabric | Modloader::Quilt =
			self.config.modifications.get_modloader(self.kind.to_side())
		{
			classpath.extend(
				self.get_fabric_quilt(paths, manager)
					.await
					.context("Failed to install Fabric/Quilt")?,
			);
		}

		classpath.add_path(&jar_path);

		// Options
		let mut keys = HashMap::new();
		let version_info = &manager.version_info.get();
		if let Some(global_options) = &manager.options {
			if let Some(global_options) = &global_options.client {
				let global_keys = options::client::create_keys(global_options, version_info)
					.context("Failed to create keys for global options")?;
				keys.extend(global_keys);
			}
		}
		if let InstKind::Client {
			options: Some(options),
			..
		} = &self.kind
		{
			let override_keys = options::client::create_keys(options, version_info)
				.context("Failed to create keys for override options")?;
			keys.extend(override_keys);
		}
		if !keys.is_empty() {
			let options_path = self.dirs.get().game_dir.join("options.txt");
			let data_version = crate::io::minecraft::get_data_version(version_info, paths)
				.context("Failed to obtain data version")?;
			write_options_txt(keys, &options_path, &data_version)
				.await
				.context("Failed to write options.txt")?;
		}

		// Create keypair file
		if let AuthState::Authed(user) = &users.state {
			let user = users.users.get(user).expect("Authed user does not exist");
			self.create_keypair(user, paths)
				.context("Failed to create user keypair")?;
		}

		self.classpath = Some(classpath);
		self.jar_path.fill(jar_path);

		Ok(out)
	}

	/// Create a server
	pub async fn create_server(
		&mut self,
		manager: &UpdateManager,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<UpdateMethodResult> {
		debug_assert!(matches!(self.kind, InstKind::Server { .. }));

		let mut out = UpdateMethodResult::new();

		let version = &manager.version_info.get().version;
		self.ensure_dirs(paths)?;
		let jar_path = self.dirs.get().game_dir.join("server.jar");

		// Set the main class
		if let ServerType::Paper = self.config.modifications.server_type {
			self.main_class = Some(PAPER_SERVER_MAIN_CLASS.to_string());
		} else {
			self.main_class = Some(DEFAULT_SERVER_MAIN_CLASS.to_string());
		}

		let client_meta = manager.client_meta.get();

		let java_vers = client_meta.java_info.major_version;
		self.add_java(&java_vers.0.to_string(), manager);

		let mut classpath = if let Modloader::Fabric | Modloader::Quilt =
			self.config.modifications.get_modloader(self.kind.to_side())
		{
			self.get_fabric_quilt(paths, manager).await?
		} else {
			Classpath::new()
		};

		let eula_path = self.dirs.get().game_dir.join("eula.txt");
		let eula_task = tokio::spawn(async move {
			if !eula_path.exists() {
				tokio::fs::write(eula_path, "eula = true\n").await?;
			}

			Ok::<(), anyhow::Error>(())
		});

		self.jar_path
			.fill(match self.config.modifications.server_type {
				ServerType::Paper => {
					let process = OutputProcess::new(o);
					process.0.display(
						MessageContents::StartProcess("Checking for paper updates".into()),
						MessageLevel::Important,
					);

					let build_num = paper::get_newest_build(version, client)
						.await
						.context("Failed to get the newest Paper version")?;
					let file_name = paper::get_jar_file_name(version, build_num, client)
						.await
						.context("Failed to get the Paper file name")?;
					let paper_jar_path = self.dirs.get().game_dir.join(&file_name);
					if !manager.should_update_file(&paper_jar_path) {
						process.0.display(
							MessageContents::Success("Paper is up to date".into()),
							MessageLevel::Important,
						);
					} else {
						process.0.display(
							MessageContents::StartProcess("Downloading Paper server".into()),
							MessageLevel::Important,
						);
						paper::download_server_jar(
							version,
							build_num,
							&file_name,
							&self.dirs.get().game_dir,
							client,
						)
						.await
						.context("Failed to download Paper server JAR")?;
						process.0.display(
							MessageContents::Success("Paper server downloaded".into()),
							MessageLevel::Important,
						);
					}

					out.files_updated.insert(paper_jar_path.clone());
					paper_jar_path
				}
				_ => {
					let extern_jar_path = crate::io::minecraft::game_jar::get_path(
						self.kind.to_side(),
						version,
						paths,
					);
					if manager.should_update_file(&jar_path) {
						update_hardlink(&extern_jar_path, &jar_path)
							.context("Failed to hardlink server.jar")?;
						out.files_updated.insert(jar_path.clone());
					}
					jar_path
				}
			});

		classpath.add_path(self.jar_path.get());

		eula_task.await?.context("Failed to create eula.txt")?;

		let mut keys = HashMap::new();
		let version_info = manager.version_info.get();
		if let Some(global_options) = &manager.options {
			if let Some(global_options) = &global_options.server {
				let global_keys = options::server::create_keys(global_options, version_info)
					.context("Failed to create keys for global options")?;
				keys.extend(global_keys);
			}
		}
		if let InstKind::Server {
			options: Some(options),
		} = &self.kind
		{
			let override_keys = options::server::create_keys(options, version_info)
				.context("Failed to create keys for override options")?;
			keys.extend(override_keys);
		}
		if !keys.is_empty() {
			let options_path = self.dirs.get().game_dir.join("server.properties");
			write_server_properties(keys, &options_path)
				.await
				.context("Failed to write server.properties")?;
		}

		self.classpath = Some(classpath);

		Ok(out)
	}

	/// Create a keypair file in the instance
	pub fn create_keypair(&mut self, user: &User, paths: &Paths) -> anyhow::Result<()> {
		if let Some(uuid) = &user.uuid {
			if let Some(keypair) = &user.keypair {
				self.ensure_dirs(paths)?;
				let keys_dir = self.dirs.get().game_dir.join("profilekeys");
				let hyphenated_uuid = hyphenate_uuid(uuid).context("Failed to hyphenate UUID")?;
				let path = keys_dir.join(format!("{hyphenated_uuid}.json"));
				files::create_leading_dirs(&path)?;

				let mut file = File::create(path).context("Failed to create keypair file")?;
				serde_json::to_writer(&mut file, keypair)
					.context("Failed to write keypair to file")?;
			}
		}

		Ok(())
	}
}
