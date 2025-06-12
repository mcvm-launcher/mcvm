use std::{collections::HashSet, path::Path};

use anyhow::Context;
use mcvm_core::io::{files::create_leading_dirs, json_from_file, json_to_file};
use mcvm_net::{
	download::{self, Client},
	smithed::{self, Pack},
};
use mcvm_pkg_gen::relation_substitution::RelationSubMethod;
use mcvm_plugin::{api::CustomPlugin, hooks::CustomRepoQueryResult};
use mcvm_shared::pkg::PackageSearchResults;
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;

fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::from_manifest_file("smithed", include_str!("plugin.json"))?;

	plugin.query_custom_package_repository(|ctx, arg| {
		if arg.repository != "smithed" {
			return Ok(None);
		}

		let runtime = tokio::runtime::Runtime::new()?;
		let client = Client::new();

		let data_dir = ctx.get_data_dir()?;

		runtime.block_on(query_package(&arg.package, &client, &data_dir))
	})?;

	plugin.preload_packages(|ctx, arg| {
		if arg.repository != "smithed" {
			return Ok(());
		}

		let runtime = tokio::runtime::Runtime::new()?;
		let client = Client::new();

		let data_dir = ctx.get_data_dir()?;

		runtime.block_on(async move {
			let mut tasks = tokio::task::JoinSet::new();
			for package in arg.packages {
				let client = client.clone();
				let data_dir = data_dir.clone();

				tasks.spawn(async move { query_package(&package, &client, &data_dir).await });
			}

			while let Some(task) = tasks.join_next().await {
				let _ = task??;
			}

			Ok::<(), anyhow::Error>(())
		})?;

		Ok(())
	})?;

	plugin.search_custom_package_repository(|ctx, arg| {
		if arg.repository != "smithed" {
			return Ok(PackageSearchResults::default());
		}

		let client = Client::new();
		let runtime = tokio::runtime::Runtime::new()?;
		let storage_dir = ctx
			.get_data_dir()
			.context("Failed to get data dir")?
			.join("internal/smithed/packs");

		let (packs, total_results) = runtime.block_on(async move {
			let search_task = {
				let client = client.clone();
				let params = arg.parameters.clone();
				async move { smithed::search_packs(params, &client).await }
			};
			let count_task = {
				let client = client.clone();
				let params = arg.parameters.clone();
				async move { smithed::count_packs(params, &client).await }
			};

			let (results, total_count) = tokio::try_join!(search_task, count_task)?;

			let mut tasks = JoinSet::new();
			for pack in results {
				let client = client.clone();
				let storage_dir = storage_dir.clone();
				tasks.spawn(async move {
					let pack_info = get_cached_pack(&pack.id, true, &storage_dir, &client)
						.await
						.context("Failed to get cached pack")?
						.context("Pack does not exist")?;

					Ok::<_, anyhow::Error>(pack_info.pack.id)
				});
			}

			let mut packs = Vec::new();
			while let Some(result) = tasks.join_next().await {
				packs.push(result??);
			}

			Ok::<_, anyhow::Error>((packs, total_count))
		})?;

		Ok(PackageSearchResults {
			results: packs,
			total_results,
		})
	})?;

	Ok(())
}

/// Queries for a Smithed package
async fn query_package(
	id: &str,
	client: &Client,
	data_dir: &Path,
) -> anyhow::Result<Option<CustomRepoQueryResult>> {
	let storage_dir = data_dir.join("internal/smithed/packs");
	let pack_info = get_cached_pack(id, true, &storage_dir, &client)
		.await
		.context("Failed to get pack")?;
	let Some(pack_info) = pack_info else {
		return Ok(None);
	};

	let relation_sub_function = {
		let client = client.clone();
		let storage_dir = storage_dir.clone();

		async move |relation: &str| {
			let pack_info = get_cached_pack(relation, false, &storage_dir, &client)
				.await
				.context("Failed to get pack")?
				.context("Pack does not exist")?;

			Ok(pack_info.pack.id)
		}
	};

	let package = mcvm_pkg_gen::smithed::gen(
		pack_info.pack,
		pack_info.body,
		RelationSubMethod::Function(relation_sub_function),
		&[],
	)
	.await
	.context("Failed to generate MCVM package")?;
	let package = serde_json::to_string_pretty(&package).context("Failed to serialized package")?;

	Ok(Some(CustomRepoQueryResult {
		contents: package,
		content_type: mcvm::pkg_crate::PackageContentType::Declarative,
		flags: HashSet::new(),
	}))
}

/// Gets a cached Smithed pack or downloads it
async fn get_cached_pack(
	pack: &str,
	download_body: bool,
	storage_dir: &Path,
	client: &Client,
) -> anyhow::Result<Option<PackInfo>> {
	let pack_path = storage_dir.join(pack);
	if pack_path.exists() {
		let mut pack_info: PackInfo =
			json_from_file(&pack_path).context("Failed to read pack info from file")?;

		if download_body {
			if pack_info.body_exists && pack_info.body.is_none() {
				if let Some(body) = &pack_info.pack.display.web_page {
					if let Ok(text) = download::text(body, client).await {
						pack_info.body = Some(text);
						let _ = json_to_file(&pack_path, &pack_info);
					}
				}
			}
		}

		Ok(Some(pack_info))
	} else {
		let result = smithed::get_pack_optional(pack, &client).await?;

		let pack = match result {
			Some(result) => result,
			None => return Ok(None),
		};

		let body = if download_body {
			if let Some(url) = &pack.display.web_page {
				download::text(url, client).await.ok()
			} else {
				None
			}
		} else {
			None
		};

		let pack_info = PackInfo {
			body_exists: pack.display.web_page.is_some(),
			pack,
			body,
		};

		let _ = create_leading_dirs(&pack_path);
		// TODO: Store both the id and slug together, hardlinked to each other, to cache no matter which method is used to request
		let _ = json_to_file(&pack_path, &pack_info);

		Ok(Some(pack_info))
	}
}

#[derive(Serialize, Deserialize)]
struct PackInfo {
	pack: Pack,
	body: Option<String>,
	body_exists: bool,
}
