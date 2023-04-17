use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use color_print::{cformat, cprintln};

use crate::data::profile::update::{UpdateManager, UpdateRequirement};
use crate::io::files::{self, paths::Paths};
use crate::io::java::classpath::Classpath;
use crate::io::java::JavaKind;
use crate::io::options::{self, client::write_options_txt, server::write_server_properties};
use crate::io::Later;
use crate::net::fabric_quilt;
use crate::net::{minecraft, paper};
use crate::util::{json, print::ReplPrinter};
use shared::modifications::{Modloader, PluginLoader};

use super::{InstKind, Instance};

impl Instance {
	/// Get the requirements for this instance
	pub fn get_requirements(&self) -> HashSet<UpdateRequirement> {
		let mut out = HashSet::new();
		out.insert(UpdateRequirement::VersionJson);

		let java_kind = match &self.launch.java {
			JavaKind::Adoptium(..) => JavaKind::Adoptium(Later::Empty),
			x => x.clone(),
		};
		out.insert(UpdateRequirement::Java(java_kind));
		out.insert(UpdateRequirement::GameJar(self.kind.to_side()));
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
	) -> anyhow::Result<HashSet<PathBuf>> {
		match &self.kind {
			InstKind::Client { .. } => {
				if manager.force {
					cprintln!("<s>Rebuilding client <y!>{}</y!>", self.id);
				} else {
					cprintln!("<s>Updating client <y!>{}</y!>", self.id);
				}
				let files = self
					.create_client(manager, paths)
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
	) -> anyhow::Result<HashSet<PathBuf>> {
		debug_assert!(matches!(self.kind, InstKind::Client { .. }));

		let out = HashSet::new();
		let version = manager.found_version.get();
		let dir = self.get_dir(paths);
		files::create_leading_dirs(&dir)?;
		files::create_dir(&dir)?;
		let mc_dir = self.get_subdir(paths);
		files::create_dir(&mc_dir)?;
		let jar_path = minecraft::game_jar::get_path(self.kind.to_side(), version, paths);

		let version_json = manager.version_json.get();

		let mut classpath = Classpath::new();
		let lib_classpath = minecraft::libraries::get_classpath(version_json, paths)
			.context("Failed to extract classpath from game library list")?;
		classpath.extend(lib_classpath);

		let java_vers = json::access_i64(
			json::access_object(version_json, "javaVersion")?,
			"majorVersion",
		)?;
		self.add_java(&java_vers.to_string(), manager);

		self.main_class = Some(json::access_str(version_json, "mainClass")?.to_owned());

		let fq_mode = match self.modloader {
			Modloader::Fabric => Some(fabric_quilt::Mode::Fabric),
			Modloader::Quilt => Some(fabric_quilt::Mode::Quilt),
			_ => None,
		};
		if let Some(mode) = fq_mode {
			classpath.extend(
				self.get_fabric_quilt(mode, paths, manager, version)
					.await
					.context("Failed to install Fabric/Quilt")?,
			);
		}

		classpath.add_path(&jar_path);

		let mut keys = HashMap::new();
		let version_list = manager.version_list.get();
		if let Some(global_options) = &manager.options {
			if let Some(global_options) = &global_options.client {
				let global_keys =
					options::client::create_keys(global_options, version, version_list)
						.context("Failed to create keys for global options")?;
				keys.extend(global_keys);
			}
		}
		if let InstKind::Client {
			options: Some(options),
		} = &self.kind
		{
			let override_keys = options::client::create_keys(options, version, version_list)
				.context("Failed to create keys for override options")?;
			keys.extend(override_keys);
		}
		if !keys.is_empty() {
			let options_path = mc_dir.join("options.txt");
			write_options_txt(keys, &options_path)
				.await
				.context("Failed to write options.txt")?;
		}

		self.classpath = Some(classpath);
		self.version_json = manager.version_json.clone();
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
		let version = manager.found_version.get();
		let dir = self.get_dir(paths);
		files::create_leading_dirs(&dir)?;
		files::create_dir(&dir)?;
		let server_dir = self.get_subdir(paths);
		files::create_dir(&server_dir)?;
		let jar_path = server_dir.join("server.jar");

		let version_json = manager.version_json.get();

		let java_vers = json::access_i64(
			json::access_object(version_json, "javaVersion")?,
			"majorVersion",
		)?;
		self.add_java(&java_vers.to_string(), manager);

		let classpath = match self.modloader {
			Modloader::Fabric => Some(
				self.get_fabric_quilt(fabric_quilt::Mode::Fabric, paths, manager, version)
					.await?,
			),
			Modloader::Quilt => Some(
				self.get_fabric_quilt(fabric_quilt::Mode::Quilt, paths, manager, version)
					.await?,
			),
			_ => None,
		};

		let eula_path = server_dir.join("eula.txt");
		let eula_task = tokio::spawn(async move {
			if !eula_path.exists() {
				tokio::fs::write(eula_path, "eula = true\n").await?;
			}

			Ok::<(), anyhow::Error>(())
		});

		self.jar_path.fill(match self.plugin_loader {
			PluginLoader::Vanilla => {
				let extern_jar_path =
					minecraft::game_jar::get_path(self.kind.to_side(), version, paths);
				if manager.should_update_file(&jar_path) {
					fs::hard_link(extern_jar_path, &jar_path)
						.context("Failed to hardlink server.jar")?;
					out.insert(jar_path.clone());
				}
				jar_path
			}
			PluginLoader::Paper => {
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

		eula_task.await?.context("Failed to create eula.txt")?;

		let mut keys = HashMap::new();
		let version_list = manager.version_list.get();
		if let Some(global_options) = &manager.options {
			if let Some(global_options) = &global_options.server {
				let global_keys =
					options::server::create_keys(global_options, version, version_list)
						.context("Failed to create keys for global options")?;
				keys.extend(global_keys);
			}
		}
		if let InstKind::Server {
			options: Some(options),
		} = &self.kind
		{
			let override_keys = options::server::create_keys(options, version, version_list)
				.context("Failed to create keys for override options")?;
			keys.extend(override_keys);
		}
		if !keys.is_empty() {
			let options_path = server_dir.join("server.properties");
			write_server_properties(keys, &options_path)
				.await
				.context("Failed to write server.properties")?;
		}

		self.version_json = manager.version_json.clone();
		self.classpath = classpath;

		Ok(out)
	}
}
