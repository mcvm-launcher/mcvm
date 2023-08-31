/// Downloading game assets
pub mod assets;
/// Downloading game Java libraries
pub mod libraries;
/// Downloading and using the version manifest and version JSONs
pub mod version_manifest;

use crate::data::profile::update::UpdateManager;
use crate::io::files::paths::Paths;
use crate::util::cap_first_letter;
use crate::util::json;
use mcvm_shared::instance::Side;

use anyhow::Context;
use reqwest::Client;

use super::download;

/// Downloading the game JAR file
pub mod game_jar {
	use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};

	use super::*;

	/// Downloads the game JAR file
	pub async fn get(
		side: Side,
		client_json: &json::JsonObject,
		version: &str,
		paths: &Paths,
		manager: &UpdateManager,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		let side_str = side.to_string();
		let path = crate::io::minecraft::game_jar::get_path(side, version, paths);
		if !manager.should_update_file(&path) {
			return Ok(());
		}

		o.start_process();
		o.display(
			MessageContents::StartProcess(format!("Downloading {side_str} jar")),
			MessageLevel::Important,
		);

		let download =
			json::access_object(json::access_object(client_json, "downloads")?, &side_str)?;
		let url = json::access_str(download, "url")?;
		download::file(url, &path, &Client::new())
			.await
			.context("Failed to download file")?;
		let side_str = cap_first_letter(&side_str);

		o.display(
			MessageContents::Success(format!("{side_str} jar downloaded")),
			MessageLevel::Important,
		);
		o.end_process();

		Ok(())
	}
}
