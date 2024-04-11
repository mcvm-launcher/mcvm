use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::versions::VersionPattern;
use reqwest::Client;
use serde::Deserialize;
use tokio::{sync::Semaphore, task::JoinSet};

use crate::io::files::{self, paths::Paths};
use crate::io::json_from_file;
use crate::io::update::{UpdateManager, UpdateMethodResult};
use crate::net::download::{self, get_transfer_limit};
use crate::util::versions::VersionName;

use super::client_meta::ClientMeta;

/// Structure for the assets index
#[derive(Deserialize)]
pub struct AssetIndex {
	/// The map of asset resource locations to index entries
	pub objects: HashMap<String, IndexEntry>,
}

/// A single asset in the index
#[derive(Deserialize)]
pub struct IndexEntry {
	/// The hash of the index file
	pub hash: String,
	/// The size of the asset in bytes
	pub size: usize,
}

/// Download assets used by the client, such as game resources and icons.
pub async fn get(
	client_meta: &ClientMeta,
	paths: &Paths,
	version: &VersionName,
	version_list: &[String],
	manager: &UpdateManager,
	client: &Client,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<UpdateMethodResult> {
	let mut out = UpdateMethodResult::new();
	let version_string = version.to_string();
	let indexes_dir = paths.assets.join("indexes");
	files::create_dir(&indexes_dir)?;

	let index_path = indexes_dir.join(version_string + ".json");
	let index_url = &client_meta.asset_index.url;

	let (objects_dir, virtual_dir) = create_dirs(paths, version, version_list)
		.await
		.context("Failed to create directories for assets")?;

	let index = match download_index(index_url, &index_path, manager, client, false).await {
		Ok(val) => val,
		Err(err) => {
			o.display(
				MessageContents::Error("Failed to obtain asset index".into()),
				MessageLevel::Important,
			);
			o.display(
				MessageContents::Error(format!("{}", err)),
				MessageLevel::Important,
			);
			o.display(
				MessageContents::StartProcess("Redownloading".into()),
				MessageLevel::Important,
			);
			download_index(index_url, &index_path, manager, client, true)
				.await
				.context("Failed to obtain asset index")?
		}
	};

	let mut assets_to_download = Vec::new();
	for (name, asset) in index.objects {
		let hash = asset.hash;
		let hash_path = format!("{}/{hash}", &hash[..2]);
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

		out.files_updated.insert(path.clone());
		files::create_leading_dirs(&path)?;
		if let Some(virtual_path) = &virtual_path {
			files::create_leading_dirs(virtual_path)?;
		}
		assets_to_download.push((name, url, path, virtual_path, asset.size));
	}
	// Sort downloads by biggest first
	assets_to_download.sort_by_key(|x| std::cmp::Reverse(x.4));

	let count = assets_to_download.len();
	if count > 0 {
		o.display(
			MessageContents::StartProcess(format!("Downloading {count} assets")),
			MessageLevel::Important,
		);

		o.start_process();
	}

	let mut join = JoinSet::new();
	// Used to limit the number of open file descriptors
	let sem = Arc::new(Semaphore::new(get_transfer_limit()));
	for (name, url, path, virtual_path, _) in assets_to_download {
		let client = client.clone();
		let sem = sem.clone();
		let fut = async move {
			let _permit = sem.acquire().await;
			let response = download::bytes(url, &client)
				.await
				.context("Failed to download asset")?;

			tokio::fs::write(&path, response)
				.await
				.context("Failed to write asset to file")?;

			if let Some(virtual_path) = virtual_path {
				files::update_hardlink_async(&path, &virtual_path)
					.await
					.context("Failed to hardlink virtual asset")?;
			}
			Ok::<String, anyhow::Error>(name)
		};
		join.spawn(fut);
	}

	if count > 0 {
		o.display(
			MessageContents::Associated(
				Box::new(MessageContents::Progress {
					current: 0,
					total: count as u32,
				}),
				Box::new(MessageContents::Simple(String::new())),
			),
			MessageLevel::Important,
		);
	}
	let mut num_done = 0;
	while let Some(asset) = join.join_next().await {
		let name = asset??;
		num_done += 1;
		o.display(
			MessageContents::Associated(
				Box::new(MessageContents::Progress {
					current: num_done,
					total: count as u32,
				}),
				Box::new(MessageContents::Simple(format!("Downloaded asset {name}"))),
			),
			MessageLevel::Important,
		);
	}

	o.display(
		MessageContents::Success("Assets downloaded".into()),
		MessageLevel::Important,
	);
	o.end_process();

	Ok(out)
}

async fn download_index(
	url: &str,
	path: &Path,
	manager: &UpdateManager,
	client: &Client,
	force: bool,
) -> anyhow::Result<AssetIndex> {
	let index = if manager.allow_offline && !force && path.exists() {
		json_from_file(path).context("Failed to read asset index contents from file")?
	} else {
		let bytes = download::bytes(url, client)
			.await
			.context("Failed to download asset index")?;
		let out = serde_json::from_slice(&bytes).context("Failed to parse asset index")?;

		std::fs::write(path, &bytes).context("Failed to write asset index to a file")?;

		out
	};

	Ok(index)
}

/// Create the directories needed to store assets
async fn create_dirs(
	paths: &Paths,
	version: &VersionName,
	version_list: &[String],
) -> anyhow::Result<(PathBuf, Option<PathBuf>)> {
	let objects_dir = paths.assets.join("objects");
	files::create_dir(&objects_dir)?;
	// Apparently this directory name is used for older game versions
	let virtual_dir =
		if VersionPattern::Before("13w48b".into()).matches_single(version, version_list) {
			Some(get_virtual_dir_path(paths))
		} else {
			None
		};
	Ok((objects_dir, virtual_dir))
}

/// Get the virtual assets directory path
pub fn get_virtual_dir_path(paths: &Paths) -> PathBuf {
	paths.assets.join("virtual").join("legacy")
}
