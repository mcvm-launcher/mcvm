use std::fmt::Display;
use std::fs::File;

use anyhow::{anyhow, Context};
use color_print::cformat;
use reqwest::Client;
use serde::Deserialize;

use crate::data::profile::update::UpdateManager;
use crate::io::files;
use crate::io::files::paths::Paths;
use crate::io::java::classpath::Classpath;
use crate::util::print::ReplPrinter;
use shared::instance::Side;

use super::download;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
	Fabric,
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

#[derive(Debug, Deserialize, Clone)]
pub struct Library {
	name: String,
	url: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct MainLibrary {
	maven: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Libraries {
	common: Vec<Library>,
	client: Vec<Library>,
	server: Vec<Library>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MainClass {
	pub client: String,
	pub server: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LauncherMeta {
	libraries: Libraries,
	#[serde(rename = "mainClass")]
	pub main_class: MainClass,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FabricQuiltMeta {
	#[serde(rename = "launcherMeta")]
	pub launcher_meta: LauncherMeta,
	pub loader: MainLibrary,
	pub intermediary: MainLibrary,
}

/// Sections of a library string
#[derive(Debug, PartialEq)]
struct LibraryParts {
	orgs: Vec<String>,
	package: String,
	version: String,
}

impl LibraryParts {
	/// Extract the parts of a library string
	pub fn from_str(string: &str) -> Option<Self> {
		let mut parts = string.split(':');
		let orgs: Vec<String> = parts.next()?.split('.').map(|x| x.to_owned()).collect();
		let package = parts.next()?.to_owned();
		let version = parts.next()?.to_owned();
		Some(Self {
			orgs,
			package,
			version,
		})
	}
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
	
	let meta = if manager.allow_offline && manager.should_update_file(&path) {
		let mut file = File::open(path).context("Failed to open {mode} meta file")?;
		serde_json::from_reader(&mut file).context("Failed to parse {mode} meta from file")?
	} else {
		let meta = download::json::<Vec<FabricQuiltMeta>>(&meta_url)
			.await
			.context("Failed to download {mode} metadata")?;
		let meta = meta
			.first()
			.ok_or(anyhow!("Could not find a valid {mode} version"))?;

		meta.clone()
	};

	Ok(meta)
}

/// Get the path to a library
fn get_lib_path(name: &str) -> Option<String> {
	let parts = LibraryParts::from_str(name)?;
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
			let resp = client
				.get(url)
				.send()
				.await?
				.error_for_status()?
				.bytes()
				.await?;
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
	let client = Client::new();
	let resp = client
		.get(url)
		.send()
		.await?
		.error_for_status()?
		.bytes()
		.await?;
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
pub fn get_classpath(
	meta: &FabricQuiltMeta,
	paths: &Paths,
	side: Side,
) -> Classpath {
	let mut out = Classpath::new();

	out.extend(get_lib_list_classpath(&meta.launcher_meta.libraries.common, paths));

	let side_libs = match side {
		Side::Client => &meta.launcher_meta.libraries.client,
		Side::Server => &meta.launcher_meta.libraries.server,
	};

	out.extend(get_lib_list_classpath(side_libs, paths));

	let path = get_lib_path(&meta.loader.maven).expect("Expected a valid path");
	out.add_path(&paths.libraries.join(&path));

	let path = get_lib_path(&meta.intermediary.maven).expect("Expected a valid path");
	out.add_path(&paths.libraries.join(&path));
	
	out
}

/// Download files for Quilt/Fabric that are common for both client and server
pub async fn download_files(
	meta: &FabricQuiltMeta,
	paths: &Paths,
	mode: Mode,
	manager: &UpdateManager,
) -> anyhow::Result<()> {
	let force = manager.force;
	let mut printer = ReplPrinter::from_options(manager.print.clone());
	printer.print(&format!("Downloading {mode}"));
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
		.context("Failed to download {mode} common libraries")?;
	main_libs_task
		.await?
		.context("Failed to download {mode} main libraries")?;

	printer.print(&cformat!("<g>{} downloaded.", mode));

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

	download_libraries(&libs, &paths, manager.force).await?;
	
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_library_destructuring() {
		assert_eq!(
			LibraryParts::from_str("foo.bar.baz:hel.lo:wo.rld")
				.expect("Parts did not parse correctly"),
			LibraryParts {
				orgs: vec![
					String::from("foo"),
					String::from("bar"),
					String::from("baz")
				],
				package: String::from("hel.lo"),
				version: String::from("wo.rld")
			}
		)
	}
}
