use std::{collections::HashMap, path::PathBuf};

use anyhow::Context;
use mcvm_core::Paths;
use mcvm_options::{
	client::write_options_txt, read_options, server::write_server_properties, Options,
};
use mcvm_plugin::{
	api::{CustomPlugin, HookContext},
	hooks::{Hook, OnInstanceSetupResult},
};
use mcvm_shared::Side;

fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::new("options")?;
	plugin.on_instance_setup(|ctx, arg| {
		// Consolidate the options from all the sources
		let mut keys = HashMap::new();
		if let Some(global_options) = get_global_options(&ctx)? {
			match arg.side.unwrap() {
				Side::Client => {
					if let Some(global_options) = &global_options.client {
						let global_keys =
							mcvm_options::client::create_keys(global_options, &arg.version_info)
								.context("Failed to create keys for global options")?;
						keys.extend(global_keys);
					}
				}
				Side::Server => {
					if let Some(global_options) = &global_options.server {
						let global_keys =
							mcvm_options::server::create_keys(global_options, &arg.version_info)
								.context("Failed to create keys for global options")?;
						keys.extend(global_keys);
					}
				}
			}
		}
		// Instance-specific
		if let Some(options) = arg.custom_config.get("options") {
			// Allow profiles to specify both client and server options
			let override_keys =
				if let Ok(options) = serde_json::from_value::<Options>(options.clone()) {
					match arg.side.unwrap() {
						Side::Client => mcvm_options::client::create_keys(
							&options.client.unwrap_or_default(),
							&arg.version_info,
						)
						.context("Failed to create keys for override options")?,
						Side::Server => mcvm_options::server::create_keys(
							&options.server.unwrap_or_default(),
							&arg.version_info,
						)
						.context("Failed to create keys for override options")?,
					}
				} else {
					match arg.side.unwrap() {
						Side::Client => {
							let options = serde_json::from_value(options.clone())?;
							mcvm_options::client::create_keys(&options, &arg.version_info)
								.context("Failed to create keys for override options")?
						}
						Side::Server => {
							let options = serde_json::from_value(options.clone())?;
							mcvm_options::server::create_keys(&options, &arg.version_info)
								.context("Failed to create keys for override options")?
						}
					}
				};

			keys.extend(override_keys);
		}

		// Write the options
		if !keys.is_empty() {
			match arg.side.unwrap() {
				Side::Client => {
					let options_path = PathBuf::from(arg.game_dir).join("options.txt");
					let paths = Paths::new()?;
					let data_version =
						mcvm_core::io::minecraft::get_data_version(&arg.version_info, &paths);
					write_options_txt(keys, &options_path, &data_version)
						.context("Failed to write options.txt")?;
				}
				Side::Server => {
					let options_path = PathBuf::from(arg.game_dir).join("server.properties");
					write_server_properties(keys, &options_path)
						.context("Failed to write server.properties")?;
				}
			}
		}

		Ok(OnInstanceSetupResult::default())
	})?;

	Ok(())
}

fn get_global_options<H: Hook>(ctx: &HookContext<'_, H>) -> anyhow::Result<Option<Options>> {
	let config_file = ctx.get_config_dir()?.join("options.json");
	read_options(&config_file)
}
