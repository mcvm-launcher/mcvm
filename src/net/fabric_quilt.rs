use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;

use anyhow::{anyhow, Context};
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel, OutputProcess};
use reqwest::Client;
use serde::Deserialize;

use crate::data::profile::update::manager::UpdateManager;
use crate::io::files;
use crate::io::files::paths::Paths;
use crate::io::java::classpath::Classpath;
use crate::io::java::maven::MavenLibraryParts;
use mcvm_shared::instance::Side;

use super::download;

/// Mode we are in (Fabric / Quilt)
/// This way we don't have to duplicate a lot of functions since these both
/// have very similar download steps
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
	/// Fabric loader
	Fabric,
	/// Quilt loader
	Quilt,
}

impl Display for Mode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Fabric => "Fabric",
				Self::Quilt => "Quilt",
			}
		)
	}
}

/// A library in the Fabric/Quilt meta
#[derive(Debug, Deserialize, Clone)]
pub struct Library {
	name: String,
	#[serde(default = "default_library_url")]
	url: String,
}

/// Old format does not have a URL for the net.minecraft.launchwrapper for some reason
fn default_library_url() -> String {
	String::from("https://repo.papermc.io/repository/maven-public/")
}

/// An important library in the Fabric/Quilt meta
#[derive(Deserialize, Clone, Debug)]
pub struct MainLibrary {
	maven: String,
}

/// The struct of libraries for different sides
#[derive(Deserialize, Debug, Clone)]
pub struct Libraries {
	common: Vec<Library>,
	client: Vec<Library>,
	server: Vec<Library>,
}

/// A Java main class override provided by the meta
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MainClass {
	/// The new format with a different string for client and server
	New {
		/// Main class for the client
		client: String,
		/// Main class for the server
		server: String,
	},
	/// The old format with the same main class for both sides
	Old(String),
}

impl MainClass {
	/// Get the main class as a string
	pub fn get_main_class_string(&self, side: Side) -> &str {
		match self {
			Self::New { client, server } => match side {
				Side::Client => client,
				Side::Server => server,
			},
			Self::Old(class) => class,
		}
	}
}

/// Metadata for the launcher
#[derive(Deserialize, Debug, Clone)]
pub struct LauncherMeta {
	libraries: Libraries,
	/// The main class to override with when launching
	#[serde(rename = "mainClass")]
	pub main_class: MainClass,
}

/// Metadata for Fabric or Quilt
#[derive(Deserialize, Debug, Clone)]
pub struct FabricQuiltMeta {
	/// Metadata for the launcher
	#[serde(rename = "launcherMeta")]
	pub launcher_meta: LauncherMeta,
	/// The main library to use for the loader
	pub loader: MainLibrary,
	/// The main library to use for intermediary mappings
	pub intermediary: MainLibrary,
}

/// Get the Fabric/Quilt metadata file
pub async fn get_meta(
	version: &str,
	mode: &Mode,
	paths: &Paths,
	manager: &UpdateManager,
) -> anyhow::Result<FabricQuiltMeta> {
	let meta_url = match mode {
		Mode::Fabric => format!("https://meta.fabricmc.net/v2/versions/loader/{version}"),
		Mode::Quilt => format!("https://meta.quiltmc.org/v3/versions/loader/{version}"),
	};
	let path = paths.internal.join(format!("fq_{mode}_meta.json"));

	let meta = if manager.allow_offline && path.exists() {
		let file = File::open(path).with_context(|| format!("Failed to open {mode} meta file"))?;
		let mut file = BufReader::new(file);
		serde_json::from_reader(&mut file)
			.with_context(|| format!("Failed to parse {mode} meta from file"))?
	} else {
		let bytes = download::bytes(&meta_url, &Client::new())
			.await
			.with_context(|| format!("Failed to download {mode} metadata file"))?;
		tokio::fs::write(path, &bytes)
			.await
			.context("Failed to write meta to a file")?;

		serde_json::from_slice::<Vec<FabricQuiltMeta>>(&bytes)
			.context("Failed to parse downloaded metadata")?
	};

	let meta = meta
		.first()
		.ok_or(anyhow!("Could not find a valid {mode} version"))?;

	Ok(meta.clone())
}

/// Get the path to a library
fn get_lib_path(name: &str) -> Option<String> {
	let parts = MavenLibraryParts::from_str(name)?;
	let mut url = String::new();
	for org in parts.orgs {
		url.push_str(&org);
		url.push('/');
	}
	url.push_str(&format!(
		"{package}/{version}/{package}-{version}.jar",
		package = parts.package,
		version = parts.version
	));

	Some(url)
}

