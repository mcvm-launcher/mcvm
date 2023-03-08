use std::process::Command;

use crate::{data::instance::Instance, io::files::paths::Paths};

use super::LaunchError;

impl Instance {
	pub fn launch_server(&mut self, paths: &Paths) -> Result<(), LaunchError> {
		match &self.java {
			Some(java) => match &java.path {
				Some(java_path) => {
					let jre_path = java_path.join("bin/java");
					let server_dir = self.get_subdir(paths);

					let mut command = Command::new(jre_path.to_str().expect("Failed to convert java path to a string"));
					command.current_dir(server_dir);
					command.args(&self.launch.args.jvm.parse());
					if let Some(classpath) = &self.classpath {
						command.arg("-cp");
						command.arg(classpath);
					}
					command.arg("-jar");
					let jar_path_str = self.jar_path.as_ref().expect("Jar path missing").to_str()
						.expect("Failed to convert server.jar path to a string");
					command.arg(jar_path_str);
					if let Some(main_class) = &self.main_class {
						command.arg(main_class);
					}
					command.arg("nogui");
					let mut child = match command.spawn() {
						Ok(child) => child,
						Err(err) => return Err(LaunchError::Command(err))
					};
					command.args(&self.launch.args.game.parse());
					
					child.wait().expect("Child failed");

					Ok(())
				}
				None => Err(LaunchError::Java)
			}
			None => Err(LaunchError::Java)
		}
	}
}
