use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use anyhow::Context;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};

use crate::instance::InstanceKind;
use crate::util::versions::VersionName;

use super::LaunchConfiguration;

/// Launch the game process
pub(crate) fn launch_game_process(
	params: LaunchProcessParameters<'_>,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<std::process::Child> {
	// Create the base command based on wrapper settings
	let mut cmd = match &params.launch_config.wrapper {
		Some(wrapper) => {
			let mut cmd = Command::new(&wrapper.cmd);
			cmd.args(&wrapper.args);
			cmd.arg(params.command);
			cmd
		}
		None => Command::new(params.command),
	};

	// Fill out the command properties
	cmd.current_dir(params.cwd);
	cmd.envs(params.launch_config.env.clone());
	cmd.envs(params.props.additional_env_vars);

	// Add the arguments
	cmd.args(params.launch_config.generate_jvm_args());
	cmd.args(params.props.jvm_args);
	if let Some(main_class) = params.main_class {
		cmd.arg(main_class);
	}
	cmd.args(params.props.game_args);
	cmd.args(params.launch_config.generate_game_args(
		params.version,
		params.version_list,
		params.side.get_side(),
		o,
	));

	output_launch_command(&cmd, params.user_access_token, params.censor_secrets, o)?;

	let child = cmd.spawn().context("Failed to spawn child process")?;

	Ok(child)
}

/// Display the launch command in our own way,
/// censoring any credentials if needed
fn output_launch_command(
	command: &Command,
	access_token: Option<String>,
	censor_secrets: bool,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<()> {
	let access_token = if censor_secrets { access_token } else { None };
	o.display(
		MessageContents::Property(
			"Launch command".into(),
			Box::new(MessageContents::Simple(
				command.get_program().to_string_lossy().into(),
			)),
		),
		MessageLevel::Debug,
	);

	o.display(
		MessageContents::Header("Launch command arguments".into()),
		MessageLevel::Debug,
	);

	const CENSOR_STR: &str = "***";
	for arg in command.get_args() {
		let mut arg = arg.to_string_lossy().to_string();
		if let Some(access_token) = &access_token {
			arg = arg.replace(access_token, CENSOR_STR);
		}
		o.display(
			MessageContents::ListItem(Box::new(MessageContents::Simple(arg))),
			MessageLevel::Debug,
		);
	}

	o.display(
		MessageContents::Header("Launch command environment".into()),
		MessageLevel::Debug,
	);

	for (env, val) in command.get_envs() {
		let Some(val) = val else {continue };
		let env = env.to_string_lossy().to_string();
		let val = val.to_string_lossy().to_string();

		o.display(
			MessageContents::ListItem(Box::new(MessageContents::Property(
				env,
				Box::new(MessageContents::Simple(val)),
			))),
			MessageLevel::Debug,
		);
	}

	if let Some(dir) = command.get_current_dir() {
		o.display(
			MessageContents::Property(
				"Launch command directory".into(),
				Box::new(MessageContents::Simple(dir.to_string_lossy().into())),
			),
			MessageLevel::Debug,
		);
	}

	Ok(())
}

/// Container struct for parameters for launching the game process
pub(crate) struct LaunchProcessParameters<'a> {
	/// The base command to run, usually the path to the JVM
	pub command: &'a OsStr,
	/// The current working directory, usually the instance dir
	pub cwd: &'a Path,
	/// The Java main class to run
	pub main_class: Option<&'a str>,
	pub props: LaunchProcessProperties,
	pub launch_config: &'a LaunchConfiguration,
	pub version: &'a VersionName,
	pub version_list: &'a [String],
	pub side: &'a InstanceKind,
	pub user_access_token: Option<String>,
	pub censor_secrets: bool,
}

/// Properties for launching the game process that are created by
/// the side-specific launch routine
pub(crate) struct LaunchProcessProperties {
	/// Arguments for the JVM
	pub jvm_args: Vec<String>,
	/// Arguments for the game
	pub game_args: Vec<String>,
	/// Additional environment variables to add to the launch command
	pub additional_env_vars: HashMap<String, String>,
}
