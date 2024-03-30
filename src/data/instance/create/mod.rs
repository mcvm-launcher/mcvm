/// Creation of the client
mod client;
/// Creation of the server
mod server;

use std::collections::HashSet;
use std::fs::File;

use anyhow::Context;
use mcvm_core::user::uuid::hyphenate_uuid;
use mcvm_core::user::{User, UserManager};
use mcvm_core::version::InstalledVersion;
use mcvm_mods::fabric_quilt;
use mcvm_shared::modifications::Modloader;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use reqwest::Client;

use crate::data::profile::update::manager::{UpdateManager, UpdateMethodResult, UpdateRequirement};
use crate::io::files::{self, paths::Paths};

use super::{InstKind, Instance};

/// The default main class for the server
pub const DEFAULT_SERVER_MAIN_CLASS: &str = "net.minecraft.server.Main";
/// The main class for a Paper server
pub const PAPER_SERVER_MAIN_CLASS: &str = "io.papermc.paperclip.Main";

impl Instance {
	/// Get the requirements for this instance
	pub fn get_requirements(&self) -> HashSet<UpdateRequirement> {
		let mut out = HashSet::new();
		match self.config.modifications.get_modloader(self.kind.to_side()) {
			Modloader::Fabric => {
				out.insert(UpdateRequirement::FabricQuilt(
					fabric_quilt::Mode::Fabric,
					self.kind.to_side(),
				));
			}
			Modloader::Quilt => {
				out.insert(UpdateRequirement::FabricQuilt(
					fabric_quilt::Mode::Quilt,
					self.kind.to_side(),
				));
			}
			_ => {}
		};
		out.insert(UpdateRequirement::Options);
		match &self.kind {
			InstKind::Client { .. } => {
				if self.config.launch.use_log4j_config {
					out.insert(UpdateRequirement::ClientLoggingConfig);
				}
			}
			InstKind::Server { .. } => {}
		}
		out
	}

	/// Create the data for the instance.
	pub async fn create<'core>(
		&mut self,
		version: &'core mut InstalledVersion<'core, 'core>,
		manager: &UpdateManager,
		paths: &Paths,
		users: &UserManager,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<(UpdateMethodResult, mcvm_core::Instance<'core>)> {
		// Start by setting up custom changes
		let result = match &self.kind {
			InstKind::Client { .. } => {
				o.display(
					MessageContents::Header(format!("Updating client {}", self.id)),
					MessageLevel::Important,
				);
				o.start_section();
				let result = self
					.create_client(manager, paths, users)
					.await
					.context("Failed to create client")?;
				Ok::<_, anyhow::Error>(result)
			}
			InstKind::Server { .. } => {
				o.display(
					MessageContents::Header(format!("Updating server {}", self.id)),
					MessageLevel::Important,
				);
				o.start_section();
				let result = self
					.create_server(manager, paths, client, o)
					.await
					.context("Failed to create server")?;
				Ok(result)
			}
		}?;
		// Make the core instance
		let inst = self
			.create_core_instance(version, paths, o)
			.await
			.context("Failed to create core instance")?;
		o.end_section();

		Ok((result, inst))
	}

	/// Create a keypair file in the instance
	fn create_keypair(&mut self, user: &User, paths: &Paths) -> anyhow::Result<()> {
		if let Some(uuid) = user.get_uuid() {
			if let Some(keypair) = user.get_keypair() {
				self.ensure_dirs(paths)?;
				let keys_dir = self.dirs.get().game_dir.join("profilekeys");
				let hyphenated_uuid = hyphenate_uuid(uuid).context("Failed to hyphenate UUID")?;
				let path = keys_dir.join(format!("{hyphenated_uuid}.json"));
				files::create_leading_dirs(&path)?;

				let mut file = File::create(path).context("Failed to create keypair file")?;
				serde_json::to_writer(&mut file, keypair)
					.context("Failed to write keypair to file")?;
			}
		}

		Ok(())
	}
}
