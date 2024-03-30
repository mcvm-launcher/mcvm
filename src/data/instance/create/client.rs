use std::collections::HashMap;

use anyhow::Context;
use mcvm_core::io::java::classpath::Classpath;
use mcvm_core::user::UserManager;
use mcvm_options::{self, client::write_options_txt};
use mcvm_shared::modifications::Modloader;

use crate::data::profile::update::manager::{UpdateManager, UpdateMethodResult};
use crate::io::files::paths::Paths;

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
					.await
					.context("Failed to install Fabric/Quilt")?,
			);
		}

		// Options
		let mut keys = HashMap::new();
		let version_info = &manager.version_info.get();
		if let Some(global_options) = &manager.options {
			if let Some(global_options) = &global_options.client {
				let global_keys = mcvm_options::client::create_keys(global_options, version_info)
					.context("Failed to create keys for global options")?;
				keys.extend(global_keys);
			}
		}
		if let InstKind::Client {
			options: Some(options),
			..
		} = &self.kind
		{
			let override_keys = mcvm_options::client::create_keys(options, version_info)
				.context("Failed to create keys for override options")?;
			keys.extend(override_keys);
		}
		if !keys.is_empty() {
			let options_path = self.dirs.get().game_dir.join("options.txt");
			let data_version =
				mcvm_core::io::minecraft::get_data_version(version_info, &paths.core)
					.context("Failed to obtain data version")?;
			write_options_txt(keys, &options_path, &data_version)
				.await
				.context("Failed to write options.txt")?;
		}

		// Create keypair file
		if users.is_authenticated() {
			if let Some(user) = users.get_chosen_user() {
				self.create_keypair(user, paths)
					.context("Failed to create user keypair")?;
			}
		}

		self.classpath_extension = classpath;

		Ok(out)
	}
}
