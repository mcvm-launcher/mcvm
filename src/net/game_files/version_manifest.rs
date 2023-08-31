use anyhow::{bail, Context};
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use reqwest::Client;

use crate::{
	data::profile::update::manager::UpdateManager,
	io::files::{self, paths::Paths},
	net::download,
	util::json::{self, JsonType},
};

/// Obtain the version manifest contents
async fn get_contents(
	paths: &Paths,
	manager: &UpdateManager,
	force: bool,
) -> anyhow::Result<String> {
	let mut path = paths.internal.join("versions");
	files::create_dir_async(&path).await?;
	path.push("manifest.json");

	if manager.allow_offline && !force && path.exists() {
		return tokio::fs::read_to_string(path)
			.await
			.context("Failed to read manifest contents from file");
	}

	let text = download::text(
		"https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
		&Client::new(),
	)
	.await
	.context("Failed to download manifest")?;
	tokio::fs::write(&path, &text)
		.await
		.context("Failed to write manifest to a file")?;

	Ok(text)
}

/// Get the version manifest as a JSON object
pub async fn get(
	paths: &Paths,
	manager: &UpdateManager,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<Box<json::JsonObject>> {
	let mut manifest_contents = get_contents(paths, manager, false)
		.await
		.context("Failed to get manifest contents")?;
	let manifest = match json::parse_object(&manifest_contents) {
		Ok(manifest) => manifest,
		Err(err) => {
			o.display(
				MessageContents::Error("Failed to obtain version manifest".to_string()),
				MessageLevel::Important,
			);
			o.display(
				MessageContents::Error(format!("{}", err)),
				MessageLevel::Important,
			);
			o.display(
				MessageContents::StartProcess("Redownloading".to_string()),
				MessageLevel::Important,
			);
			manifest_contents = get_contents(paths, manager, true)
				.await
				.context("Failed to donwload manifest contents")?;
			json::parse_object(&manifest_contents)?
		}
	};
	Ok(manifest)
}

/// Make an ordered list of versions from the manifest to use for matching
pub fn make_version_list(version_manifest: &json::JsonObject) -> anyhow::Result<Vec<String>> {
	let versions = json::access_array(version_manifest, "versions")?;
	let mut out = Vec::new();
	for entry in versions {
		let entry_obj = json::ensure_type(entry.as_object(), JsonType::Obj)?;
		out.push(json::access_str(entry_obj, "id")?.to_owned());
	}
	out.reverse();
	Ok(out)
}

/// Gets the specific client info JSON file for a Minecraft version
pub async fn get_client_json(
	version: &str,
	version_manifest: &json::JsonObject,
	paths: &Paths,
	manager: &UpdateManager,
) -> anyhow::Result<Box<json::JsonObject>> {
	let version_string = version.to_owned();

	let versions = json::access_array(version_manifest, "versions")?;
	let mut version_url: Option<&str> = None;
	for entry in versions.iter() {
		let entry = json::ensure_type(entry.as_object(), JsonType::Obj)?;
		if json::access_str(entry, "id")? == version_string {
			version_url = Some(json::access_str(entry, "url")?);
		}
	}
	if version_url.is_none() {
		bail!("Minecraft version does not exist or was not found in the manifest");
	}

	let client_json_name: String = version_string.clone() + ".json";
	let version_dir = paths.internal.join("versions").join(version_string);
	files::create_dir_async(&version_dir).await?;
	let path = version_dir.join(client_json_name);
	let text = if manager.allow_offline && path.exists() {
		tokio::fs::read_to_string(path)
			.await
			.context("Failed to read client JSON from file")?
	} else {
		let text = download::text(version_url.expect("Version does not exist"), &Client::new())
			.await
			.context("Failed to download client JSON")?;
		tokio::fs::write(path, &text)
			.await
			.context("Failed to write client JSON to a file")?;

		text
	};

	let version_doc = json::parse_object(&text).context("Failed to parse client JSON")?;

	Ok(version_doc)
}
