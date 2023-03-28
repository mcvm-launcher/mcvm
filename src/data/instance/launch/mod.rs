pub mod client;
pub mod server;

use anyhow::Context;
use color_print::cprintln;

use crate::data::profile::update::UpdateManager;
use crate::data::{instance::InstKind, user::Auth};
use crate::io::files::paths::Paths;
use crate::util::print::PrintOptions;

use super::Instance;

impl Instance {
	// Launch the instance
	pub async fn launch(
		&mut self,
		paths: &Paths,
		auth: &Auth,
		debug: bool,
	) -> anyhow::Result<()> {
		cprintln!("Checking for updates...");
		let options = PrintOptions::new(false, 0);
		let mut manager = UpdateManager::new(options, false);
		manager.add_requirements(self.get_requirements());
		manager.fulfill_requirements(paths, &self.version).await.context("Update failed")?;
		
		self.create(&manager, paths).await.context("Failed to update instance")?;
		cprintln!("<g>Launching!");
		match &self.kind {
			InstKind::Client => {
				self.launch_client(paths, auth, debug).context("Failed to launch client")?;
			}
			InstKind::Server => {
				self.launch_server(paths, debug).context("Failed to launch server")?;
			}
		}
		Ok(())
	}
}
