use std::collections::HashMap;

use anyhow::Context;
use mcvm_shared::output::MCVMOutput;

use crate::data::instance::launch::LaunchProcessProperties;
use crate::data::instance::{InstKind, Instance};
use crate::data::profile::update::manager::UpdateManager;
use crate::io::files::paths::Paths;
use mcvm_shared::versions::VersionInfo;

impl Instance {
	/// Launch a server
	pub fn launch_server(
		&mut self,
		paths: &Paths,
		version_info: &VersionInfo,
		_manager: &UpdateManager,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		assert!(matches!(self.kind, InstKind::Server { .. }));
		let java_path = self.java.get().path.get();
		let jre_path = java_path.join("bin/java");
		self.ensure_dirs(paths);
		let server_dir = &self.dirs.get().game_dir;

		let mut jvm_args = Vec::new();
		let mut game_args = Vec::new();
		if let Some(classpath) = &self.classpath {
			jvm_args.push("-cp".into());
			jvm_args.push(classpath.get_str());
		}
		game_args.push("nogui".into());

		let launch_properties = LaunchProcessProperties {
			cwd: server_dir,
			command: jre_path
				.to_str()
				.context("Failed to convert java path to a string")?,
			jvm_args: &jvm_args,
			main_class: self.main_class.as_deref(),
			game_args: &game_args,
			additional_env_vars: &HashMap::new(),
		};

		self.launch_game_process(launch_properties, version_info, paths, o)
			.context("Failed to launch game process")?;

		Ok(())
	}
}
