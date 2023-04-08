pub mod client;
pub mod server;

use anyhow::Context;
use color_print::cprintln;

use crate::data::profile::update::UpdateManager;
use crate::data::{instance::InstKind, user::Auth};
use crate::io::files::paths::Paths;
use crate::util::print::PrintOptions;
use crate::util::versions::MinecraftVersion;

use super::Instance;

impl Instance {
	// Launch the instance
	pub async fn launch(&mut self, paths: &Paths, auth: &Auth, debug: bool, version: &MinecraftVersion) -> anyhow::Result<()> {
		cprintln!("Checking for updates...");
		let options = PrintOptions::new(false, 0);
		let mut manager = UpdateManager::new(options, false);
		manager.fulfill_version_manifest(paths, version).await.context("Failed to get version data")?;
		manager.add_requirements(self.get_requirements());
		manager
			.fulfill_requirements(paths)
			.await
			.context("Update failed")?;

		self.create(&manager, paths)
			.await
			.context("Failed to update instance")?;
		let version = manager.found_version.as_ref().expect("Found version missing");
		let version_list = manager.version_list.as_ref().expect("Found version missing");
		cprintln!("<g>Launching!");
		match &self.kind {
			InstKind::Client { .. } => {
				self.launch_client(paths, auth, debug, version, version_list)
					.context("Failed to launch client")?;
			}
			InstKind::Server { .. } => {
				self.launch_server(paths, debug, version, version_list)
					.context("Failed to launch server")?;
			}
		}
		Ok(())
	}
}
