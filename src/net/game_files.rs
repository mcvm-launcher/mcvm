use crate::Paths;
use crate::io::files::lib;
use crate::util::versions::{VersionNotFoundError, MinecraftVersion};
use crate::util::json::{self, JsonObject};
use crate::net::helper::{Download, DownloadError};
use crate::util::mojang;
use crate::util::print::ReplPrinter;

use color_print::{cprintln, cformat};

use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum VersionManifestError {
	#[error("Failed to download version manifest:\n{}", .0)]
	Download(#[from] DownloadError),
	#[error("Failed to evaluate json file:\n{}", .0)]
	ParseError(#[from] json::JsonError),
	#[error("File operation failed:\n\t{}", .0)]
	Io(#[from] std::io::Error)
}

// So we can do this without a retry
fn get_version_manifest_contents(paths: &Paths) -> Result<Box<Download>, VersionManifestError> {
	let mut path = paths.internal.join("versions");
	lib::create_dir(&path)?;
	path.push("manifest.json");

	let mut download = Download::new();
	download.url("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")?;
	download.add_str();
	download.add_file(path.as_path())?;
	download.perform()?;
	Ok(Box::new(download))
}

pub fn get_version_manifest(paths: &Paths, verbose: bool)
-> Result<(Box<json::JsonObject>, Box<Download>), VersionManifestError> {
	if verbose {
		println!("\tObtaining version index...");
	}

	let mut download = get_version_manifest_contents(paths)?;
	let mut manifest_contents = download.get_str()?;
	let manifest = match json::parse_object(&manifest_contents) {
		Ok(manifest) => manifest,
		Err(..) => {
			cprintln!("<r>Failed to parse version manifest. Redownloading...");
			download = get_version_manifest_contents(paths)?;
			manifest_contents = download.get_str()?;
			json::parse_object(&manifest_contents)?
		}
	};
	Ok((manifest, download))
}

#[derive(Debug, thiserror::Error)]
pub enum VersionJsonError {
	#[error("Version {} does not exist", .0)]
	VersionNotFound(#[from] VersionNotFoundError),
	#[error("Failed to evaluate json file:\n\t{}", .0)]
	ParseError(#[from] json::JsonError),
	#[error("{}", .0)]
	VersionManifest(#[from] VersionManifestError),
	#[error("Error when downloading version json:\n\t{}", .0)]
	Download(#[from] DownloadError),
	#[error("File operation failed:\n\t{}", .0)]
	Io(#[from] std::io::Error)
}

pub fn get_version_json(version: &MinecraftVersion, paths: &Paths, verbose: bool)
-> Result<(Box<json::JsonObject>, Box<Download>), VersionJsonError> {
	let version_string = version.as_string().to_owned();

	let (manifest_doc, mut download) = get_version_manifest(paths, verbose)?;
	// Find the version out of all of them
	let versions = json::access_array(&manifest_doc, "versions")?;
	let mut version_url: Option<&str> = None;
	for entry in versions.iter() {
		let obj = json::ensure_type(entry.as_object(), json::JsonType::Object)?;
		if json::access_str(obj, "id")? == version_string {
			version_url = Some(json::access_str(obj, "url")?);
		}
	}
	if version_url.is_none() {
		return Err(VersionJsonError::from(VersionNotFoundError::new(version)));
	}

	let version_json_name: String = version_string.clone() + ".json";
	let version_folder = paths.internal.join("versions").join(version_string);
	lib::create_dir(&version_folder)?;
	download.reset();
	download.url(version_url.expect("Version does not exist"))?;
	download.add_file(&version_folder.join(version_json_name))?;
	download.add_str();
	download.perform()?;

	let version_doc = json::parse_object(&download.get_str()?)?;

	Ok((version_doc, download))
}

#[derive(Debug, thiserror::Error)]
pub enum LibrariesError {
	#[error("Failed to evaluate json file:\n\t{}", .0)]
	ParseError(#[from] json::JsonError),
	#[error("Error when downloading library:\n\t{}", .0)]
	Download(#[from] DownloadError),
	#[error("Failed to convert string to UTF-8")]
	Utf,
	#[error("File operation failed:\n\t{}", .0)]
	Io(#[from] std::io::Error)
}

// Checks the rules of a library to see if it should be installed
fn is_library_allowed(lib: &JsonObject) -> Result<bool, LibrariesError> {
	if let Some(rules_val) = lib.get("rules") {
		let rules = json::ensure_type(rules_val.as_array(), json::JsonType::Array)?;
		for rule_val in rules.iter() {
			let rule = json::ensure_type(rule_val.as_object(), json::JsonType::Object)?;
			let action = json::access_str(rule, "action")?;
			if let Some(os_val) = rule.get("os") {
				let os = json::ensure_type(os_val.as_object(), json::JsonType::Object)?;
				let os_name = json::access_str(os, "name")?;
				if mojang::is_allowed(action) != (os_name == mojang::OS_STRING) {
					return Ok(false);
				}
			}
		}
	}
	Ok(true)
}

// Finishes up and downloads a library
fn download_library(
	download: &mut Download,
	lib_download: &json::JsonObject,
	path: &Path,
	classpath: &mut String
) -> Result<(), LibrariesError> {
	println!("lib");
	lib::create_leading_dirs(path)?;
	let url = json::access_str(lib_download, "url")?;
	download.reset();
	download.url(url)?;
	download.add_file(path)?;
	download.perform()?;
	Ok(())
}

pub fn get_libraries(
	version_json: &json::JsonObject,
	paths: &Paths,
	version: &MinecraftVersion,
	verbose: bool,
	force: bool
) -> Result<String, LibrariesError> {
	let libraries_path = paths.internal.join("libraries");
	lib::create_dir(&libraries_path)?;
	let natives_path = paths.internal.join("versions").join(version.as_string()).join("natives");
	lib::create_dir(&natives_path)?;
	let natives_jars_path = paths.internal.join("natives");
	// I can't figure out how to get curl multi to work with non-static write methods :( so this will be kinda slow
	// Might have to make it unsafe >:)
	
	let mut native_paths: Vec<PathBuf> = Vec::new();
	let mut classpath = String::new();
	let mut download = Download::new();
	let mut printer = ReplPrinter::new(verbose);
	printer.indent(1);

	let libraries = json::access_array(version_json, "libraries")?;
	printer.print(&cformat!("Downloading <b>{}</> libraries...", libraries.len()));

	for lib_val in libraries.iter() {
		let lib = json::ensure_type(lib_val.as_object(), json::JsonType::Object)?;
		if !is_library_allowed(lib)? {
			continue;
		}
		let name = json::access_str(lib, "name")?;
		let downloads = json::access_object(lib, "downloads")?;
		if let Some(natives_val) = lib.get("natives") {
			let natives = json::ensure_type(natives_val.as_object(), json::JsonType::Object)?;
			let key = json::access_str(natives, mojang::OS_STRING)?;
			let classifier = json::access_object(
				json::access_object(downloads, "classifiers")?, key
			)?;

			let path = natives_jars_path.join(json::access_str(classifier, "path")?);
			classpath.push_str(path.to_str().ok_or(LibrariesError::Utf)?);
			classpath.push(':');
			if !force && path.exists() {
				continue;
			}
			printer.print(&cformat!("Downloading library <b!>{}</>...", name));
			download_library(&mut download, classifier, &path, &mut classpath)?;
			native_paths.push(path);
			continue;
		}
		if let Some(artifact_val) = downloads.get("artifact") {
			let artifact = json::ensure_type(artifact_val.as_object(), json::JsonType::Object)?;
			let path = libraries_path.join(json::access_str(artifact, "path")?);
			classpath.push_str(path.to_str().ok_or(LibrariesError::Utf)?);
			classpath.push(':');
			if !force && path.exists() {
				continue;
			}
			printer.print(&cformat!("Downloading library <b>{}</>...", name));
			download_library(&mut download, artifact, &path, &mut classpath)?;
			continue;
		}
	}
	printer.print(&cformat!("<g>Libraries downloaded."));
	printer.finish();

	Ok(classpath)
}

#[derive(Debug, thiserror::Error)]
pub enum AssetsError {
	#[error("Failed to evaluate json file: {}", .0)]
	ParseError(#[from] json::JsonError),
	#[error("Error when downloading asset:\n\t{}", .0)]
	Download(#[from] DownloadError),
	#[error("File operation failed:\n\t{}", .0)]
	Io(#[from] std::io::Error)
}

fn download_asset_index(url: &str, path: &Path) -> Result<Box<json::JsonObject>, AssetsError> {
	let mut download = Download::new();
	download.url(url)?;
	download.add_file(path)?;
	download.add_str();
	download.perform()?;
	
	let doc = json::parse_object(&download.get_str()?)?;
	Ok(doc)
}

// Download a single asset from the index. Returns false if the loop should continue
fn download_asset(
	asset: &json::JsonObject,
	objects_dir: &Path,
	download: &mut Download,
	force: bool
) -> Result<bool, AssetsError> {
	let hash = json::access_str(asset, "hash")?.to_owned();
	let hash_path = hash[..2].to_owned() + "/" + &hash;
	let url = "https://resources.download.minecraft.net/".to_owned() + &hash_path;
	
	let path = objects_dir.join(&hash_path);
	if !force && path.exists() {
		return Ok(false);
	}
	lib::create_leading_dirs(&path)?;
	
	download.reset();
	download.url(&url)?;
	download.add_file(&path)?;
	download.perform()?;

	Ok(true)
}

pub fn get_assets(
	version_json: &json::JsonObject,
	paths: &Paths,
	version: &MinecraftVersion,
	verbose: bool,
	force: bool
) -> Result<(), AssetsError> {
	let version_string = version.as_string().to_owned();
	let indexes_dir = paths.assets.join("indexes");
	lib::create_dir(&indexes_dir)?;
	
	let index_path = indexes_dir.join(version_string + ".json");
	let index_url = json::access_str(
		json::access_object(version_json, "assetIndex")?, "url"
	)?;
	
	let objects_dir = paths.assets.join("objects");
	lib::create_dir(&objects_dir)?;
	let virtual_dir = paths.assets.join("virtual");
	if !force && virtual_dir.exists() && !virtual_dir.is_symlink() {
		lib::dir_symlink(&virtual_dir, &objects_dir)?;
	}

	let index = match download_asset_index(index_url, &index_path) {
		Ok(val) => val,
		Err(err) => {
			cprintln!("<r>Failed to obtain asset index:\n\t{}\nRedownloading...", err);
			download_asset_index(index_url, &index_path)?
		}
	};

	let assets = json::access_object(&index, "objects")?;
	let count = assets.len();
	if verbose {
		cprintln!("\tDownloading <b>{}</> assets...", count);
	}

	let mut download = Download::new();
	let mut printer = ReplPrinter::new(verbose);
	printer.indent(1);
	for (i, (key, asset_val)) in assets.iter().enumerate() {
		let asset = json::ensure_type(asset_val.as_object(), json::JsonType::Object)?;
		if !download_asset(asset, &objects_dir, &mut download, force)? {
			continue;
		}
		printer.print(&cformat!("(<b>{}</b><k!>/</k!><b>{}</b>) <k!>{}", i, count, key));
	}
	printer.print(&cformat!("<g>Assets downloaded."));
	printer.finish();

	Ok(())
}
