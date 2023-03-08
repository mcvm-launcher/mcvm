use std::fs;

use color_print::{cformat, cprintln};

use crate::data::addon::{Modloader, PluginLoader};
use crate::io::files::{self, paths::Paths};
use crate::io::java::JavaError;
use crate::io::java::classpath::Classpath;
use crate::net::fabric_quilt::FabricError;
use crate::net::{download, mojang, paper};
use crate::util::{json, print::ReplPrinter};

use super::{InstKind, Instance};

#[derive(Debug, thiserror::Error)]
pub enum CreateError {
	#[error("Failed to evaluate json file:\n{}", .0)]
	Parse(#[from] json::JsonError),
	#[error("Error when downloading file:\n{}", .0)]
	Download(#[from] download::DownloadError),
	#[error("Failed to process version json:\n{}", .0)]
	VersionJson(#[from] mojang::VersionJsonError),
	#[error("Failed to install libraries:\n{}", .0)]
	Libraries(#[from] mojang::LibrariesError),
	#[error("Failed to download assets:\n{}", .0)]
	Assets(#[from] mojang::AssetsError),
	#[error("Error when accessing files:\n{}", .0)]
	Io(#[from] std::io::Error),
	#[error("Failed to install java for this instance:\n{}", .0)]
	Java(#[from] JavaError),
	#[error("Failed to install a Paper server:\n{}", .0)]
	Paper(#[from] paper::PaperError),
	#[error("Failed to install Fabric or Quilt:\n{}", .0)]
	Fabric(#[from] FabricError),
}

impl Instance {
	// Create the data for the instance
	pub async fn create(
		&mut self,
		version_manifest: &json::JsonObject,
		paths: &Paths,
		verbose: bool,
		force: bool,
	) -> Result<(), CreateError> {
		match &self.kind {
			InstKind::Client => {
				if force {
					cprintln!("<s>Rebuilding client <y!>{}</y!>", self.id);
				} else {
					cprintln!("<s>Updating client <y!>{}</y!>", self.id);
				}
				self.create_client(version_manifest, paths, verbose, force)
					.await?;
			}
			InstKind::Server => {
				if force {
					cprintln!("<s>Rebuilding server <c!>{}</c!>", self.id);
				} else {
					cprintln!("<s>Updating server <c!>{}</c!>", self.id);
				}
				self.create_server(version_manifest, paths, verbose, force)
					.await?
			}
		}
		Ok(())
	}

	pub async fn create_client(
		&mut self,
		version_manifest: &json::JsonObject,
		paths: &Paths,
		verbose: bool,
		force: bool,
	) -> Result<(), CreateError> {
		let dir = self.get_dir(paths);
		files::create_leading_dirs(&dir)?;
		files::create_dir(&dir)?;
		let mc_dir = self.get_subdir(paths);
		files::create_dir(&mc_dir)?;
		let jar_path = dir.join("client.jar");

		let (version_json, mut dwn) =
			mojang::get_version_json(&self.version, version_manifest, paths)?;

		let mut classpath = Classpath::new();
		classpath.extend(mojang::get_libraries(&version_json, paths, &self.version, verbose, force)?);

		mojang::get_assets(&version_json, paths, &self.version, verbose, force).await?;

		let java_vers = json::access_i64(
			json::access_object(&version_json, "javaVersion")?,
			"majorVersion",
		)?;
		self.get_java(&java_vers.to_string(), paths, verbose, force)?;

		if !jar_path.exists() || force {
			let mut printer = ReplPrinter::new(verbose);
			printer.indent(1);
			printer.print("Downloading client jar...");
			dwn.reset();
			dwn.add_file(&jar_path)?;
			let client_download =
				json::access_object(json::access_object(&version_json, "downloads")?, "client")?;
			dwn.url(json::access_str(client_download, "url")?)?;
			dwn.perform()?;
			printer.print(cformat!("<g>Client jar downloaded.").as_str());
			printer.finish();
		}

		if let Modloader::Quilt = self.modloader {
			classpath.extend(self.get_quilt(paths, verbose, force).await?);
		}

		classpath.add_path(&jar_path);

		self.main_class = Some(json::access_str(&version_json, "mainClass")?.to_owned());
		self.classpath = Some(classpath);
		self.version_json = Some(version_json);
		self.jar_path = Some(jar_path);
		Ok(())
	}

	pub async fn create_server(
		&mut self,
		version_manifest: &json::JsonObject,
		paths: &Paths,
		verbose: bool,
		force: bool,
	) -> Result<(), CreateError> {
		let dir = self.get_dir(paths);
		files::create_leading_dirs(&dir)?;
		files::create_dir(&dir)?;
		let server_dir = self.get_subdir(paths);
		files::create_dir(&server_dir)?;
		let jar_path = server_dir.join("server.jar");

		let (version_json, mut dwn) =
			mojang::get_version_json(&self.version, version_manifest, paths)?;

		let java_vers = json::access_i64(
			json::access_object(&version_json, "javaVersion")?,
			"majorVersion",
		)?;
		self.get_java(&java_vers.to_string(), paths, verbose, force)?;

		if !jar_path.exists() || force {
			let mut printer = ReplPrinter::new(verbose);
			printer.indent(1);
			printer.print("Downloading server jar...");
			dwn.reset();
			dwn.add_file(&jar_path)?;
			let client_download =
				json::access_object(json::access_object(&version_json, "downloads")?, "server")?;
			dwn.url(json::access_str(client_download, "url")?)?;
			dwn.perform()?;
			printer.print(&cformat!("<g>Server jar downloaded."));
		}

		let classpath = if let Modloader::Quilt = self.modloader {
			Some(self.get_quilt(paths, verbose, force).await?)
		} else {
			None
		};

		fs::write(server_dir.join("eula.txt"), "eula = true\n")?;

		self.jar_path = Some(match self.plugin_loader {
			PluginLoader::Vanilla => jar_path,
			PluginLoader::Paper => {
				let mut printer = ReplPrinter::new(verbose);
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
		Ok(())
	}
}
