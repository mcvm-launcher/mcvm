use super::lib::{CmdData, CmdError};

use color_print::cprintln;

pub fn help() {
	cprintln!("<i>launch:</i> Launch instances to play the game");
	cprintln!("<s>Usage:</s> mcvm launch <k!><<instance>></k!>");
}

pub fn run(argc: usize, argv: &[String], data: &mut CmdData)
-> Result<(), CmdError> {
	if argc == 0 {
		help();
		return Ok(());
	}

	data.ensure_paths()?;
	data.ensure_config()?;

	if let Some(config) = &mut data.config {
		if let Some(paths) = &data.paths {
			if let Some(instance) = config.instances.get_mut(&argv[0]) {
				instance.launch(&paths, &config.auth)?;
			} else {
				return Err(CmdError::Custom(format!("Unknown instance '{}'", &argv[0])));
			}
		}
	}

	Ok(())
}
