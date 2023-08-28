pub mod client;
pub mod server;

use anyhow::Context;
use color_print::cprintln;

use crate::data::profile::update::UpdateManager;
use crate::data::{instance::InstKind, user::Auth};
use crate::io::files::paths::Paths;
use crate::io::lock::Lockfile;
use crate::util::print::PrintOptions;
use crate::util::versions::MinecraftVersion;

use super::Instance;

impl Instance {
	// Launch the instance
	pub async fn launch(
		&mut self,
		paths: &Paths,
		lock: &mut Lockfile,
		auth: &Auth,
		debug: bool,
		token: Option<String>,
		version: &MinecraftVersion,
	) -> anyhow::Result<()> {
		cprintln!("Checking for updates...");
		let options = PrintOptions::new(false, 0);
		let mut manager = UpdateManager::new(options, false, true);
		manager
			.fulfill_version_manifest(paths, version)
			.await
			.context("Failed to get version data")?;
		manager.add_requirements(self.get_requirements());
		manager
			.fulfill_requirements(paths, lock)
			.await
			.context("Update failed")?;

		self.create(&manager, paths, auth)
			.await
			.context("Failed to update instance")?;
		let version_info = manager.version_info.get();
		cprintln!("<g>Launching!");
		match &self.kind {
			InstKind::Client { .. } => {
				self.launch_client(paths, auth, debug, token, version_info)
					.context("Failed to launch client")?;
			}
			InstKind::Server { .. } => {
				if token.is_some() {
					cprintln!("<y>Notice: Ignoring 'token' argument for server instance");
				}
				self.launch_server(paths, debug, version_info)
					.context("Failed to launch server")?;
			}
		}
		Ok(())
	}
}
