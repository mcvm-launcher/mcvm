use std::process::Command;

use anyhow::{bail, Context};

use crate::data::instance::{Instance, InstKind};
use crate::io::files::paths::Paths;

impl Instance {
	/// Launch a server
	pub fn launch_server(&mut self, paths: &Paths) -> anyhow::Result<()> {
		debug_assert!(self.kind == InstKind::Server);
		match &self.java {
			Some(java) => match &java.path {
				Some(java_path) => {
					let jre_path = java_path.join("bin/java");
					let server_dir = self.get_subdir(paths);

					let mut command = Command::new(
						jre_path
							.to_str()
							.context("Failed to convert java path to a string")?,
					);
					command.current_dir(server_dir);
					command.args(&self.launch.generate_jvm_args());
					if let Some(classpath) = &self.classpath {
						command.arg("-cp");
						command.arg(classpath.get_str());
					}
					command.arg("-jar");
					let jar_path_str = self
						.jar_path
						.as_ref()
						.expect("Jar path missing")
						.to_str()
						.context("Failed to convert server.jar path to a string")?;
					command.arg(jar_path_str);
					if let Some(main_class) = &self.main_class {
						command.arg(main_class);
					}
					command.arg("nogui");
					let mut child = command.spawn().context("Failed to spawn child process")?;
					command.args(&self.launch.game_args);

					child.wait().context("Failed to wait for child process to spawn")?;

					Ok(())
				}
				None => bail!("Java path is missing"),
			},
			None => bail!("Java installation missing"),
		}
	}
}
