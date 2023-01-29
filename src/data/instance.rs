use crate::lib::json;
use crate::lib::versions::MinecraftVersion;
use crate::net::helper;
use crate::io::files::files;
use crate::io::java::{Java, JavaKind, JavaError};
use crate::Paths;
use crate::net::game_files;
use crate::lib::print::ReplPrinter;

use color_print::{cprintln, cformat};

use std::fs;

#[derive(Debug)]
pub enum InstKind {
	Client,
	Server
}

#[derive(Debug)]
pub struct Instance {
	pub kind: InstKind,
	pub id: String,
	pub version: MinecraftVersion,
	version_json: Option<Box<json::JsonObject>>,
	java: Option<Java>
}

#[derive(Debug, thiserror::Error)]
pub enum CreateError {
	#[error("Failed to evaluate json file:\n\t{}", .0)]
	ParseError(#[from] json::JsonError),
	#[error("Error when downloading file:\n\t{}", .0)]
	Download(#[from] helper::DownloadError),
	#[error("Failed to process version json:\n\t{}", .0)]
	VersionJson(#[from] game_files::VersionJsonError),
	#[error("Failed to install libraries:\n\t{}", .0)]
	Libraries(#[from] game_files::LibrariesError),
	#[error("Failed to download assets:\n\t{}", .0)]
	Assets(#[from] game_files::AssetsError),
	#[error("Error when accessing files:\n\t{}", .0)]
	Io(#[from] std::io::Error),
	#[error("Failed to install java for this instance:\n\t{}", .0)]
	Java(#[from] JavaError)
}

impl Instance {
	pub fn new(kind: InstKind, id: &str, version: &MinecraftVersion) -> Self {
		Self {
			kind,
			id: id.to_owned(),
			version: version.to_owned(),
			version_json: None,
			java: None
		}
	}

	// Create the data for the instance
	pub fn create(&mut self, paths: &Paths, verbose: bool, force: bool) -> Result<(), CreateError> {
		match &self.kind {
			InstKind::Client => {
				if force {
					cprintln!("<s>Rebuilding client <y!>{}</y!>", self.id);
				} else {
					cprintln!("<s>Updating client <y!>{}</y!>", self.id);
				}
				self.create_client(paths, verbose, force)?;
			},
			InstKind::Server => {
				if force {
					cprintln!("<s>Rebuilding server <c!>{}</c!>", self.id);
				} else {
					cprintln!("<s>Updating server <c!>{}</c!>", self.id);
				}
				self.create_server(paths, verbose, force)?
			}
		}
		Ok(())
	}

	fn create_client(&mut self, paths: &Paths, verbose: bool, force: bool) -> Result<(), CreateError> {
		let dir = paths.data.join("client").join(&self.id);
		files::create_leading_dirs(&dir).expect("Failed to create client directory");
		files::create_dir(&dir).expect("Failed to create client directory");
		let mc_dir = dir.join(".minecraft");
		files::create_dir(&mc_dir).expect("Failed to create minecraft directory");
		let jar_path = dir.join("client.jar");

		let (version_json, mut download) = game_files::get_version_json(&self.version, paths, verbose)?;
		
		let mut classpath = game_files::get_libraries(&version_json, paths, &self.version, verbose, force)?;
		classpath.push_str(jar_path.to_str().expect("Failed to convert client.jar path to a string"));

		game_files::get_assets(&version_json, paths, &self.version, verbose, force)?;

		let java = Java::new(JavaKind::Adoptium, "17");
		let java_path = java.install(paths, verbose, force)?;
		let jre_path = java_path.join("bin/java");

		if !jar_path.exists() || force {
			let mut printer = ReplPrinter::new();
			if verbose {
				printer.print("\tDownloading client jar...");
			}
			download.reset();
			download.add_file(&jar_path)?;
			let client_download = json::access_object(
				json::access_object(&version_json, "downloads")?,
				"client"
			)?;
			download.url(json::access_str(client_download, "url")?)?;
			download.perform()?;
			if verbose {
				printer.print(cformat!("\t<g>Client jar downloaded.").as_str());
				printer.finish();
			}
		}

		self.version_json = Some(version_json);
		self.java = Some(java);
		Ok(())
	}

	fn create_server(&mut self, paths: &Paths, verbose: bool, force: bool) -> Result<(), CreateError> {
		let dir = paths.data.join("server").join(&self.id);
		files::create_leading_dirs(&dir).expect("Failed to create server directory");
		files::create_dir(&dir).expect("Failed to create server directory");
		let server_dir = dir.join("server");
		files::create_dir(&server_dir).expect("Failed to create server directory");
		let jar_path = server_dir.join("server.jar");

		let (version_json, mut download) = game_files::get_version_json(&self.version, paths, verbose)?;

		if !jar_path.exists() || force {
			let mut printer = ReplPrinter::new();
			if verbose {
				printer.print("\tDownloading server jar...");
			}
			download.reset();
			download.add_file(&jar_path)?;
			let client_download = json::access_object(
				json::access_object(&version_json, "downloads")?,
				"server"
			)?;
			download.url(json::access_str(client_download, "url")?)?;
			download.perform()?;
			if verbose {
				printer.print(cformat!("\t<g>Server jar downloaded.").as_str());
				printer.finish();
			}
		}

		fs::write(server_dir.join("eula.txt"), "eula = true\n")?;
		
		self.version_json = Some(version_json);
		Ok(())
	}
}
