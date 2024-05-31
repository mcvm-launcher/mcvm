use anyhow::Context;
use mcvm_core::io::java::classpath::Classpath;
use mcvm_core::user::UserManager;
use mcvm_shared::modifications::Modloader;

use crate::io::paths::Paths;

use super::super::update::manager::{UpdateManager, UpdateMethodResult};
use super::{InstKind, Instance};

impl Instance {
	/// Create a client
	pub async fn create_client(
		&mut self,
		manager: &UpdateManager,
		paths: &Paths,
		users: &UserManager,
	) -> anyhow::Result<UpdateMethodResult> {
		debug_assert!(matches!(self.kind, InstKind::Client { .. }));

		let out = UpdateMethodResult::new();
		self.ensure_dirs(paths)?;

		let mut classpath = Classpath::new();

		if let Modloader::Fabric | Modloader::Quilt =
			self.config.modifications.get_modloader(self.kind.to_side())
		{
			classpath.extend(
				self.get_fabric_quilt(paths, manager)
					.context("Failed to install Fabric/Quilt")?,
			);
		}

		// Create keypair file
		if users.is_authenticated() {
			if let Some(user) = users.get_chosen_user() {
				self.create_keypair(user, paths)
					.context("Failed to create user keypair")?;
			}
		}

		self.modification_data.classpath_extension = classpath;

		Ok(out)
	}
}
