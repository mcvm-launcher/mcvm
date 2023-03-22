use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use color_print::{cformat, cprintln};
use reqwest::Client;

use crate::data::addon::{Modloader, PluginLoader};
use crate::data::profile::update::{UpdateRequirement, UpdateManager};
use crate::io::files::{self, paths::Paths};
use crate::io::java::{JavaError, JavaKind};
use crate::io::java::classpath::Classpath;
use crate::net::fabric_quilt::{FabricQuiltError, self};
use crate::net::minecraft::VersionManifestError;
use crate::net::{minecraft, paper};
use crate::util::{json, print::ReplPrinter};

use super::{InstKind, Instance};

#[derive(Debug, thiserror::Error)]
pub enum CreateError {
	#[error("Failed to evaluate json file:\n{}", .0)]
	Parse(#[from] json::JsonError),
	#[error("Error when downloading file:\n{}", .0)]
	Download(#[from] reqwest::Error),
	#[error("Failed to process version json:\n{}", .0)]
	VersionJson(#[from] minecraft::VersionJsonError),
	#[error("Failed to install libraries:\n{}", .0)]
	Libraries(#[from] minecraft::LibrariesError),
	#[error("Failed to download assets:\n{}", .0)]
	Assets(#[from] minecraft::AssetsError),
	#[error("Error when accessing files:\n{}", .0)]
	Io(#[from] std::io::Error),
	#[error("Failed to install java for this instance:\n{}", .0)]
	Java(#[from] JavaError),
	#[error("Failed to install a Paper server:\n{}", .0)]
	Paper(#[from] paper::PaperError),
	#[error("Failed to install Fabric or Quilt:\n{}", .0)]
	Fabric(#[from] FabricQuiltError),
	#[error("Failed to download version manifest:\n{}", .0)]
	Manifest(#[from] VersionManifestError)
}

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
		match &self.kind {
			InstKind::Client => {
				out.insert(UpdateRequirement::GameAssets);
				out.insert(UpdateRequirement::GameLibraries);
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
	) -> Result<HashSet<PathBuf>, CreateError> {
		match &self.kind {
			InstKind::Client => {
				if manager.force {
					cprintln!("<s>Rebuilding client <y!>{}</y!>", self.id);
				} else {
					cprintln!("<s>Updating client <y!>{}</y!>", self.id);
				}
				let files = self.create_client(manager, paths).await?;
				Ok(files)
			}
			InstKind::Server => {
				if manager.force {
					cprintln!("<s>Rebuilding server <c!>{}</c!>", self.id);
				} else {
					cprintln!("<s>Updating server <c!>{}</c!>", self.id);
				}
				let files = self.create_server(manager, paths).await?;
				Ok(files)
			}
		}
	}

	pub async fn create_client(
		&mut self,
		manager: &UpdateManager,
		paths: &Paths,
	) -> Result<HashSet<PathBuf>, CreateError> {
		let mut out = HashSet::new();
		
		let dir = self.get_dir(paths);
		files::create_leading_dirs(&dir)?;
		files::create_dir(&dir)?;
		let mc_dir = self.get_subdir(paths);
		files::create_dir(&mc_dir)?;
		let jar_path = dir.join("client.jar");

		let version_json = manager.version_json.clone().expect("Version json missing");

		let client = Client::new();

		let mut classpath = Classpath::new();
		let (lib_classpath, files) = minecraft::get_libraries(&version_json, paths, &self.version, manager).await?;
		classpath.extend(lib_classpath);
		out.extend(files);

		let java_vers = json::access_i64(
			json::access_object(&version_json, "javaVersion")?,
			"majorVersion",
		)?;
		self.add_java(&java_vers.to_string());

		if manager.should_update_file(&jar_path) {
			let mut printer = ReplPrinter::from_options(manager.print.clone());
			printer.indent(1);
			printer.print("Downloading client jar...");

			let client_download =
				json::access_object(json::access_object(&version_json, "downloads")?, "client")?;
			let url = json::access_str(client_download, "url")?;
			fs::write(&jar_path, client.get(url).send().await?.bytes().await?)?;
			printer.print(cformat!("<g>Client jar downloaded.").as_str());
			printer.finish();
			out.insert(jar_path.clone());
		}

		self.main_class = Some(json::access_str(&version_json, "mainClass")?.to_owned());
		
		let fq_mode = match self.modloader {
			Modloader::Fabric => Some(fabric_quilt::Mode::Fabric),
			Modloader::Quilt => Some(fabric_quilt::Mode::Quilt),
			_ => None,
		};
		if let Some(mode) = fq_mode {
			classpath.extend(self.get_fabric_quilt(mode, paths, manager).await?);
		}

		classpath.add_path(&jar_path);

		self.classpath = Some(classpath);
		self.version_json = Some(version_json);
		self.jar_path = Some(jar_path);
		Ok(out)
	}

	pub async fn create_server(
		&mut self,
		manager: &UpdateManager,
		paths: &Paths,
	) -> Result<HashSet<PathBuf>, CreateError> {
		let mut out = HashSet::new();
		
		let dir = self.get_dir(paths);
		files::create_leading_dirs(&dir)?;
		files::create_dir(&dir)?;
		let server_dir = self.get_subdir(paths);
		files::create_dir(&server_dir)?;
		let jar_path = server_dir.join("server.jar");

		let version_json = manager.version_json.clone().expect("Version json missing");

		let client = Client::new();

		let java_vers = json::access_i64(
			json::access_object(&version_json, "javaVersion")?,
			"majorVersion",
		)?;
		self.add_java(&java_vers.to_string());

		if manager.should_update_file(&jar_path) {
			let mut printer = ReplPrinter::from_options(manager.print.clone());
			printer.indent(1);
			printer.print("Downloading server jar...");
			let server_download =
				json::access_object(json::access_object(&version_json, "downloads")?, "server")?;
			let url = json::access_str(server_download, "url")?;
			fs::write(&jar_path, client.get(url).send().await?.bytes().await?)?;
			printer.print(&cformat!("<g>Server jar downloaded."));

			out.insert(jar_path.clone());
		}

		let classpath = match self.modloader {
			Modloader::Fabric => Some(self.get_fabric_quilt(fabric_quilt::Mode::Fabric, paths, manager).await?),
			Modloader::Quilt => Some(self.get_fabric_quilt(fabric_quilt::Mode::Quilt, paths, manager).await?),
			_ => None,
		};

		fs::write(server_dir.join("eula.txt"), "eula = true\n")?;

		self.jar_path = Some(match self.plugin_loader {
			PluginLoader::Vanilla => jar_path,
			PluginLoader::Paper => {
				let mut printer = ReplPrinter::from_options(manager.print.clone());
				printer.indent(1);
				printer.print("Checking for paper updates...");
				let (build_num, ..) = paper::get_newest_build(&self.version).await?;
				let file_name = paper::get_jar_file_name(&self.version, build_num).await?;
				let paper_jar_path = server_dir.join(&file_name);
				if paper_jar_path.exists() {
					printer.print(&cformat!("<g>Paper is up to date."));
				} else {
					printer.print("Downloading Paper server...");
					paper::download_server_jar(&self.version, build_num, &file_name, &server_dir)
						.await?;
					printer.print(&cformat!("<g>Paper server downloaded."));
				}
				paper_jar_path
			}
		});

		self.version_json = Some(version_json);
		self.classpath = classpath;
		Ok(out)
	}
}
