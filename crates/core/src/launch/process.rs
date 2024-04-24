use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{Child, Command};

use anyhow::Context;
use mcvm_auth::mc::AccessToken;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};

use crate::instance::InstanceKind;
use crate::util::versions::VersionName;
use crate::WrapperCommand;

use super::LaunchConfiguration;

/// Launch the game process
pub(crate) fn launch_game_process(
	mut params: LaunchGameProcessParameters<'_>,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<std::process::Child> {
	// Modify the parameters based on game-specific properties

	// Prepend generated game args to the beginning
	let previous_game_args = params.props.game_args.clone();
	params.props.game_args = params.launch_config.generate_game_args(
		params.version,
		params.version_list,
		params.side.get_side(),
		o,
	);
	params.props.game_args.extend(previous_game_args);

	// Create the parameters for the process
	let proc_params = LaunchProcessParameters {
		command: params.command,
		cwd: params.cwd,
		main_class: params.main_class,
		props: params.props,
		launch_config: params.launch_config,
	};

	// Get the command and output it
	let mut cmd = get_process_launch_command(proc_params)
		.context("Failed to create process launch command")?;

	output_launch_command(&cmd, params.user_access_token, params.censor_secrets, o)?;

	// Spawn
	let child = cmd.spawn().context("Failed to spawn child process")?;

	Ok(child)
}

/// Launch a generic process with the core's config system
pub fn launch_process(params: LaunchProcessParameters<'_>) -> anyhow::Result<Child> {
	let mut cmd =
		get_process_launch_command(params).context("Failed to create process launch command")?;

	cmd.spawn().context("Failed to spawn child process")
}

/// Get the command for launching a generic process using the core's config system
pub fn get_process_launch_command(params: LaunchProcessParameters<'_>) -> anyhow::Result<Command> {
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

	Ok(cmd)
}

/// Display the launch command in our own way,
/// censoring any credentials if needed
fn output_launch_command(
	command: &Command,
	access_token: Option<&AccessToken>,
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
pub(crate) struct LaunchGameProcessParameters<'a> {
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
	pub user_access_token: Option<&'a AccessToken>,
	pub censor_secrets: bool,
}

/// Container struct for parameters for launching a generic Java process
pub struct LaunchProcessParameters<'a> {
	/// The base command to run, usually the path to the JVM
	pub command: &'a OsStr,
	/// The current working directory, usually the instance dir
	pub cwd: &'a Path,
	/// The Java main class to run
	pub main_class: Option<&'a str>,
	/// Properties for launching
	pub props: LaunchProcessProperties,
	/// The launch configuration
	pub launch_config: &'a LaunchConfiguration,
}

/// Properties for launching the game process that are created by
/// the side-specific launch routine
#[derive(Default)]
pub struct LaunchProcessProperties {
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
