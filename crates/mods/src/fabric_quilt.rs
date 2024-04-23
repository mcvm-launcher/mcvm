use std::fmt::Display;

use anyhow::{anyhow, Context};
use mcvm_core::io::java::classpath::Classpath;
use mcvm_core::io::java::maven::MavenLibraryParts;
use mcvm_core::io::json_from_file;
use mcvm_core::io::update::UpdateManager;
use mcvm_core::io::{files, json_to_file};
use mcvm_core::net::download;
use mcvm_core::{MCVMCore, Paths};
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel, OutputProcess};
use mcvm_shared::versions::VersionInfo;
use mcvm_shared::Side;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;

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

/// Install Fabric/Quilt using the core and information about the version.
/// First, create the core and the version you want. Then, get the version info from the version.
/// Finally, run this function. Returns the classpath and main class to add to the instance you are launching
pub async fn install_from_core(
	core: &mut MCVMCore,
	version_info: &VersionInfo,
	mode: Mode,
	side: Side,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<(Classpath, String)> {
	let meta = get_meta(
		&version_info.version,
		&mode,
		core.get_paths(),
		core.get_update_manager(),
		core.get_client(),
	)
	.await
	.context("Failed to download Fabric/Quilt metadata")?;
	download_files(
		&meta,
		core.get_paths(),
		mode,
		core.get_update_manager(),
		core.get_client(),
		o,
	)
	.await
	.context("Failed to download common Fabric/Quilt files")?;

	download_side_specific_files(
		&meta,
		core.get_paths(),
		side,
		core.get_update_manager(),
		core.get_client(),
	)
	.await
	.context("Failed to download {mode} files for {side}")?;

	let classpath = get_classpath(&meta, core.get_paths(), side);

	Ok((
		classpath,
		meta.launcher_meta
			.main_class
			.get_main_class_string(side)
			.into(),
	))
}

/// Metadata for Fabric or Quilt
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FabricQuiltMeta {
	/// Metadata for the launcher
	#[serde(rename = "launcherMeta")]
	pub launcher_meta: LauncherMeta,
	/// The main library to use for the loader
	pub loader: MainLibrary,
	/// The main library to use for intermediary mappings
	pub intermediary: MainLibrary,
}

/// Metadata for the launcher
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LauncherMeta {
	libraries: Libraries,
	/// The main class to override with when launching
	#[serde(rename = "mainClass")]
	pub main_class: MainClass,
}

/// A library in the Fabric/Quilt meta
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Library {
	name: String,
	#[serde(default = "default_library_url")]
	url: String,
}

/// Old format does not have a URL for the net.minecraft.launchwrapper for some reason
fn default_library_url() -> String {
	"https://repo.papermc.io/repository/maven-public/".into()
}

/// An important library in the Fabric/Quilt meta
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MainLibrary {
	maven: String,
}

/// The struct of libraries for different sides
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Libraries {
	common: Vec<Library>,
	client: Vec<Library>,
	server: Vec<Library>,
}

/// A Java main class override provided by the meta
#[derive(Deserialize, Serialize, Debug, Clone)]
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

/// Get the Fabric/Quilt metadata file
pub async fn get_meta(
	version: &str,
	mode: &Mode,
	paths: &Paths,
	manager: &UpdateManager,
	client: &Client,
) -> anyhow::Result<FabricQuiltMeta> {
	let meta_url = match mode {
		Mode::Fabric => format!("https://meta.fabricmc.net/v2/versions/loader/{version}"),
		Mode::Quilt => format!("https://meta.quiltmc.org/v3/versions/loader/{version}"),
	};
	let mode_lowercase = mode.to_string().to_lowercase();
	let path = paths
		.internal
		.join("fabric_quilt")
		.join(format!("meta_{mode_lowercase}_{version}.json"));
	files::create_leading_dirs_async(&path)
		.await
		.context("Failed to create parent directories for Fabric/Quilt meta")?;

	let meta = if manager.allow_offline() && path.exists() {
		json_from_file(path).with_context(|| format!("Failed to parse {mode} meta from file"))?
	} else {
		let bytes = download::bytes(&meta_url, client)
			.await
			.with_context(|| format!("Failed to download {mode} metadata file"))?;
		let out = serde_json::from_slice::<Vec<FabricQuiltMeta>>(&bytes)
			.context("Failed to parse downloaded metadata")?;

		json_to_file(path, &out).context("Failed to serialize meta to a file")?;

		out
	};

	let meta = meta
		.first()
		.ok_or(anyhow!("Could not find a valid {mode} version"))?;

	Ok(meta.clone())
}

/// Download files for Quilt/Fabric that are common for both client and server
pub async fn download_files(
	meta: &FabricQuiltMeta,
	paths: &Paths,
	mode: Mode,
	manager: &UpdateManager,
	client: &Client,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<()> {
	let force = manager.force_reinstall();

	let process = OutputProcess::new(o);
	process.0.display(
		MessageContents::StartProcess(format!("Downloading {mode}")),
		MessageLevel::Important,
	);

	let libs = meta.launcher_meta.libraries.common.clone();
	let paths_clone = paths.clone();

	let client_clone = client.clone();
	let common_task =
		tokio::spawn(
			async move { download_libraries(&libs, &paths_clone, &client_clone, force).await },
		);

	let paths_clone = paths.clone();
	let loader_clone = meta.loader.clone();
	let intermediary_clone = meta.intermediary.clone();
	let loader_url = match mode {
		Mode::Fabric => "https://maven.fabricmc.net/",
		Mode::Quilt => "https://maven.quiltmc.org/repository/release/",
	};

	let client_clone = client.clone();
	let main_libs_task = tokio::spawn(async move {
		let task1 = download_main_library(
			&loader_clone,
			loader_url,
			&paths_clone,
			&client_clone,
			force,
		);
		let task2 = download_main_library(
			&intermediary_clone,
			"https://maven.fabricmc.net/",
			&paths_clone,
			&client_clone,
			force,
		);

		tokio::try_join!(task1, task2)?;

		Ok::<(), anyhow::Error>(())
	});

	let (res1, res2) = tokio::try_join!(common_task, main_libs_task)?;
	res1.with_context(|| format!("Failed to download {mode} common libraries"))?;
	res2.with_context(|| format!("Failed to download {mode} main libraries"))?;

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
	client: &Client,
) -> anyhow::Result<()> {
	let libs = match side {
		Side::Client => &meta.launcher_meta.libraries.client,
		Side::Server => &meta.launcher_meta.libraries.server,
	};

	download_libraries(libs, paths, client, manager.force_reinstall()).await?;

	Ok(())
}

/// Get the path to a library
fn get_lib_path(name: &str) -> Option<String> {
	let parts = MavenLibraryParts::parse_from_str(name)?;
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
	client: &Client,
	force: bool,
) -> anyhow::Result<Classpath> {
	let mut classpath = Classpath::new();
	let mut tasks = JoinSet::new();
	for lib in libs.iter() {
		let path = get_lib_path(&lib.name);
		if let Some(path) = path {
			let lib_path = paths.libraries.join(&path);
			classpath.add_path(&lib_path);
			if !force && lib_path.exists() {
				continue;
			}
			let url = lib.url.clone() + &path;

			let client = client.clone();
			let task = async move {
				files::create_leading_dirs_async(&lib_path).await?;
				let resp = download::bytes(url, &client).await?;
				tokio::fs::write(&lib_path, resp).await?;
				Ok::<(), anyhow::Error>(())
			};

			tasks.spawn(task);
		}

		while let Some(result) = tasks.join_next().await {
			result??;
		}
	}

	Ok(classpath)
}

/// Download a main library from Fabric or Quilt, such as the loader or mappings
async fn download_main_library(
	lib: &MainLibrary,
	url: &str,
	paths: &Paths,
	client: &Client,
	force: bool,
) -> anyhow::Result<()> {
	let path = get_lib_path(&lib.maven).expect("Expected a valid path");
	let lib_path = paths.libraries.join(&path);
	if !force && lib_path.exists() {
		return Ok(());
	}
	let url = url.to_owned() + &path;
	let resp = download::bytes(url, client).await?;

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
