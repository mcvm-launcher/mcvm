use std::collections::HashMap;

use super::{process::LaunchProcessProperties, LaunchParameters};

/// Create launch properties for the server
pub(crate) fn get_launch_props(params: &LaunchParameters) -> anyhow::Result<LaunchProcessProperties> {
	let mut jvm_args = Vec::new();
	let mut game_args = Vec::new();

	jvm_args.push("-cp".into());
	jvm_args.push(params.classpath.get_str());
	game_args.push("nogui".into());

	let props = LaunchProcessProperties {
		jvm_args,
		game_args,
		additional_env_vars: HashMap::new(),
	};
	Ok(props)
}
