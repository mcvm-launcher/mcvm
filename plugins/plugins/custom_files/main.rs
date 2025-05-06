use std::path::PathBuf;

use anyhow::Context;
use mcvm_core::io::files::{create_leading_dirs, update_hardlink};
use mcvm_plugin::api::CustomPlugin;
use mcvm_plugin::hooks::OnInstanceSetupResult;
use serde::{Deserialize, Serialize};

fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::from_manifest_file("custom_files", include_str!("plugin.json"))?;
	plugin.on_instance_setup(|_, args| {
		let Some(config) = args.config.common.plugin_config.get("custom_files") else {
			return Ok(OnInstanceSetupResult::default());
		};
		let config: Config = serde_json::from_value(config.clone())
			.context("Configuration is incorrectly formatted")?;

		let game_dir = PathBuf::from(args.game_dir);

		// Copy all of the files
		for file in config.files {
			let src = PathBuf::from(shellexpand::tilde(&file.source).to_string());
			let target = game_dir.join(PathBuf::from(file.target));

			create_leading_dirs(&target).context("Failed to create leading directories to file")?;

			if file.link {
				update_hardlink(&src, &target)
					.with_context(|| format!("Failed to link custom file {}", file.source))?;
			} else {
				std::fs::copy(src, target)
					.with_context(|| format!("Failed to link custom file {}", file.source))?;
			}
		}

		Ok(OnInstanceSetupResult::default())
	})?;

	Ok(())
}

#[derive(Serialize, Deserialize)]
struct Config {
	files: Vec<File>,
}

#[derive(Serialize, Deserialize)]
struct File {
	source: String,
	target: String,
	link: bool,
}
