/// Client arguments
mod args;

use anyhow::{anyhow, bail, Context};

use std::collections::HashMap;

#[cfg(target_os = "linux")]
use mcvm_shared::versions::VersionPattern;
use mcvm_shared::{output::MCVMOutput, skip_none};

pub use args::create_quick_play_args;

use crate::net::game_files::client_meta::args::Arguments;

use super::{process::LaunchProcessProperties, LaunchParameters};

/// Create launch properties for the client
pub(crate) async fn get_launch_props(
	params: &mut LaunchParameters<'_>,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<LaunchProcessProperties> {
	// Ensure a user is picked
	if !params.users.is_user_chosen() {
		bail!("No user chosen");
	}

	// Ensure the user is authenticated
	params
		.users
		.authenticate(params.paths, params.req_client, o)
		.await
		.context("Failed to authenticate user")?;

	// Build up arguments
	let mut jvm_args = Vec::new();
	let mut game_args = Vec::new();

	if params.launch_config.use_log4j_config {
		let logging_arg = params.client_meta.logging.client.argument.clone();
		let logging_arg = args::fill_logging_path_arg(logging_arg, params.version, params.paths)
			.ok_or(anyhow!("Failed to convert logging path to a string"))?;
		jvm_args.push(logging_arg);
	}

	match &params.client_meta.arguments {
		Arguments::New(args) => {
			for arg in &args.jvm {
				for sub_arg in args::process_arg(arg, params) {
					jvm_args.push(sub_arg);
				}
			}

			for arg in &args.game {
				for sub_arg in args::process_arg(arg, params) {
					game_args.push(sub_arg);
				}
			}
		}
		Arguments::Old(args) => {
			jvm_args.push(format!(
				"-Djava.library.path={}",
				params
					.paths
					.internal
					.join("versions")
					.join(params.version.to_string())
					.join("natives")
					.to_str()
					.context("Failed to convert natives directory to a string")?
			));
			jvm_args.push("-cp".into());
			jvm_args.push(params.classpath.get_str());

			for arg in args.split(' ') {
				game_args.push(skip_none!(args::replace_arg_placeholders(arg, params)));
			}
		}
	}

	let env_vars =
		get_additional_environment_variables(params.version, &params.version_manifest.list);

	let props = LaunchProcessProperties {
		jvm_args,
		game_args,
		additional_env_vars: env_vars,
	};
	Ok(props)
}

/// Get additional environment variables for the client
fn get_additional_environment_variables(
	version: &str,
	version_list: &[String],
) -> HashMap<String, String> {
	#[cfg(not(target_os = "linux"))]
	{
		let _ = version;
		let _ = version_list;
	}

	#[cfg(target_os = "linux")]
	let mut env_vars = HashMap::new();
	#[cfg(not(target_os = "linux"))]
	let env_vars = HashMap::new();

	// Compatability env var for old versions on Linux to prevent graphical issues
	#[cfg(target_os = "linux")]
	{
		if VersionPattern::Before("1.8.9".into()).matches_single(version, version_list) {
			env_vars.insert("__GL_THREADED_OPTIMIZATIONS".to_string(), "0".to_string());
		}
	}

	env_vars
}
