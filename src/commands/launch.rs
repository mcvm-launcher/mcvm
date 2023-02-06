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

	data.config.load()?;
	if let Some(config) = &mut data.config.data {
		if let Some(instance) = config.instances.get_mut(&argv[0]) {
			instance.launch(&data.paths, &config.auth)?;
		} else {
			return Err(CmdError::Custom(format!("Unknown instance '{}'", &argv[0])));
		}
	}

	Ok(())
}
