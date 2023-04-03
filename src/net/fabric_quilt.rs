use std::fmt::Display;

use anyhow::{anyhow, Context};
use color_print::cformat;
use reqwest::Client;
use serde::Deserialize;

use crate::data::instance::Side;
use crate::data::profile::update::UpdateManager;
use crate::io::files;
use crate::io::files::paths::Paths;
use crate::io::java::classpath::Classpath;
use crate::util::json::{self, JsonType};
use crate::util::print::ReplPrinter;

use super::download::download_text;

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

#[derive(Deserialize, Clone)]
pub struct MainLibrary {
	maven: String,
}

#[derive(Deserialize)]
pub struct Libraries {
	common: Vec<Library>,
	client: Vec<Library>,
	server: Vec<Library>,
}

#[derive(Deserialize)]
pub struct MainClass {
	pub client: String,
	pub server: String,
}

#[derive(Deserialize)]
pub struct LauncherMeta {
	libraries: Libraries,
	#[serde(rename = "mainClass")]
	pub main_class: MainClass,
}

#[derive(Deserialize)]
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
pub async fn get_meta(version: &str, mode: &Mode) -> anyhow::Result<FabricQuiltMeta> {
	let meta_url = match mode {
		Mode::Fabric => format!("https://meta.fabricmc.net/v2/versions/loader/{version}"),
		Mode::Quilt => format!("https://meta.quiltmc.org/v3/versions/loader/{version}"),
	};
	let meta = download_text(&meta_url)
		.await
		.context("Failed to download Fabric/Quilt metadata")?;
	let meta = json::parse_json(&meta).context("Failed to parse Fabric/Quilt metadata")?;
	let meta = json::ensure_type(meta.as_array(), JsonType::Arr)?;
	let meta = meta
		.first()
		.ok_or(anyhow!("Could not find a valid Fabric/Quilt version"))?;

	Ok(serde_json::from_value(meta.clone())?)
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
) -> anyhow::Result<String> {
	let path = get_lib_path(&lib.maven).expect("Expected a valid path");
	let lib_path = paths.libraries.join(&path);
	let lib_path_str = lib_path
		.to_str()
		.expect("Failed to convert path to a string")
		.to_owned();
	if !force && lib_path.exists() {
		return Ok(lib_path_str);
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
	Ok(lib_path_str)
}

/// Download all needed files for Quilt/Fabric
pub async fn download_files(
	meta: &FabricQuiltMeta,
	paths: &Paths,
	side: Side,
	mode: Mode,
	manager: &UpdateManager,
) -> anyhow::Result<Classpath> {
	let force = manager.force;
	let mut printer = ReplPrinter::from_options(manager.print.clone());
	printer.print(&format!("Downloading {mode}"));
	let mut classpath = Classpath::new();
	let libs = meta.launcher_meta.libraries.common.clone();
	let paths_clone = paths.clone();
	let common_task =
		tokio::spawn(async move { download_libraries(&libs, &paths_clone, force).await });

	let libs = match side {
		Side::Client => meta.launcher_meta.libraries.client.clone(),
		Side::Server => meta.launcher_meta.libraries.server.clone(),
	};
	let paths_clone = paths.clone();
	let side_task =
		tokio::spawn(async move { download_libraries(&libs, &paths_clone, force).await });

	let paths_clone = paths.clone();
	let loader_clone = meta.loader.clone();
	let intermediary_clone = meta.intermediary.clone();
	let loader_url = match mode {
		Mode::Fabric => "https://maven.fabricmc.net/",
		Mode::Quilt => "https://maven.quiltmc.org/repository/release/",
	};
	let main_libs_task = tokio::spawn(async move {
		(
			download_main_library(&loader_clone, loader_url, &paths_clone, force).await,
			download_main_library(
				&intermediary_clone,
				"https://maven.fabricmc.net/",
				&paths_clone,
				force,
			)
			.await,
		)
	});

	classpath.extend(
		common_task
			.await?
			.context("Failed to download Fabric/Quilt common libraries")?,
	);
	classpath.extend(
		side_task
			.await?
			.context("Failed to download Fabric/Quilt side-specific libraries")?,
	);
	let (loader_name, intermediary_name) = main_libs_task.await?;
	classpath.add(&loader_name?);
	classpath.add(&intermediary_name?);

	printer.print(&cformat!("<g>{} downloaded.", mode));

	Ok(classpath)
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
