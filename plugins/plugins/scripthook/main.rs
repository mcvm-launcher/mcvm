use anyhow::{bail, Context};
use mcvm_plugin::api::CustomPlugin;
fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::from_manifest_file("scripthook", include_str!("plugin.json"))?;

	plugin.on_instance_launch(|_, arg| {
		if let Some(cmd) = arg.config.common.plugin_config.get("on_launch") {
			let cmd: String =
				serde_json::from_value(cmd.clone()).context("Invalid command format")?;

			run_hook(&cmd).context("Failed to run script")?;
		}

		Ok(())
	})?;

	plugin.on_instance_stop(|_, arg| {
		if let Some(cmd) = arg.config.common.plugin_config.get("on_stop") {
			let cmd: String =
				serde_json::from_value(cmd.clone()).context("Invalid command format")?;

			run_hook(&cmd).context("Failed to run script")?;
		}

		Ok(())
	})?;

	Ok(())
}

fn run_hook(cmd: &str) -> anyhow::Result<()> {
	#[cfg(target_family = "unix")]
	{
		let shell = std::env::var("SHELL").unwrap_or("/bin/sh".into());

		let mut command = std::process::Command::new(shell);
		command.arg("-c");
		command.arg(cmd);

		let success = command.spawn()?.wait()?.success();
		if !success {
			bail!("Command returned a non-zero exit code");
		}
	}

	Ok(())
}
