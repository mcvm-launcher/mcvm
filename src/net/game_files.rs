use crate::Paths;
use crate::io::files::files::create_existing_dir;
use crate::lib::versions::{VersionNotFoundError, MinecraftVersion};
use crate::lib::json;
use crate::net::helper;
use crate::net::helper::Download;

use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum VersionManifestError {
	#[error("Failed to download version manifest:\n{}", .0)]
	Download(#[from] helper::DownloadError),
	#[error("Failed to evaluate json file: {}", .0)]
	ParseError(#[from] json::JsonError)
}

// So we can do this without a retry
fn get_version_manifest_contents(paths: &Paths) -> Result<Box<Download>, VersionManifestError> {
	let mut path = paths.internal.join("versions");
	create_existing_dir(&path).unwrap();
	path.push("manifest.json");

	let mut download = Download::new();
	download.url("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")?;
	download.add_str();
	download.add_file(path.as_path())?;
	download.perform()?;
	Ok(Box::new(download))
}

pub fn get_version_manifest(paths: &Paths, verbose: bool) -> Result<(Box<Value>, Box<Download>), VersionManifestError> {
	if verbose {
		println!("\tObtaining version index...");
	}

	let mut download = get_version_manifest_contents(paths)?;
	let mut manifest_contents = download.get_str();
	let manifest = match json::parse_json(&manifest_contents) {
		Ok(manifest) => manifest,
		Err(..) => {
			println!("Failed to parse version manifest. Redownloading...");
			download = get_version_manifest_contents(paths)?;
			manifest_contents = download.get_str();
			json::parse_json(&manifest_contents)?
		}
	};
	Ok((manifest, download))
}

#[derive(Debug, thiserror::Error)]
pub enum VersionJsonError {
	#[error("Version {} does not exist", .0)]
	VersionNotFound(#[from] VersionNotFoundError),
	#[error("Failed to evaluate json file: {}", .0)]
	ParseError(#[from] json::JsonError),
	#[error("{}", .0)]
	VersionManifest(#[from] VersionManifestError),
	#[error("Error when downloading version json:\n{}", .0)]
	Download(#[from] helper::DownloadError)
}

pub fn get_version_json(version: MinecraftVersion, paths: &Paths, verbose: bool)
-> Result<(Box<Value>, Box<Download>), VersionJsonError> {
	let version_string = version.as_string().to_owned();

	let (manifest_doc, mut download) = get_version_manifest(paths, verbose)?;
	let manifest_doc = manifest_doc.as_object().unwrap();
	// Find the version out of all of them
	let versions = json::access_array(manifest_doc, "versions")?;
	let mut version_url: Option<&str> = None;
	for entry in versions.iter() {
		let obj = json::ensure_type(entry.as_object(), "(version)", json::JsonType::Object)?;
		if json::access_str(obj, "id")? == version_string {
			version_url = Some(json::access_str(obj, "url")?);
		}
	}
	if version_url.is_none() {
		return Err(VersionJsonError::from(VersionNotFoundError::new(version)));
	}

	let version_json_name: String = version_string.clone() + ".json";
	let version_folder = paths.internal.join("versions").join(version_string);
	create_existing_dir(&version_folder).expect("Failed to create version folder");
	download.reset();
	download.url(version_url.expect("Version does not exist"))?;
	download.add_file(&version_folder.join(version_json_name))?;
	download.add_str();
	download.perform()?;

	let version_doc = json::parse_json(&download.get_str())?;

	Ok((version_doc, download))
}
