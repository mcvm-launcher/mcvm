use std::collections::HashMap;

use anyhow::Context;

use crate::data::instance::{InstKind, Instance};
use crate::io::files::paths::Paths;
use crate::io::launch::{launch, LaunchArgument};
use mcvm_shared::versions::VersionInfo;

impl Instance {
	/// Launch a server
	pub fn launch_server(
		&mut self,
		paths: &Paths,
		debug: bool,
		version_info: &VersionInfo,
	) -> anyhow::Result<()> {
		debug_assert!(matches!(self.kind, InstKind::Server { .. }));
		let java_path = self.java.get().path.get();
		let jre_path = java_path.join("bin/java");
		let server_dir = self.get_subdir(paths);

		let mut jvm_args = Vec::new();
		let mut game_args = Vec::new();
		if let Some(classpath) = &self.classpath {
			jvm_args.push(String::from("-cp"));
			jvm_args.push(classpath.get_str());
		}
		game_args.push(String::from("nogui"));

		let launch_args = LaunchArgument {
			instance_name: &self.id,
			side: self.kind.to_side(),
			options: &self.launch,
			debug,
			version_info,
			cwd: &server_dir,
			command: jre_path
				.to_str()
				.context("Failed to convert java path to a string")?,
			jvm_args: &jvm_args,
			main_class: self.main_class.as_deref(),
			game_args: &game_args,
			additional_env_vars: &HashMap::new(),
		};

		launch(paths, &launch_args).context("Failed to run launch command")?;

		Ok(())
	}
}
