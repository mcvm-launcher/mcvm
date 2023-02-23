use crate::util::json;
use crate::util::versions::MinecraftVersion;
use crate::net::download;
use crate::io::files;
use crate::io::java::{Java, JavaKind, JavaError};
use crate::{Paths, skip_none};
use crate::net::game_files;
use crate::util::print::ReplPrinter;
use super::user::Auth;
use super::client_args::{process_client_arg, process_string_arg};

use color_print::{cprintln, cformat};

use std::fs;
use std::path::PathBuf;
use std::process::Command;

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
	java: Option<Java>,
	classpath: Option<String>
}

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
	Java(#[from] JavaError)
}

#[derive(Debug, thiserror::Error)]
pub enum LaunchError {
	#[error("Failed to create instance:\n{}", .0)]
	Create(#[from] CreateError),
	#[error("Java is not installed")]
	Java,
	#[error("Command failed:\n{}", .0)]
	Command(std::io::Error),
	#[error("Failed to evaluate json file:\n{}", .0)]
	Json(#[from] json::JsonError),
}

impl Instance {
	pub fn new(kind: InstKind, id: &str, version: &MinecraftVersion) -> Self {
		Self {
			kind,
			id: id.to_owned(),
			version: version.to_owned(),
			version_json: None,
			java: None,
			classpath: None
		}
	}

	fn get_java(&mut self, kind: JavaKind, version: &str, paths: &Paths, verbose: bool, force: bool)
	-> Result<(), JavaError> {
		let mut java = Java::new(kind, version);
		java.install(paths, verbose, force)?;
		self.java = Some(java);
		Ok(())
	}

	pub fn get_dir(&self, paths: &Paths) -> PathBuf {
		match &self.kind {
			InstKind::Client => paths.project.data_dir().join("client").join(&self.id),
			InstKind::Server => paths.project.data_dir().join("server").join(&self.id),
		}
	}

	// Create the data for the instance
	pub async fn create(&mut self, paths: &Paths, verbose: bool, force: bool) -> Result<(), CreateError> {
		match &self.kind {
			InstKind::Client => {
				if force {
					cprintln!("<s>Rebuilding client <y!>{}</y!>", self.id);
				} else {
					cprintln!("<s>Updating client <y!>{}</y!>", self.id);
				}
				self.create_client(paths, verbose, force).await?;
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

	async fn create_client(&mut self, paths: &Paths, verbose: bool, force: bool) -> Result<(), CreateError> {
		let dir = self.get_dir(paths);
		files::create_leading_dirs(&dir)?;
		files::create_dir(&dir)?;
		let mc_dir = dir.join(".minecraft");
		files::create_dir(&mc_dir)?;
		let jar_path = dir.join("client.jar");

		let (version_json, mut dwn) = game_files::get_version_json(&self.version, paths, verbose)?;
		
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
		Ok(())
	}

	fn create_server(&mut self, paths: &Paths, verbose: bool, force: bool) -> Result<(), CreateError> {
		let dir = self.get_dir(paths);
		files::create_leading_dirs(&dir)?;
		files::create_dir(&dir)?;
		let server_dir = dir.join("server");
		files::create_dir(&server_dir)?;
		let jar_path = server_dir.join("server.jar");

		let (version_json, mut dwn) = game_files::get_version_json(&self.version, paths, verbose)?;
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
			printer.print(cformat!("<g>Server jar downloaded.").as_str());
			printer.finish();
		}

		fs::write(server_dir.join("eula.txt"), "eula = true\n")?;
		
		self.version_json = Some(version_json);
		Ok(())
	}
	
	// Launch the instance
	pub async fn launch(&mut self, paths: &Paths, auth: &Auth) -> Result<(), LaunchError> {
		cprintln!("Checking for updates...");
		match &self.kind {
			InstKind::Client => {
				self.create_client(paths, false, false).await?;
				cprintln!("<g>Launching!");
				self.launch_client(paths, auth)?;
			},
			InstKind::Server => {
				self.create_server(paths, false, false)?;
				cprintln!("<g>Launching!");
				self.launch_server(paths, auth)?;
			}
		}
		Ok(())
	}

	fn launch_client(&mut self, paths: &Paths, auth: &Auth) -> Result<(), LaunchError> {
		match &self.java {
			Some(java) => match &java.path {
				Some(java_path) => {
					let jre_path = java_path.join("bin/java");
					let client_dir = self.get_dir(paths).join(".minecraft");
					let mut command = Command::new(jre_path.to_str().expect("Failed to convert java path to a string"));
					command.current_dir(client_dir);

					if let Some(version_json) = &self.version_json {
						if let Some(classpath) = &self.classpath {
							let main_class = json::access_str(version_json, "mainClass")?;

							if let Ok(args) = json::access_object(version_json, "arguments") {
								for arg in json::access_array(args, "jvm")? {
									for sub_arg in process_client_arg(self, arg, paths, auth, classpath) {
										command.arg(sub_arg);
									}
								}

								command.arg(main_class);

								for arg in json::access_array(args, "game")? {
									for sub_arg in process_client_arg(self, arg, paths, auth, classpath) {
										command.arg(sub_arg);
									}
								}
							} else {
								// Behavior for versions prior to 1.12.2
								let args = json::access_str(version_json, "minecraftArguments")?;

								command.arg("-cp");
								command.arg(classpath);

								command.arg(main_class);

								for arg in args.split(' ') {
									command.arg(skip_none!(process_string_arg(self, arg, paths, auth, classpath)));
								}
							}

							let mut child = match command.spawn() {
								Ok(child) => child,
								Err(err) => return Err(LaunchError::Command(err))
							};
		
							child.wait().expect("Failed to wait for child process");
						}
					}
					Ok(())
				}
				None => Err(LaunchError::Java)
			}
			None => Err(LaunchError::Java)
		}
	}

	fn launch_server(&mut self, paths: &Paths, _auth: &Auth) -> Result<(), LaunchError> {
		match &self.java {
			Some(java) => match &java.path {
				Some(java_path) => {
					let jre_path = java_path.join("bin/java");
					let server_dir = self.get_dir(paths).join("server");
					let jar_path = server_dir.join("server.jar");

					let mut command = Command::new(jre_path.to_str().expect("Failed to convert java path to a string"));
					command.current_dir(server_dir);
					command.arg("-jar");
					command.arg(jar_path.to_str().expect("Failed to convert server.jar path to a string"));
					command.arg("nogui");
					let mut child = match command.spawn() {
						Ok(child) => child,
						Err(err) => return Err(LaunchError::Command(err))
					};

					child.wait().expect("Child failed");

					Ok(())
				}
				None => Err(LaunchError::Java)
			}
			None => Err(LaunchError::Java)
		}
	}
}
