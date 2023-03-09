use std::fs;

use color_print::cformat;
use reqwest::Client;
use serde::Deserialize;

use crate::data::instance::InstKind;
use crate::io::files;
use crate::io::files::paths::Paths;
use crate::io::java::classpath::Classpath;
use crate::util::json::{self, JsonType};
use crate::util::print::ReplPrinter;


#[derive(Debug, thiserror::Error)]
pub enum FabricError {
	#[error("Failed to evaluate json file:\n{}", .0)]
	ParseError(#[from] json::JsonError),
	#[error("Failed to parse json file:\n{}", .0)]
	Serde(#[from] serde_json::Error),
	#[error("Error when downloading modloader:\n{}", .0)]
	Download(#[from] reqwest::Error),
	#[error("File operation failed:\n{}", .0)]
	Io(#[from] std::io::Error),
	#[error("Failed to join task:\n{}", .0)]
	Join(#[from] tokio::task::JoinError),
	#[error("No compatible modloader version found")]
	NoneFound,
}

#[derive(Debug, Deserialize, Clone)]
pub struct QuiltLibrary {
	name: String,
	url: String,
}

#[derive(Deserialize, Clone)]
pub struct QuiltMainLibrary {
	maven: String,
}

#[derive(Deserialize)]
pub struct QuiltLibraries {
	common: Vec<QuiltLibrary>,
	client: Vec<QuiltLibrary>,
	server: Vec<QuiltLibrary>,
}

#[derive(Deserialize)]
pub struct MainClass {
	pub client: String,
	pub server: String,
}

#[derive(Deserialize)]
pub struct LauncherMeta {
	libraries: QuiltLibraries,
	#[serde(rename = "mainClass")]
	pub main_class: MainClass,
}

#[derive(Deserialize)]
pub struct QuiltMeta {
	#[serde(rename = "launcherMeta")]
	pub launcher_meta: LauncherMeta,
	pub loader: QuiltMainLibrary,
	pub intermediary: QuiltMainLibrary
}

#[derive(Debug, PartialEq)]
struct LibraryParts {
	orgs: Vec<String>,
	package: String,
	version: String,
}

impl LibraryParts {
	pub fn from_str(string: &str) -> Option<Self> {
		let mut parts = string.split(':');
		let orgs: Vec<String> = parts.nth(0)?.split('.').map(|x| x.to_owned()).collect();
		let package = parts.nth(0)?.to_owned();
		let version = parts.nth(0)?.to_owned();
		Some(Self {
			orgs,
			package,
			version,
		})
	}
}

pub async fn get_quilt_meta(version: &str) -> Result<QuiltMeta, FabricError> {
	let meta_url = format!("https://meta.quiltmc.org/v3/versions/loader/{version}");
	let client = Client::new();
	let meta = client.get(&meta_url).send().await?.error_for_status()?.text().await?;
	let meta = json::parse_json(&meta)?;
	let meta = json::ensure_type(meta.as_array(), JsonType::Arr)?;
	let meta = meta.first().ok_or(FabricError::NoneFound)?;

	Ok(serde_json::from_value(meta.clone())?)
}

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

async fn download_quilt_libraries(
	libs: &[QuiltLibrary],
	paths: &Paths,
	force: bool,
) -> Result<Classpath, FabricError> {
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
			let resp = client.get(url).send().await?.error_for_status()?.bytes().await?;
			files::create_leading_dirs(&lib_path)?;
			fs::write(&lib_path, resp)?;
		}
	}

	Ok(classpath)
}

async fn download_quilt_main_library(
	lib: &QuiltMainLibrary,
	url: &str,
	paths: &Paths,
	force: bool
) -> Result<String, FabricError> {
	let path = get_lib_path(&lib.maven).expect("Expected a valid path");
	let lib_path = paths.libraries.join(&path);
	let lib_path_str = lib_path.to_str().expect("Failed to convert path to a string").to_owned();
	if !force && lib_path.exists() {
		return Ok(lib_path_str);
	}
	let url = url.to_owned() + &path;
	let client = Client::new();
	let resp = client.get(url).send().await?.error_for_status()?.bytes().await?;
	files::create_leading_dirs(&lib_path)?;
	fs::write(&lib_path, resp)?;
	Ok(lib_path_str)
}

pub async fn download_quilt_files(
	meta: &QuiltMeta,
	paths: &Paths,
	side: InstKind,
	verbose: bool,
	force: bool,
) -> Result<Classpath, FabricError> {
	let mut printer = ReplPrinter::new(verbose);
	printer.indent(1);
	printer.print("Downloading Quilt...");
	let mut classpath = Classpath::new();
	let libs = meta.launcher_meta.libraries.common.clone();
	let paths_clone = paths.clone();
	let common_task = tokio::spawn(
		async move { download_quilt_libraries(&libs, &paths_clone, force).await }
	);

	let libs = match side {
		InstKind::Client => meta.launcher_meta.libraries.client.clone(),
		InstKind::Server => meta.launcher_meta.libraries.server.clone(),
	};
	let paths_clone = paths.clone();
	let side_task = tokio::spawn(
		async move { download_quilt_libraries(&libs, &paths_clone, force).await }
	);

	let paths_clone = paths.clone();
	let loader_clone = meta.loader.clone();
	let intermediary_clone = meta.intermediary.clone();
	let main_libs_task = tokio::spawn(async move {
		(
			download_quilt_main_library(&loader_clone, "https://maven.quiltmc.org/repository/release/", &paths_clone, force).await,
			download_quilt_main_library(&intermediary_clone, "https://maven.fabricmc.net/", &paths_clone, force).await,
		)
	});

	classpath.extend(common_task.await??);
	classpath.extend(side_task.await??);
	let (loader_name, intermediary_name) = main_libs_task.await?;
	classpath.add(&loader_name?);
	classpath.add(&intermediary_name?);

	printer.print(&cformat!("<g>Quilt downloaded."));

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
