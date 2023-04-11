use anyhow::Context;

use crate::data::instance::{InstKind, Instance};
use crate::io::files::paths::Paths;
use crate::io::launch::launch;

impl Instance {
	/// Launch a server
	pub fn launch_server(
		&mut self,
		paths: &Paths,
		debug: bool,
		version: &str,
		version_list: &[String],
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
		jvm_args.push(String::from("-jar"));
		let jar_path_str = self.jar_path.get().to_str()
			.context("Failed to convert server.jar path to a string")?;
		jvm_args.push(String::from(jar_path_str));
		game_args.push(String::from("nogui"));

		launch(
			paths,
			&self.id,
			self.kind.to_side(),
			&self.launch,
			debug,
			version,
			version_list,
			&server_dir,
			jre_path
				.to_str()
				.context("Failed to convert java path to a string")?,
			&jvm_args,
			self.main_class.as_deref(),
			&game_args,
		)
		.context("Failed to run launch command")?;

		Ok(())
	}
}
