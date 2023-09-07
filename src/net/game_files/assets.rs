use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	sync::Arc,
};

use anyhow::Context;
use mcvm_shared::{
	output::{MCVMOutput, MessageContents, MessageLevel},
	versions::{VersionInfo, VersionPattern},
};
use reqwest::Client;
use serde::Deserialize;
use tokio::{sync::Semaphore, task::JoinSet};

use crate::{
	data::profile::update::manager::{UpdateManager, UpdateMethodResult},
	io::files::{self, paths::Paths},
	net::download::{self, FD_SENSIBLE_LIMIT},
};

use super::client_meta::ClientMeta;

/// A single asset in the index
#[derive(Deserialize)]
pub struct IndexEntry {
	/// The hash of the index file
	pub hash: String,
}

/// Structure for the assets index
#[derive(Deserialize)]
pub struct AssetIndex {
	/// The map of asset resource locations to index entries
	pub objects: HashMap<String, IndexEntry>,
}

async fn download_index(
	url: &str,
	path: &Path,
	manager: &UpdateManager,
	client: &Client,
	force: bool,
) -> anyhow::Result<AssetIndex> {
	let text = if manager.allow_offline && !force && path.exists() {
		tokio::fs::read_to_string(path)
			.await
			.context("Failed to read index contents from file")?
	} else {
		let text = download::text(url, client)
			.await
			.context("Failed to download index")?;
		tokio::fs::write(path, &text)
			.await
			.context("Failed to write index to a file")?;

		text
	};

	let index = serde_json::from_str(&text).context("Failed to parse index")?;
	Ok(index)
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
	let virtual_dir = if VersionPattern::Before("13w48b".into()).matches_info(version_info) {
		Some(get_virtual_dir_path(paths))
	} else {
		None
	};
	Ok((objects_dir, virtual_dir))
}

/// Download assets used by the client, such as game resources and icons.
pub async fn get(
	client_meta: &ClientMeta,
	paths: &Paths,
	version_info: &VersionInfo,
	manager: &UpdateManager,
	client: &Client,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<UpdateMethodResult> {
	let mut out = UpdateMethodResult::new();
	let version_string = version_info.version.clone();
	let indexes_dir = paths.assets.join("indexes");
	files::create_dir_async(&indexes_dir).await?;

	let index_path = indexes_dir.join(version_string + ".json");
	let index_url = &client_meta.asset_index.url;

	let (objects_dir, virtual_dir) = create_dirs(paths, version_info)
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

		out.files_updated.insert(path.clone());
		files::create_leading_dirs_async(&path).await?;
		if let Some(virtual_path) = &virtual_path {
			files::create_leading_dirs_async(virtual_path).await?;
		}
		assets_to_download.push((name, url, path, virtual_path));
	}

	let count = assets_to_download.len();
	if manager.print.verbose && count > 0 {
		o.display(
			MessageContents::StartProcess(format!("Downloading {count} assets")),
			MessageLevel::Important,
		);

		o.start_process();
	}

	let mut num_done = 0;
	let mut join = JoinSet::new();
	// Used to limit the number of open file descriptors
	let sem = Arc::new(Semaphore::new(FD_SENSIBLE_LIMIT));
	for (name, url, path, virtual_path) in assets_to_download {
		let client = client.clone();
		let permit = sem.clone().acquire_owned().await;
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

		o.display(
			MessageContents::Associated(
				format!("{num_done}/{count}"),
				Box::new(MessageContents::Simple(name)),
			),
			MessageLevel::Important,
		);
	}

	while let Some(asset) = join.join_next().await {
		asset??;
	}

	o.display(
		MessageContents::Success("Assets downloaded".into()),
		MessageLevel::Important,
	);
	o.end_process();

	Ok(out)
}
