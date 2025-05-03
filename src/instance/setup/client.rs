use anyhow::Context;
use mcvm_core::user::UserManager;

use crate::io::paths::Paths;

use super::super::update::manager::UpdateMethodResult;
use super::{InstKind, Instance};

impl Instance {
	/// Set up data for a client
	pub async fn setup_client(
		&mut self,
		paths: &Paths,
		users: &UserManager,
	) -> anyhow::Result<UpdateMethodResult> {
		debug_assert!(matches!(self.kind, InstKind::Client { .. }));

		let out = UpdateMethodResult::new();
		self.ensure_dirs(paths)?;

		// Create keypair file
		if users.is_authenticated() {
			if let Some(user) = users.get_chosen_user() {
				self.create_keypair(user, paths)
					.context("Failed to create user keypair")?;
			}
		}

		Ok(out)
	}
}
