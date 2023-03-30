use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use color_print::{cformat, cprintln};

use crate::data::addon::{Modloader, PluginLoader};
use crate::data::profile::update::{UpdateRequirement, UpdateManager};
use crate::io::files::{self, paths::Paths};
use crate::io::java::JavaKind;
use crate::io::java::classpath::Classpath;
use crate::io::options::write_options_txt;
use crate::net::fabric_quilt;
use crate::net::{minecraft, paper};
use crate::util::{json, print::ReplPrinter};

use super::{InstKind, Instance};

impl Instance {
	/// Get the requirements for this instance
	pub fn get_requirements(&self) -> HashSet<UpdateRequirement> {
		let mut out = HashSet::new();
		out.insert(UpdateRequirement::VersionJson);

		let java_kind = match &self.launch.java {
			JavaKind::Adoptium(..) => JavaKind::Adoptium(None),
			x => x.clone(),
		};
		out.insert(UpdateRequirement::Java(java_kind));
		out.insert(UpdateRequirement::GameJar(self.kind.clone()));
		match &self.kind {
			InstKind::Client => {
				out.insert(UpdateRequirement::GameAssets);
				out.insert(UpdateRequirement::GameLibraries);
				out.insert(UpdateRequirement::Options);
			}
			InstKind::Server => {}
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
			InstKind::Client => {
				if manager.force {
					cprintln!("<s>Rebuilding client <y!>{}</y!>", self.id);
				} else {
					cprintln!("<s>Updating client <y!>{}</y!>", self.id);
				}
				let files = self.create_client(manager, paths).await
					.context("Failed to create client")?;
				Ok(files)
			}
			InstKind::Server => {
				if manager.force {
					cprintln!("<s>Rebuilding server <c!>{}</c!>", self.id);
				} else {
					cprintln!("<s>Updating server <c!>{}</c!>", self.id);
				}
				let files = self.create_server(manager, paths).await
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
		debug_assert!(self.kind == InstKind::Client);
		let out = HashSet::new();
		
		let dir = self.get_dir(paths);
		files::create_leading_dirs(&dir)?;
		files::create_dir(&dir)?;
		let mc_dir = self.get_subdir(paths);
		files::create_dir(&mc_dir)?;
		let jar_path = minecraft::game_jar_path(&self.kind, &self.version, paths);

		let version_json = manager.version_json.clone().expect("Version json missing");

		let mut classpath = Classpath::new();
		let lib_classpath = minecraft::get_lib_classpath(&version_json, paths)
			.context("Failed to extract classpath from game library list")?;
		classpath.extend(lib_classpath);

		let java_vers = json::access_i64(
			json::access_object(&version_json, "javaVersion")?,
			"majorVersion",
		)?;
		self.add_java(&java_vers.to_string(), manager);

		self.main_class = Some(json::access_str(&version_json, "mainClass")?.to_owned());
		
		let fq_mode = match self.modloader {
			Modloader::Fabric => Some(fabric_quilt::Mode::Fabric),
			Modloader::Quilt => Some(fabric_quilt::Mode::Quilt),
			_ => None,
		};
		if let Some(mode) = fq_mode {
			classpath.extend(self.get_fabric_quilt(mode, paths, manager).await
				.context("Failed to install Fabric/Quilt")?);
		}

		classpath.add_path(&jar_path);

		if let Some(options) = &manager.options {
			let options_path = mc_dir.join("options.txt");
			write_options_txt(
				options,
				&options_path,
				&self.version,
				manager.version_list.as_ref().expect("Version list missing")
			).context("Failed to write options.txt")?;
		}

		self.classpath = Some(classpath);
		self.version_json = Some(version_json);
		self.jar_path = Some(jar_path);
		Ok(out)
	}

	/// Create a server
	pub async fn create_server(
		&mut self,
		manager: &UpdateManager,
		paths: &Paths,
	) -> anyhow::Result<HashSet<PathBuf>> {
		debug_assert!(self.kind == InstKind::Server);
		let mut out = HashSet::new();
		
		let dir = self.get_dir(paths);
		files::create_leading_dirs(&dir)?;
		files::create_dir(&dir)?;
		let server_dir = self.get_subdir(paths);
		files::create_dir(&server_dir)?;
		let jar_path = server_dir.join("server.jar");
		
		let version_json = manager.version_json.clone().expect("Version json missing");
		
		let java_vers = json::access_i64(
			json::access_object(&version_json, "javaVersion")?,
			"majorVersion",
		)?;
		self.add_java(&java_vers.to_string(), manager);
		
		let classpath = match self.modloader {
			Modloader::Fabric => Some(self.get_fabric_quilt(fabric_quilt::Mode::Fabric, paths, manager).await?),
			Modloader::Quilt => Some(self.get_fabric_quilt(fabric_quilt::Mode::Quilt, paths, manager).await?),
			_ => None,
		};
		
		let eula_path = server_dir.join("eula.txt");
		let eula_task = tokio::spawn(async move {
			if !eula_path.exists() {
				tokio::fs::write(eula_path, "eula = true\n").await?;
			}

			Ok::<(), anyhow::Error>(())
		});
		
		self.jar_path = Some(match self.plugin_loader {
			PluginLoader::Vanilla => {
				let extern_jar_path = minecraft::game_jar_path(&self.kind, &self.version, paths);
				if manager.should_update_file(&jar_path) {
					fs::hard_link(extern_jar_path, &jar_path).context("Failed to hardlink server.jar")?;
					out.insert(jar_path.clone());
				} 
				jar_path
			}
			PluginLoader::Paper => {
				let mut printer = ReplPrinter::from_options(manager.print.clone());
				printer.indent(1);
				printer.print("Checking for paper updates...");
				let (build_num, ..) = paper::get_newest_build(&self.version).await
					.context("Failed to get the newest Paper version")?;
				let file_name = paper::get_jar_file_name(&self.version, build_num).await
					.context("Failed to get the Paper file name")?;
				let paper_jar_path = server_dir.join(&file_name);
				if !manager.should_update_file(&paper_jar_path) {
					printer.print(&cformat!("<g>Paper is up to date."));
				} else {
					printer.print("Downloading Paper server...");
					paper::download_server_jar(&self.version, build_num, &file_name, &server_dir).await
						.context("Failed to download Paper server JAR")?;
					printer.print(&cformat!("<g>Paper server downloaded."));
				}
				out.insert(paper_jar_path.clone());
				paper_jar_path
			}
		});

		eula_task.await?.context("Failed to create eula.txt")?;
		
		self.version_json = Some(version_json);
		self.classpath = classpath;
		Ok(out)
	}
}
