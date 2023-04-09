use super::CmdData;

use anyhow::{bail, Context};

pub async fn run(instance: &str, debug: bool, data: &mut CmdData) -> anyhow::Result<()> {
	data.ensure_paths().await?;
	data.ensure_config().await?;

	if let Some(config) = &mut data.config {
		if let Some(paths) = &data.paths {
			if let Some(instance) = config.instances.get_mut(instance) {
				let (.., profile) = config
					.profiles
					.iter()
					.find(|(.., profile)| profile.instances.contains(&instance.id))
					.expect("Instance does not belong to any profiles");
				instance
					.launch(paths, &config.auth, debug, &profile.version)
					.await
					.context("Instance failed to launch")?;
			} else {
				bail!("Unknown instance '{}'", instance);
			}
		}
	}

	Ok(())
}
