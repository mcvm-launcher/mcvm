pub mod client;
pub mod server;

use color_print::cprintln;

use crate::{data::{instance::InstKind, user::Auth}, util::json, io::files::paths::Paths};

use super::{Instance, create::CreateError};

#[derive(Debug, thiserror::Error)]
pub enum LaunchError {
	#[error("Failed to create instance:\n{}", .0)]
	Create(#[from] CreateError),
	#[error("Java is not installed")]
	Java,
	#[error("Command failed:\n{}", .0)]
	Command(std::io::Error),
	#[error("Failed to evaluate json file:\n{}", .0)]
	Json(#[from] json::JsonError)
}

impl Instance {
	// Launch the instance
	pub async fn launch(
		&mut self,
		version_manifest: &json::JsonObject,
		paths: &Paths,
		auth: &Auth
	) -> Result<(), LaunchError> {
		cprintln!("Checking for updates...");
		match &self.kind {
			InstKind::Client => {
				self.create_client(version_manifest, paths, false, false).await?;
				cprintln!("<g>Launching!");
				self.launch_client(paths, auth)?;
			},
			InstKind::Server => {
				self.create_server(version_manifest, paths, false, false).await?;
				cprintln!("<g>Launching!");
				self.launch_server(paths)?;
			}
		}
		Ok(())
	}
}
