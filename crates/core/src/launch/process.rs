use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use anyhow::Context;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};

use crate::instance::InstanceKind;
use crate::user::auth::AccessToken;
use crate::util::versions::VersionName;
use crate::WrapperCommand;

use super::LaunchConfiguration;

/// Launch the game process
pub(crate) fn launch_game_process(
	params: LaunchProcessParameters<'_>,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<std::process::Child> {
	// Create the base command based on wrapper settings
	let mut cmd = create_wrapped_command(params.command, &params.launch_config.wrappers);

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
	access_token: Option<AccessToken>,
	censor_secrets: bool,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<()> {
	o.end_process();
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
			arg = arg.replace(&access_token.0, CENSOR_STR);
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
		let Some(val) = val else { continue };
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

/// Creates a command wrapped in multiple other wrappers
fn create_wrapped_command(command: &OsStr, wrappers: &[WrapperCommand]) -> Command {
	let mut cmd = Command::new(command);
	for wrapper in wrappers {
		cmd = wrap_single(cmd, wrapper);
	}
	cmd
}

/// Wraps a single command in a wrapper
fn wrap_single(command: Command, wrapper: &WrapperCommand) -> Command {
	let mut new_cmd = Command::new(&wrapper.cmd);
	new_cmd.args(&wrapper.args);
	new_cmd.arg(command.get_program());
	new_cmd.args(command.get_args());
	new_cmd
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
	pub user_access_token: Option<AccessToken>,
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_wrappers() {
		let wrappers = vec![
			WrapperCommand {
				cmd: "hello".into(),
				args: Vec::new(),
			},
			WrapperCommand {
				cmd: "world".into(),
				args: vec!["foo".into(), "bar".into()],
			},
		];
		let cmd = create_wrapped_command(OsStr::new("run"), &wrappers);
		dbg!(&cmd);
		assert_eq!(cmd.get_program(), OsStr::new("world"));
		let mut args = cmd.get_args();
		assert_eq!(args.next(), Some(OsStr::new("foo")));
		assert_eq!(args.next(), Some(OsStr::new("bar")));
		assert_eq!(args.next(), Some(OsStr::new("hello")));
		assert_eq!(args.next(), Some(OsStr::new("run")));
	}
}
