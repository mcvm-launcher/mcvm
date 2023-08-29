use std::{
	collections::HashSet,
	path::{Path, PathBuf},
	sync::Arc,
};

use anyhow::Context;
use color_print::{cformat, cprintln};
use mcvm_shared::versions::{VersionInfo, VersionPattern};
use reqwest::Client;
use tokio::{sync::Semaphore, task::JoinSet};

use crate::{
	data::profile::update::UpdateManager,
	io::files::{self, paths::Paths},
	net::download::{self, FD_SENSIBLE_LIMIT},
	util::{
		json::{self, JsonType},
		print::ReplPrinter,
	},
};

async fn download_index(
	url: &str,
	path: &Path,
	manager: &UpdateManager,
	force: bool,
) -> anyhow::Result<Box<json::JsonObject>> {
	let text = if manager.allow_offline && !force && path.exists() {
		tokio::fs::read_to_string(path)
			.await
			.context("Failed to read index contents from file")?
	} else {
		let text = download::text(url, &Client::new())
			.await
			.context("Failed to download index")?;
		tokio::fs::write(path, &text)
			.await
			.context("Failed to write index to a file")?;

		text
	};

	let doc = json::parse_object(&text).context("Failed to parse index")?;
	Ok(doc)
}

/// Get the virtual assets directory path
pub fn get_virtual_dir_path(paths: &Paths) -> PathBuf {
	paths.assets.join("virtual").join("legacy")
}

/// Create the directories needed to store assets
async fn create_dirs(
	paths: &Paths,
	version_info: &VersionInfo,
) -> anyhow::Result<(PathBuf, Option<PathBuf>)> {
	let objects_dir = paths.assets.join("objects");
	files::create_dir_async(&objects_dir).await?;
	// Apparently this directory name is used for older game versions
	if VersionPattern::Before(String::from("13w48b")).matches_info(version_info) {}
	let virtual_dir = if VersionPattern::Before(String::from("13w48b")).matches_info(version_info) {
		Some(get_virtual_dir_path(paths))
	} else {
		None
	};
	Ok((objects_dir, virtual_dir))
}

/// Download assets used by the client, such as game resources and icons.
pub async fn get(
	client_json: &json::JsonObject,
	paths: &Paths,
	version_info: &VersionInfo,
	manager: &UpdateManager,
) -> anyhow::Result<HashSet<PathBuf>> {
	let mut out = HashSet::new();
	let version_string = version_info.version.clone();
	let indexes_dir = paths.assets.join("indexes");
	files::create_dir_async(&indexes_dir).await?;

	let index_path = indexes_dir.join(version_string + ".json");
	let index_url = json::access_str(json::access_object(client_json, "assetIndex")?, "url")?;

	let (objects_dir, virtual_dir) = create_dirs(paths, version_info)
		.await
		.context("Failed to create directories for assets")?;

	let index = match download_index(index_url, &index_path, manager, false).await {
		Ok(val) => val,
		Err(err) => {
			cprintln!(
				"<r>Failed to obtain asset index:\n{}\nRedownloading...",
				err
			);
			download_index(index_url, &index_path, manager, true)
				.await
				.context("Failed to obtain asset index")?
		}
	};

	let assets = json::access_object(&index, "objects")?.clone();

	let mut assets_to_download = Vec::new();
	for (name, asset) in assets {
		let asset = json::ensure_type(asset.as_object(), JsonType::Obj)?;

		let hash = json::access_str(asset, "hash")?.to_owned();
		let hash_path = format!("{}/{hash}", hash[..2].to_owned());
		let url = format!("https://resources.download.minecraft.net/{hash_path}");

		let path = objects_dir.join(&hash_path);
		let virtual_path = virtual_dir.as_ref().map(|x| x.join(&hash_path));
		if !manager.should_update_file(&path) {
			if let Some(virtual_path) = &virtual_path {
				if !manager.should_update_file(virtual_path) {
					continue;
				}
			} else {
				continue;
			}
		}

		out.insert(path.clone());
		files::create_leading_dirs_async(&path).await?;
		if let Some(virtual_path) = &virtual_path {
			files::create_leading_dirs_async(virtual_path).await?;
		}
		assets_to_download.push((name, url, path, virtual_path));
	}

	let mut printer = ReplPrinter::from_options(manager.print.clone());
	let count = assets_to_download.len();
	if manager.print.verbose && count > 0 {
		cprintln!("Downloading <b>{}</> assets...", count);
	}

	let mut num_done = 0;
	let client = Client::new();
	let mut join = JoinSet::new();
	// Used to limit the number of open file descriptors
	let sem = Arc::new(Semaphore::new(FD_SENSIBLE_LIMIT));
	for (name, url, path, virtual_path) in assets_to_download {
		let client = client.clone();
		let permit = Arc::clone(&sem).acquire_owned().await;
		let fut = async move {
			let response = client.get(url).send();
			let _permit = permit;
			tokio::fs::write(&path, response.await?.error_for_status()?.bytes().await?).await?;
			if let Some(virtual_path) = virtual_path {
				files::update_hardlink(&path, &virtual_path)
					.context("Failed to hardlink virtual asset")?;
			}
			Ok::<(), anyhow::Error>(())
		};
		join.spawn(fut);
		num_done += 1;
		printer.print(&cformat!(
			"(<b>{}</b><k!>/</k!><b>{}</b>) <k!>{}",
			num_done,
			count,
			name
		));
	}

	while let Some(asset) = join.join_next().await {
		asset??;
	}

	printer.print(&cformat!("<g>Assets downloaded."));
	printer.finish();

	Ok(out)
}
