use super::CmdData;

use anyhow::{bail, Context};

pub async fn run(instance: &str, debug: bool, data: &mut CmdData) -> anyhow::Result<()> {
	data.ensure_paths()?;
	data.ensure_config()?;

	if let Some(config) = &mut data.config {
		if let Some(paths) = &data.paths {
			if let Some(instance) = config.instances.get_mut(instance) {
				instance
					.launch(paths, &config.auth, debug)
					.await
					.context("Instance failed to launch")?;
			} else {
				bail!("Unknown instance '{}'", instance);
			}
		}
	}

	Ok(())
}
