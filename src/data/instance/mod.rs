pub mod create;
pub mod launch;

use crate::util::json;
use crate::io::files;
use crate::io::java::{Java, JavaKind, JavaError};
use crate::Paths;
use self::create::CreateError;

use super::addon::{Addon, AddonKind, Modloader, PluginLoader};
use super::config::instance::LaunchOptions;

use std::fs;
use std::path::{PathBuf, Path};

#[derive(Debug, Clone, PartialEq)]
pub enum InstKind {
	Client,
	Server
}

impl InstKind {
	pub fn from_str(string: &str) -> Option<Self> {
		match string {
			"client" => Some(Self::Client),
			"server" => Some(Self::Server),
			_ => None
		}
	}
}

#[derive(Debug)]
pub struct Instance {
	pub kind: InstKind,
	pub id: String,
	pub version: String,
	modloader: Modloader,
	plugin_loader: PluginLoader,
	launch: LaunchOptions,
	version_json: Option<Box<json::JsonObject>>,
	java: Option<Java>,
	classpath: Option<String>,
	jar_path: Option<PathBuf>
}

impl Instance {
	pub fn new(
		kind: InstKind,
		id: &str,
		version: &str,
		modloader: Modloader,
		plugin_loader: PluginLoader,
		launch: LaunchOptions
	) -> Self {
		Self {
			kind,
			id: id.to_owned(),
			version: version.to_owned(),
			modloader,
			plugin_loader,
			launch,
			version_json: None,
			java: None,
			classpath: None,
			jar_path: None
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
			InstKind::Server => paths.project.data_dir().join("server").join(&self.id)
		}
	}

	pub fn get_subdir(&self, paths: &Paths) -> PathBuf {
		self.get_dir(paths).join(match self.kind {
			InstKind::Client => ".minecraft",
			InstKind::Server => "server"
		})
	}
	
	pub fn get_linked_addon_path(&self, addon: &Addon, paths: &Paths) -> Option<PathBuf> {
		let inst_dir = self.get_subdir(paths);
		match addon.kind {
			AddonKind::ResourcePack => if let InstKind::Client = self.kind {
				Some(inst_dir.join("resourcepacks"))
			} else {
				None
			}
			AddonKind::Mod => Some(inst_dir.join("mods")),
			AddonKind::Plugin => if let InstKind::Server = self.kind {
				Some(inst_dir.join("plugins"))
			} else {
				None
			}
			AddonKind::Shader => if let InstKind::Client = self.kind {
				Some(inst_dir.join("shaders"))
			} else {
				None
			}
		}
	}
	
	fn link_addon(dir: &Path, addon: &Addon, paths: &Paths) -> Result<(), CreateError> {
		files::create_dir(dir)?;
		let link = dir.join(&addon.name);
		if !link.exists() {
			fs::hard_link(addon.get_path(paths), dir.join(&addon.name))?;
		}
		Ok(())
	}
	
	pub fn create_addon(&self, addon: &Addon, paths: &Paths) -> Result<(), CreateError> {
		let inst_dir = self.get_subdir(paths);
		files::create_leading_dirs(&inst_dir)?;
		files::create_dir(&inst_dir)?;
		if let Some(path) = self.get_linked_addon_path(addon, paths) {
			Self::link_addon(&path, addon, paths)?;
		}
		
		Ok(())
	}

	pub fn remove_addon(&self, addon: &Addon, paths: &Paths) -> Result<(), CreateError> {
		if let Some(path) = self.get_linked_addon_path(addon, paths) {
			let path = path.join(&addon.name);
			if path.exists() {
				fs::remove_file(path)?;
			}
		}
		
		Ok(())
	}

	// Removes the paper server jar file from a server instance
	pub fn remove_paper(&self, paths: &Paths, paper_file_name: String) -> Result<(), CreateError> {
		let inst_dir = self.get_subdir(paths);
		let paper_path = inst_dir.join(paper_file_name);
		if paper_path.exists() {
			fs::remove_file(paper_path)?;
		}

		Ok(())
	}

	// Removes files such as the game jar for when the profile version changes
	pub fn teardown(&self, paths: &Paths, paper_file_name: Option<String>) -> Result<(), CreateError> {
		match self.kind {
			InstKind::Client => {
				let inst_dir = self.get_dir(paths);
				let jar_path = inst_dir.join("client.jar");
				if jar_path.exists() {
					fs::remove_file(jar_path)?;
				}
			}
			InstKind::Server => {
				let inst_dir = self.get_subdir(paths);
				let jar_path = inst_dir.join("server.jar");
				if jar_path.exists() {
					fs::remove_file(jar_path)?;
				}

				if let Some(file_name) = paper_file_name {
					self.remove_paper(paths, file_name)?;
				}
			}
		}

		Ok(())
	}
}
