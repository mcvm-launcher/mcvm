use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::PathBuf;

use anyhow::Context;
use color_print::{cformat, cprintln};

use crate::data::profile::update::{UpdateManager, UpdateRequirement};
use crate::data::user::uuid::hyphenate_uuid;
use crate::data::user::{AuthState, User, UserManager};
use crate::io::files::update_hardlink;
use crate::io::files::{self, paths::Paths};
use crate::io::java::classpath::Classpath;
use crate::io::java::JavaKind;
use crate::io::options::{self, client::write_options_txt, server::write_server_properties};
use crate::net::{fabric_quilt, minecraft, paper};
use crate::util::{json, print::ReplPrinter};
use mcvm_shared::later::Later;
use mcvm_shared::modifications::{Modloader, ServerType};

use super::{InstKind, Instance};

pub const DEFAULT_SERVER_MAIN_CLASS: &str = "net.minecraft.server.Main";
pub const PAPER_SERVER_MAIN_CLASS: &str = "io.papermc.paperclip.Main";

impl Instance {
	/// Get the requirements for this instance
	pub fn get_requirements(&self) -> HashSet<UpdateRequirement> {
		let mut out = HashSet::new();
		out.insert(UpdateRequirement::ClientJson);

		let java_kind = match &self.launch.java {
			JavaKind::Adoptium(..) => JavaKind::Adoptium(Later::Empty),
			x => x.clone(),
		};
		out.insert(UpdateRequirement::Java(java_kind));
		out.insert(UpdateRequirement::GameJar(self.kind.to_side()));
		match self.modifications.get_modloader(self.kind.to_side()) {
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
				out.insert(UpdateRequirement::GameAssets);
				out.insert(UpdateRequirement::GameLibraries);
			}
			InstKind::Server { .. } => {}
		}
		out
	}

	/// Create the data for the instance.
	/// Returns a list of files to be added to the update manager.
	pub async fn create(
		&mut self,
		manager: &UpdateManager,
		paths: &Paths,
		users: &UserManager,
	) -> anyhow::Result<HashSet<PathBuf>> {
		match &self.kind {
			InstKind::Client { .. } => {
				if manager.force {
					cprintln!("<s>Rebuilding client <y!>{}</y!>", self.id);
				} else {
					cprintln!("<s>Updating client <y!>{}</y!>", self.id);
				}
				let files = self
					.create_client(manager, paths, users)
					.await
					.context("Failed to create client")?;
				Ok(files)
			}
			InstKind::Server { .. } => {
				if manager.force {
					cprintln!("<s>Rebuilding server <c!>{}</c!>", self.id);
				} else {
					cprintln!("<s>Updating server <c!>{}</c!>", self.id);
				}
				let files = self
					.create_server(manager, paths)
					.await
					.context("Failed to create server")?;
				Ok(files)
			}
		}
	}

	/// Create a client
	pub async fn create_client(
		&mut self,
		manager: &UpdateManager,
		paths: &Paths,
		users: &UserManager,
	) -> anyhow::Result<HashSet<PathBuf>> {
		debug_assert!(matches!(self.kind, InstKind::Client { .. }));

		let out = HashSet::new();
		let version = &manager.version_info.get().version;
		let dir = self.get_dir(paths);
		files::create_leading_dirs(&dir)?;
		files::create_dir(&dir)?;
		let mc_dir = self.get_subdir(paths);
		files::create_dir(&mc_dir)?;
		let jar_path =
			crate::io::minecraft::game_jar::get_path(self.kind.to_side(), version, paths);

		let client_json = manager.client_json.get();

		let mut classpath = Classpath::new();
		let lib_classpath = minecraft::libraries::get_classpath(client_json, paths)
			.context("Failed to extract classpath from game library list")?;
		classpath.extend(lib_classpath);

		let java_vers = json::access_i64(
			json::access_object(client_json, "javaVersion")?,
			"majorVersion",
		)?;
		self.add_java(&java_vers.to_string(), manager);

		self.main_class = Some(json::access_str(client_json, "mainClass")?.to_owned());

		if let Modloader::Fabric | Modloader::Quilt =
			self.modifications.get_modloader(self.kind.to_side())
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
			let options_path = mc_dir.join("options.txt");
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
		self.client_json = manager.client_json.clone();
		self.jar_path.fill(jar_path);

		Ok(out)
	}

	/// Create a server
	pub async fn create_server(
		&mut self,
		manager: &UpdateManager,
		paths: &Paths,
	) -> anyhow::Result<HashSet<PathBuf>> {
		debug_assert!(matches!(self.kind, InstKind::Server { .. }));

		let mut out = HashSet::new();

		let version = &manager.version_info.get().version;
		let dir = self.get_dir(paths);
		files::create_leading_dirs(&dir)?;
		files::create_dir(&dir)?;
		let server_dir = self.get_subdir(paths);
		files::create_dir(&server_dir)?;
		let jar_path = server_dir.join("server.jar");

		// Set the main class
		if let ServerType::Paper = self.modifications.server_type {
			self.main_class = Some(PAPER_SERVER_MAIN_CLASS.to_string());
		} else {
			self.main_class = Some(DEFAULT_SERVER_MAIN_CLASS.to_string());
		}

		let client_json = manager.client_json.get();

		let java_vers = json::access_i64(
			json::access_object(client_json, "javaVersion")?,
			"majorVersion",
		)?;
		self.add_java(&java_vers.to_string(), manager);

		let mut classpath = if let Modloader::Fabric | Modloader::Quilt =
			self.modifications.get_modloader(self.kind.to_side())
		{
			self.get_fabric_quilt(paths, manager).await?
		} else {
			Classpath::new()
		};

		let eula_path = server_dir.join("eula.txt");
		let eula_task = tokio::spawn(async move {
			if !eula_path.exists() {
				tokio::fs::write(eula_path, "eula = true\n").await?;
			}

			Ok::<(), anyhow::Error>(())
		});

		self.jar_path.fill(match self.modifications.server_type {
			ServerType::None
			| ServerType::Vanilla
			| ServerType::Forge
			| ServerType::Fabric
			| ServerType::Quilt => {
				let extern_jar_path =
					crate::io::minecraft::game_jar::get_path(self.kind.to_side(), version, paths);
				if manager.should_update_file(&jar_path) {
					update_hardlink(&extern_jar_path, &jar_path)
						.context("Failed to hardlink server.jar")?;
					out.insert(jar_path.clone());
				}
				jar_path
			}
			ServerType::Paper => {
				let mut printer = ReplPrinter::from_options(manager.print.clone());
				printer.indent(1);
				printer.print("Checking for paper updates...");
				let (build_num, ..) = paper::get_newest_build(version)
					.await
					.context("Failed to get the newest Paper version")?;
				let file_name = paper::get_jar_file_name(version, build_num)
					.await
					.context("Failed to get the Paper file name")?;
				let paper_jar_path = server_dir.join(&file_name);
				if !manager.should_update_file(&paper_jar_path) {
					printer.print(&cformat!("<g>Paper is up to date."));
				} else {
					printer.print("Downloading Paper server...");
					paper::download_server_jar(version, build_num, &file_name, &server_dir)
						.await
						.context("Failed to download Paper server JAR")?;
					printer.print(&cformat!("<g>Paper server downloaded."));
				}
				out.insert(paper_jar_path.clone());
				paper_jar_path
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
			let options_path = server_dir.join("server.properties");
			write_server_properties(keys, &options_path)
				.await
				.context("Failed to write server.properties")?;
		}

		self.client_json = manager.client_json.clone();
		self.classpath = Some(classpath);

		Ok(out)
	}

	/// Create a keypair file in the instance
	pub fn create_keypair(&self, user: &User, paths: &Paths) -> anyhow::Result<()> {
		if let Some(uuid) = &user.uuid {
			if let Some(keypair) = &user.keypair {
				let mc_dir = self.get_subdir(paths);
				let keys_dir = mc_dir.join("profilekeys");
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
