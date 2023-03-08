use crate::net::mojang::get_version_manifest;

use super::lib::{CmdData, CmdError};

use color_print::cprintln;

pub fn help() {
	cprintln!("<i>launch:</i> Launch instances to play the game");
	cprintln!("<s>Usage:</s> mcvm launch <k!><<instance>></k!>");
}

pub async fn run(argc: usize, argv: &[String], data: &mut CmdData) -> Result<(), CmdError> {
	if argc == 0 {
		help();
		return Ok(());
	}

	data.ensure_paths()?;
	data.ensure_config()?;

	if let Some(config) = &mut data.config {
		if let Some(paths) = &data.paths {
			if let Some(instance) = config.instances.get_mut(&argv[0]) {
				let (version_manifest, ..) = get_version_manifest(paths)?;
				instance
					.launch(&version_manifest, paths, &config.auth)
					.await?;
			} else {
				return Err(CmdError::Custom(format!("Unknown instance '{}'", &argv[0])));
			}
		}
	}

	Ok(())
}
