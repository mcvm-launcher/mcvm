/// Downloading game assets
pub mod assets;
/// Structure for the client metadata file
pub mod client_meta;
/// Downloading game Java libraries
pub mod libraries;
/// Downloading and using the version manifest
pub mod version_manifest;

use crate::data::profile::update::manager::UpdateManager;
use crate::io::files::paths::Paths;
use crate::util::cap_first_letter;
use mcvm_shared::instance::Side;

use anyhow::Context;
use reqwest::Client;

use super::download;

/// Downloading the game JAR file
pub mod game_jar {
	use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel, OutputProcess};

	use super::{client_meta::ClientMeta, *};

	/// Downloads the game JAR file
	pub async fn get(
		side: Side,
		client_json: &ClientMeta,
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

		let process = OutputProcess::new(o);
		process.0.display(
			MessageContents::StartProcess(format!("Downloading {side_str} jar")),
			MessageLevel::Important,
		);

		let download = match side {
			Side::Client => &client_json.downloads.client,
			Side::Server => &client_json.downloads.server,
		};

		download::file(&download.url, &path, &Client::new())
			.await
			.context("Failed to download file")?;
		let side_str = cap_first_letter(&side_str);

		process.0.display(
			MessageContents::Success(format!("{side_str} jar downloaded")),
			MessageLevel::Important,
		);

		Ok(())
	}
}
