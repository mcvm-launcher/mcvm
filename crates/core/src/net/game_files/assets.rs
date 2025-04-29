use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::versions::VersionPattern;
use mcvm_shared::{translate, try_3, UpdateDepth};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::{sync::Semaphore, task::JoinSet};

use crate::io::files::{self, paths::Paths};
use crate::io::update::{UpdateManager, UpdateMethodResult};
use crate::io::{json_from_file, json_to_file};
use crate::net::download::{self, get_transfer_limit};
use crate::util::versions::VersionName;

use super::client_meta::ClientMeta;

/// Structure for the assets index
#[derive(Deserialize, Serialize)]
pub struct AssetIndex {
	/// The map of asset resource locations to index entries
	pub objects: HashMap<String, IndexEntry>,
}

/// A single asset in the index
#[derive(Deserialize, Serialize)]
pub struct IndexEntry {
	/// The hash of the index file
	pub hash: String,
	/// The size of the asset in bytes
	pub size: usize,
}

impl IndexEntry {
	/// Get the hash path for this asset, which is used for the relative location on the filesystem
	/// and the remote server where they are downloaded
	pub fn get_hash_path(&self) -> String {
		format!("{}/{}", &self.hash[..2], self.hash)
	}
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
				MessageContents::Error(translate!(o, AssetIndexFailed)),
				MessageLevel::Important,
			);
			o.display(
				MessageContents::Error(format!("{}", err)),
				MessageLevel::Important,
			);
			o.display(
				MessageContents::StartProcess(translate!(o, Redownloading)),
				MessageLevel::Important,
			);
			download_index(index_url, &index_path, manager, client, true)
				.await
				.context("Failed to obtain asset index")?
		}
	};

	let mut assets_to_download = Vec::new();
	for (name, asset) in index.objects {
		let hash_path = asset.get_hash_path();
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
		let data = AssetData {
			name,
			url,
			path,
			virtual_path,
			size: asset.size,
		};
		assets_to_download.push(data);
	}
	// Sort downloads by biggest first
	assets_to_download.sort_by_key(|x| std::cmp::Reverse(x.size));

	let count = assets_to_download.len();
	if count > 0 {
		o.display(
			MessageContents::StartProcess(translate!(
				o,
				StartDownloadingAssets,
				"count" = &format!("{count}")
			)),
			MessageLevel::Important,
		);

		o.start_process();
	}

	let mut join = JoinSet::new();
	// Used to limit the number of open file descriptors
	let sem = Arc::new(Semaphore::new(get_transfer_limit()));
	for asset in assets_to_download {
		let client = client.clone();
		let sem = sem.clone();
		let fut = async move {
			let _permit = sem.acquire().await;

			try_3!({ download_asset(&asset, &client).await })
				.context("Failed three times to download asset")?;

			Ok::<String, anyhow::Error>(asset.name)
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
	let mut num_failures = 0;
	while let Some(asset) = join.join_next().await {
		let Ok(name) = asset else {
			num_failures += 1;
			continue;
		};
		let name = match name {
			Ok(name) => name,
			Err(e) => {
				o.display(
					MessageContents::Error(translate!(o, AssetFailed, "error" = &e.to_string())),
					MessageLevel::Important,
				);
				num_failures += 1;
				continue;
			}
		};

		num_done += 1;
		o.display(
			MessageContents::Associated(
				Box::new(MessageContents::Progress {
					current: num_done,
					total: count as u32,
				}),
				Box::new(MessageContents::Simple(translate!(
					o,
					DownloadedAsset,
					"asset" = &name
				))),
			),
			MessageLevel::Important,
		);
	}

	if num_failures > 0 {
		o.display(
			MessageContents::Error(translate!(
				o,
				AssetsFailed,
				"num" = &num_failures.to_string()
			)),
			MessageLevel::Important,
		);
	}

	o.display(
		MessageContents::Success(translate!(o, FinishDownloadingAssets)),
		MessageLevel::Important,
	);
	o.end_process();

	Ok(out)
}

/// Downloads and loads a single asset
async fn download_asset(asset: &AssetData, client: &Client) -> anyhow::Result<()> {
	let response = download::bytes(&asset.url, &client)
		.await
		.context("Failed to download asset")?;

	// Write JSON as minified to save storage space, if there are no errors
	let result = if asset.name.ends_with(".json") {
		if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&response) {
			json_to_file(&asset.path, &json).ok()
		} else {
			None
		}
	} else {
		None
	};

	if result.is_none() {
		tokio::fs::write(&asset.path, response)
			.await
			.context("Failed to write asset to file")?;
	}

	if let Some(virtual_path) = &asset.virtual_path {
		files::update_hardlink_async(&asset.path, virtual_path)
			.await
			.context("Failed to hardlink virtual asset")?;
	}

	Ok(())
}

struct AssetData {
	name: String,
	url: String,
	path: PathBuf,
	virtual_path: Option<PathBuf>,
	size: usize,
}

/// Downloads the asset index which contains all of the assets that need to be downloaded
async fn download_index(
	url: &str,
	path: &Path,
	manager: &UpdateManager,
	client: &Client,
	force: bool,
) -> anyhow::Result<AssetIndex> {
	let index = if manager.update_depth < UpdateDepth::Force && !force && path.exists() {
		json_from_file(path).context("Failed to read asset index contents from file")?
	} else {
		let index = download::json(url, client)
			.await
			.context("Failed to download asset index")?;

		json_to_file(path, &index).context("Failed to serialize asset index to a file")?;

		index
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
