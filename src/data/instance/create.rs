use std::fs;

use color_print::{cprintln, cformat};

use crate::data::addon::PluginLoader;
use crate::io::files::{paths::Paths, self};
use crate::io::java::{JavaError, JavaKind};
use crate::net::{paper, game_files, download};
use crate::util::{json, print::ReplPrinter};

use super::{Instance, InstKind};

#[derive(Debug, thiserror::Error)]
pub enum CreateError {
	#[error("Failed to evaluate json file:\n{}", .0)]
	Parse(#[from] json::JsonError),
	#[error("Error when downloading file:\n{}", .0)]
	Download(#[from] download::DownloadError),
	#[error("Failed to process version json:\n{}", .0)]
	VersionJson(#[from] game_files::VersionJsonError),
	#[error("Failed to install libraries:\n{}", .0)]
	Libraries(#[from] game_files::LibrariesError),
	#[error("Failed to download assets:\n{}", .0)]
	Assets(#[from] game_files::AssetsError),
	#[error("Error when accessing files:\n{}", .0)]
	Io(#[from] std::io::Error),
	#[error("Failed to install java for this instance:\n{}", .0)]
	Java(#[from] JavaError),
	#[error("Failed to install a Paper server:\n{}", .0)]
	Paper(#[from] paper::PaperError)
}

impl Instance {
	// Create the data for the instance
	pub async fn create(&mut self, version_manifest: &json::JsonObject, paths: &Paths, verbose: bool, force: bool) -> Result<(), CreateError> {
		match &self.kind {
			InstKind::Client => {
				if force {
					cprintln!("<s>Rebuilding client <y!>{}</y!>", self.id);
				} else {
					cprintln!("<s>Updating client <y!>{}</y!>", self.id);
				}
				self.create_client(version_manifest, paths, verbose, force).await?;
			},
			InstKind::Server => {
				if force {
					cprintln!("<s>Rebuilding server <c!>{}</c!>", self.id);
				} else {
					cprintln!("<s>Updating server <c!>{}</c!>", self.id);
				}
				self.create_server(version_manifest, paths, verbose, force).await?
			}
		}
		Ok(())
	}

	pub async fn create_client(
		&mut self,
		version_manifest: &json::JsonObject,
		paths: &Paths,
		verbose: bool,
		force: bool
	) -> Result<(), CreateError> {
		let dir = self.get_dir(paths);
		files::create_leading_dirs(&dir)?;
		files::create_dir(&dir)?;
		let mc_dir = self.get_subdir(paths);
		files::create_dir(&mc_dir)?;
		let jar_path = dir.join("client.jar");

		let (version_json, mut dwn) = game_files::get_version_json(&self.version, version_manifest, paths)?;
		
		let mut classpath = game_files::get_libraries(&version_json, paths, &self.version, verbose, force)?;
		classpath.push_str(jar_path.to_str().expect("Failed to convert client.jar path to a string"));
		
		game_files::get_assets(&version_json, paths, &self.version, verbose, force).await?;

		let java_vers = json::access_i64(
			json::access_object(&version_json, "javaVersion")?,	"majorVersion"
		)?;
		self.get_java(JavaKind::Adoptium, &java_vers.to_string(), paths, verbose, force)?;

		if !jar_path.exists() || force {
			let mut printer = ReplPrinter::new(verbose);
			printer.indent(1);
			printer.print("Downloading client jar...");
			dwn.reset();
			dwn.add_file(&jar_path)?;
			let client_download = json::access_object(
				json::access_object(&version_json, "downloads")?,
				"client"
			)?;
			dwn.url(json::access_str(client_download, "url")?)?;
			dwn.perform()?;
			printer.print(cformat!("<g>Client jar downloaded.").as_str());
			printer.finish();
		}
		
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
		force: bool
	) -> Result<(), CreateError> {
		let dir = self.get_dir(paths);
		files::create_leading_dirs(&dir)?;
		files::create_dir(&dir)?;
		let server_dir = self.get_subdir(paths);
		files::create_dir(&server_dir)?;
		let jar_path = server_dir.join("server.jar");

		let (version_json, mut dwn) = game_files::get_version_json(&self.version, version_manifest, paths)?;

		let java_vers = json::access_i64(
			json::access_object(&version_json, "javaVersion")?,	"majorVersion"
		)?;
		self.get_java(JavaKind::Adoptium, &java_vers.to_string(), paths, verbose, force)?;

		if !jar_path.exists() || force {
			let mut printer = ReplPrinter::new(verbose);
			printer.indent(1);
			printer.print("Downloading server jar...");
			dwn.reset();
			dwn.add_file(&jar_path)?;
			let client_download = json::access_object(
				json::access_object(&version_json, "downloads")?,
				"server"
			)?;
			dwn.url(json::access_str(client_download, "url")?)?;
			dwn.perform()?;
			printer.print(&cformat!("<g>Server jar downloaded."));
			printer.finish();
		}

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
				if !paper_jar_path.exists() {
					printer.print("Downloading Paper server...");
					paper::download_server_jar(&self.version, build_num, &file_name, &server_dir).await?;
					printer.print(&cformat!("<g>Paper server downloaded."));
				}
				paper_jar_path
			}
		});
		
		self.version_json = Some(version_json);
		Ok(())
	}
}
