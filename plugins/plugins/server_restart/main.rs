use std::fs::File;
use std::io::{BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::Context;
use mcvm_plugin::api::CustomPlugin;
use mcvm_plugin::hooks::OnInstanceSetupResult;
use mcvm_shared::Side;
use serde::Deserialize;

fn main() -> anyhow::Result<()> {
	let mut plugin =
		CustomPlugin::from_manifest_file("server_restart", include_str!("plugin.json"))?;
	plugin.on_instance_setup(|_, arg| {
		if !arg.side.is_some_and(|x| x == Side::Server) {
			return Ok(OnInstanceSetupResult::default());
		}

		let config = if let Some(config) = arg.custom_config.get("restart") {
			serde_json::from_value(config.clone()).context("Failed to deserialize config")?
		} else {
			Config::default()
		};

		#[cfg(target_os = "windows")]
		let filename = "start.bat";
		#[cfg(not(target_os = "windows"))]
		let filename = "start.sh";
		let path = PathBuf::from(&arg.game_dir).join(filename);
		create_script(&path, &arg.id, config)
			.context("Failed to create startup script for instance")?;

		Ok(OnInstanceSetupResult::default())
	})?;

	Ok(())
}

/// Config for restart behavior on an instance
#[derive(Deserialize, Default)]
struct Config {
	/// Mode to use
	mode: Mode,
}

/// Mode for the restart (what is used to launch)
#[derive(Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum Mode {
	/// Using mcvm_cli
	#[default]
	Cli,
}

/// Create the restart script file at the specified path
fn create_script(path: &Path, inst_ref: &str, config: Config) -> anyhow::Result<()> {
	if !path.exists() {
		let mut file = BufWriter::new(File::create(&path)?);
		#[cfg(target_family = "unix")]
		{
			writeln!(&mut file, "#!/bin/sh")?;
			writeln!(&mut file)?;
		}

		match config.mode {
			Mode::Cli => {
				writeln!(&mut file, "mcvm instance launch {inst_ref}")?;
			}
		}
	}

	// Make executable
	#[cfg(target_family = "unix")]
	{
		let mut perms = std::fs::metadata(&path)
			.context("Failed to get file metadata")?
			.permissions();
		perms.set_mode(0o777);

		std::fs::set_permissions(path, perms).context("Failed to set writable permissions")?;
	}

	Ok(())
}
