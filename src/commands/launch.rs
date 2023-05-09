use super::CmdData;

use anyhow::{bail, Context};

pub async fn run(
	instance: &str,
	debug: bool,
	token: Option<String>,
	data: &mut CmdData,
) -> anyhow::Result<()> {
	data.ensure_paths().await?;
	data.ensure_config().await?;
	let paths = data.paths.get();
	let config = data.config.get_mut();

	if let Some(instance) = config.instances.get_mut(instance) {
		let (.., profile) = config
			.profiles
			.iter()
			.find(|(.., profile)| profile.instances.contains(&instance.id))
			.expect("Instance does not belong to any profiles");
		instance
			.launch(paths, &config.auth, debug, token, &profile.version)
			.await
			.context("Instance failed to launch")?;
	} else {
		bail!("Unknown instance '{}'", instance);
	}

	Ok(())
}
