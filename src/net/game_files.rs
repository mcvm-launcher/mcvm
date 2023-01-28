use crate::Paths;
use crate::io::files::files::create_existing_dir;
use crate::lib::versions::VersionNotFoundError;

use curl::easy::Easy;
use serde_json::Value;
use color_print::cprintln;

use std::io::Write;
use std::fs;

// So we can do this without a retry
fn get_version_manifest_contents(paths: &Paths) -> (String, Box<Easy>) {
	let path = paths.internal.join("manifest.json");
	let mut file = fs::File::create(path).unwrap();
	let mut easy = Easy::new();
	let mut write_vec = Vec::new();
	// TODO: Retries
	easy.url("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json").unwrap();
	{
		let mut transfer = easy.transfer();
		transfer.write_function(|data| {
			file.write_all(data).unwrap();
			write_vec.extend_from_slice(data);
			Ok(data.len())
		}).unwrap();
		transfer.perform().unwrap();
	}
	let contents = String::from_utf8(write_vec)
		.expect("Failed to convert version manifest to a UTF-8 string");
	(contents, Box::new(easy))
}

pub fn get_version_manifest(paths: &Paths, verbose: bool) -> (Box<Value>, Box<Easy>) {
	create_existing_dir(&paths.internal.join("versions")).unwrap();
	
	if verbose {
		println!("\tObtaining version index...");
	}

	let (manifest_contents, easy) = get_version_manifest_contents(paths);
	let manifest = match serde_json::from_str(&manifest_contents) {
		Ok(manifest) => manifest,
		Err(..) => {
			println!("Failed to parse version manifest. Redownloading...");
			let (manifest_contents, _) = get_version_manifest_contents(paths);
			serde_json::from_str(&manifest_contents).expect("Failed to parse manifest a second time")
		}
	};
	(manifest, easy)
}

pub fn get_version_json(version: &str, paths: &Paths, verbose: bool) -> Result<(Box<Value>, Box<Easy>), VersionNotFoundError> {
	let (manifest_doc, mut easy) = get_version_manifest(paths, verbose);
	let manifest_doc = manifest_doc.as_object().unwrap();
	// Find the version out of all of them
	let versions = manifest_doc.get("versions")
		.expect("Missing key [versions] in version manifest").as_array()
		.expect("Key [versions] in version manifest was expected to be an array");
	let mut version_url: Option<&str> = None;
	for version in versions.iter() {
		let obj = version.as_object().expect("Expected version to be an object");
		if obj.get("id").expect("Missing key [id] in version").as_str()
			.expect("Key [id] in version was expected to be a string") == version
		{
			version_url = Some(obj.get("url").expect("Missing key [url] in version").as_str()
				.expect("Key [url] in version was expected to be a string"));
		}
	}
	if version_url == None {
		return Err(VersionNotFoundError::new(version));
	}

	let version_json_name: String = version.to_owned() + ".json";
	let version_folder = paths.internal.join("versions").join(version);
	create_existing_dir(&version_folder).expect("Failed to create version folder");
	let mut file: Option<fs::File> = match fs::File::create(version_folder.join(&version_json_name)) {
		Ok(file) => Some(file),
		Err(err) => {
			cprintln!("<y>Warning: Failed to open {}: {}", version_json_name, err);
			None
		}
	};
	easy.url(version_url.expect("Version does not exist")).expect("URL is invalid");
	let mut write_vec = Vec::new();
	{
		let mut transfer = easy.transfer();
		transfer.write_function(|data| {
			match &mut file {
				Some(file) => file.write_all(data).unwrap(),
				None => {}
			}
			write_vec.extend_from_slice(data);
			Ok(data.len())
		}).unwrap();
		transfer.perform().unwrap();
	}
	let contents = String::from_utf8(write_vec)
		.expect("Failed to convert version manifest to a UTF-8 string");

	let version_doc: Value = serde_json::from_str(&contents)
		.expect("Failed to parse {version_json_name}");

	Ok((Box::new(version_doc), easy))
}
