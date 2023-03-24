pub mod create;
pub mod launch;

use anyhow::Context;

use self::launch::LaunchOptions;
use crate::io::files;
use crate::io::java::classpath::Classpath;
use crate::io::java::Java;
use crate::net::fabric_quilt;
use crate::util::json;
use crate::Paths;

use super::addon::{Addon, AddonKind, Modloader, PluginLoader};
use super::profile::update::UpdateManager;

use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum InstKind {
	Client,
	Server,
}

impl InstKind {
	pub fn from_str(string: &str) -> Option<Self> {
		match string {
			"client" => Some(Self::Client),
			"server" => Some(Self::Server),
			_ => None,
		}
	}
}

impl Display for InstKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", match self {
			Self::Client => "client",
			Self::Server => "server"
		})
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
	classpath: Option<Classpath>,
	jar_path: Option<PathBuf>,
	main_class: Option<String>,
}

impl Instance {
	pub fn new(
		kind: InstKind,
		id: &str,
		version: &str,
		modloader: Modloader,
		plugin_loader: PluginLoader,
		launch: LaunchOptions,
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
			jar_path: None,
			main_class: None,
		}
	}
	
	pub fn get_dir(&self, paths: &Paths) -> PathBuf {
		match &self.kind {
			InstKind::Client => paths.project.data_dir().join("client").join(&self.id),
			InstKind::Server => paths.project.data_dir().join("server").join(&self.id),
		}
	}
	
	pub fn get_subdir(&self, paths: &Paths) -> PathBuf {
		self.get_dir(paths).join(match self.kind {
			InstKind::Client => ".minecraft",
			InstKind::Server => "server",
		})
	}

	/// Set the java installation for the instance
	fn add_java(&mut self, version: &str, manager: &UpdateManager) {
		let mut java = manager.java.as_ref().expect("Update Manager Java is missing").clone();
		java.add_version(version);
		self.java = Some(java);
	}

	async fn get_fabric_quilt(
		&mut self,
		mode: fabric_quilt::Mode,
		paths: &Paths,
		manager: &UpdateManager,
	) -> anyhow::Result<Classpath> {
		let meta = fabric_quilt::get_meta(&self.version, &mode).await?;
		let classpath =
			fabric_quilt::download_files(&meta, paths, self.kind.clone(), mode, manager).await
				.context("Failed to download Fabric/Quilt")?;
		self.main_class = Some(match self.kind {
			InstKind::Client => meta.launcher_meta.main_class.client,
			InstKind::Server => meta.launcher_meta.main_class.server,
		});

		Ok(classpath)
	}
	
	pub fn get_linked_addon_path(&self, addon: &Addon, paths: &Paths) -> Option<PathBuf> {
		let inst_dir = self.get_subdir(paths);
		match addon.kind {
			AddonKind::ResourcePack => {
				if let InstKind::Client = self.kind {
					Some(inst_dir.join("resourcepacks"))
				} else {
					None
				}
			}
			AddonKind::Mod => Some(inst_dir.join("mods")),
			AddonKind::Plugin => {
				if let InstKind::Server = self.kind {
					Some(inst_dir.join("plugins"))
				} else {
					None
				}
			}
			AddonKind::Shader => {
				if let InstKind::Client = self.kind {
					Some(inst_dir.join("shaders"))
				} else {
					None
				}
			}
		}
	}

	fn link_addon(dir: &Path, addon: &Addon, paths: &Paths) -> anyhow::Result<()> {
		files::create_dir(dir)?;
		let link = dir.join(&addon.name);
		if !link.exists() {
			fs::hard_link(addon.get_path(paths), dir.join(&addon.name))
				.context("Failed to create hard link")?;
		}
		Ok(())
	}

	pub fn create_addon(&self, addon: &Addon, paths: &Paths) -> anyhow::Result<()> {
		let inst_dir = self.get_subdir(paths);
		files::create_leading_dirs(&inst_dir)?;
		files::create_dir(&inst_dir)?;
		if let Some(path) = self.get_linked_addon_path(addon, paths) {
			Self::link_addon(&path, addon, paths)
				.with_context(|| format!("Failed to link addon {}", addon.name))?;
		}

		Ok(())
	}

	pub fn remove_addon(&self, addon: &Addon, paths: &Paths) -> anyhow::Result<()> {
		if let Some(path) = self.get_linked_addon_path(addon, paths) {
			let path = path.join(&addon.name);
			if path.exists() {
				fs::remove_file(&path).with_context(|| format!("Failed to remove addon at {}", path.display()))?;
			}
		}

		Ok(())
	}

	// Removes the paper server jar file from a server instance
	pub fn remove_paper(&self, paths: &Paths, paper_file_name: String) -> anyhow::Result<()> {
		let inst_dir = self.get_subdir(paths);
		let paper_path = inst_dir.join(paper_file_name);
		if paper_path.exists() {
			fs::remove_file(paper_path).context("Failed to remove Paper jar")?;
		}

		Ok(())
	}

	// Removes files such as the game jar for when the profile version changes
	pub fn teardown(
		&self,
		paths: &Paths,
		paper_file_name: Option<String>,
	) -> anyhow::Result<()> {
		match self.kind {
			InstKind::Client => {
				let inst_dir = self.get_dir(paths);
				let jar_path = inst_dir.join("client.jar");
				if jar_path.exists() {
					fs::remove_file(jar_path).context("Failed to remove client.jar")?;
				}
			}
			InstKind::Server => {
				let inst_dir = self.get_subdir(paths);
				let jar_path = inst_dir.join("server.jar");
				if jar_path.exists() {
					fs::remove_file(jar_path).context("Failed to remove server.jar")?;
				}

				if let Some(file_name) = paper_file_name {
					self.remove_paper(paths, file_name).context("Failed to remove Paper")?;
				}
			}
		}

		Ok(())
	}
}