/// Download all Fabric/Quilt libraries. Returns the resulting classpath.
async fn download_libraries(
	libs: &[Library],
	paths: &Paths,
	force: bool,
) -> anyhow::Result<Classpath> {
	let mut classpath = Classpath::new();
	let client = Client::new();
	for lib in libs.iter() {
		let path = get_lib_path(&lib.name);
		if let Some(path) = path {
			let lib_path = paths.libraries.join(&path);
			classpath.add_path(&lib_path);
			if !force && lib_path.exists() {
				continue;
			}
			let url = lib.url.clone() + &path;
			files::create_leading_dirs(&lib_path)?;
			let resp = download::bytes(url, &client).await?;
			tokio::fs::write(&lib_path, resp).await?;
		}
	}

	Ok(classpath)
}

/// Download a main library from Fabric or Quilt, such as the loader or mappings
async fn download_main_library(
	lib: &MainLibrary,
	url: &str,
	paths: &Paths,
	force: bool,
) -> anyhow::Result<()> {
	let path = get_lib_path(&lib.maven).expect("Expected a valid path");
	let lib_path = paths.libraries.join(&path);
	if !force && lib_path.exists() {
		return Ok(());
	}
	let url = url.to_owned() + &path;
	let resp = download::bytes(url, &Client::new()).await?;

	files::create_leading_dirs_async(&lib_path).await?;
	tokio::fs::write(&lib_path, resp).await?;

	Ok(())
}

/// Get the classpath of a list of libraries
fn get_lib_list_classpath(libs: &[Library], paths: &Paths) -> Classpath {
	let mut out = Classpath::new();

	for lib in libs.iter() {
		let path = get_lib_path(&lib.name);
		if let Some(path) = path {
			let lib_path = paths.libraries.join(&path);
			out.add_path(&lib_path);
		}
	}

	out
}

/// Get the classpath for Quilt/Fabric
pub fn get_classpath(meta: &FabricQuiltMeta, paths: &Paths, side: Side) -> Classpath {
	let mut out = Classpath::new();

	out.extend(get_lib_list_classpath(
		&meta.launcher_meta.libraries.common,
		paths,
	));

	let side_libs = match side {
		Side::Client => &meta.launcher_meta.libraries.client,
		Side::Server => &meta.launcher_meta.libraries.server,
	};

	out.extend(get_lib_list_classpath(side_libs, paths));

	let path = get_lib_path(&meta.loader.maven).expect("Expected a valid path");
	out.add_path(&paths.libraries.join(path));

	let path = get_lib_path(&meta.intermediary.maven).expect("Expected a valid path");
	out.add_path(&paths.libraries.join(path));

	out
}

/// Download files for Quilt/Fabric that are common for both client and server
pub async fn download_files(
	meta: &FabricQuiltMeta,
	paths: &Paths,
	mode: Mode,
	manager: &UpdateManager,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<()> {
	let force = manager.force;

	let process = OutputProcess::new(o);
	process.0.display(
		MessageContents::StartProcess(format!("Downloading {mode}")),
		MessageLevel::Important,
	);

	let libs = meta.launcher_meta.libraries.common.clone();
	let paths_clone = paths.clone();
	let common_task =
		tokio::spawn(async move { download_libraries(&libs, &paths_clone, force).await });

	let paths_clone = paths.clone();
	let loader_clone = meta.loader.clone();
	let intermediary_clone = meta.intermediary.clone();
	let loader_url = match mode {
		Mode::Fabric => "https://maven.fabricmc.net/",
		Mode::Quilt => "https://maven.quiltmc.org/repository/release/",
	};
	let main_libs_task = tokio::spawn(async move {
		download_main_library(&loader_clone, loader_url, &paths_clone, force).await?;
		download_main_library(
			&intermediary_clone,
			"https://maven.fabricmc.net/",
			&paths_clone,
			force,
		)
		.await?;

		Ok::<(), anyhow::Error>(())
	});

	common_task
		.await?
		.with_context(|| format!("Failed to download {mode} common libraries"))?;
	main_libs_task
		.await?
		.with_context(|| format!("Failed to download {mode} main libraries"))?;

	process.0.display(
		MessageContents::Success(format!("{mode} downloaded")),
		MessageLevel::Important,
	);

	Ok(())
}

/// Download files for Quilt/Fabric that are side-specific
pub async fn download_side_specific_files(
	meta: &FabricQuiltMeta,
	paths: &Paths,
	side: Side,
	manager: &UpdateManager,
) -> anyhow::Result<()> {
	let libs = match side {
		Side::Client => meta.launcher_meta.libraries.client.clone(),
		Side::Server => meta.launcher_meta.libraries.server.clone(),
	};

	download_libraries(&libs, paths, manager.force).await?;

	Ok(())
}
