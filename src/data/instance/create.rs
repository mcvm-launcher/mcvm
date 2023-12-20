use std::collections::{HashMap, HashSet};
use std::fs::File;

use anyhow::Context;
use mcvm_core::io::java::classpath::Classpath;
use mcvm_core::user::uuid::hyphenate_uuid;
use mcvm_core::user::{User, UserManager};
use mcvm_core::version::InstalledVersion;
use mcvm_mods::paper;
use mcvm_mods::{fabric_quilt, sponge};
use mcvm_options::{self, client::write_options_txt, server::write_server_properties};
use mcvm_shared::modifications::{Modloader, ServerType};
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel, OutputProcess};
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

	/// Create a client
	async fn create_client(
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

	/// Create a server
	async fn create_server(
		&mut self,
		manager: &UpdateManager,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<UpdateMethodResult> {
		debug_assert!(matches!(self.kind, InstKind::Server { .. }));

		let mut out = UpdateMethodResult::new();

		let version = &manager.version_info.get().version;
		self.ensure_dirs(paths)?;

		let classpath = if let Modloader::Fabric | Modloader::Quilt =
			self.config.modifications.get_modloader(self.kind.to_side())
		{
			self.get_fabric_quilt(paths, manager).await?
		} else {
			Classpath::new()
		};

		// TODO: This is janky
		match self.config.modifications.server_type {
			ServerType::Paper => {
				let process = OutputProcess::new(o);
				process.0.display(
					MessageContents::StartProcess("Checking for Paper updates".into()),
					MessageLevel::Important,
				);

				let build_num = paper::get_newest_build(paper::Mode::Paper, version, client)
					.await
					.context("Failed to get the newest Paper version")?;
				let file_name =
					paper::get_jar_file_name(paper::Mode::Paper, version, build_num, client)
						.await
						.context("Failed to get the Paper file name")?;
				let paper_jar_path =
					paper::get_local_jar_path(paper::Mode::Paper, version, &paths.core);
				if !manager.should_update_file(&paper_jar_path) {
					process.0.display(
						MessageContents::Success("Paper is up to date".into()),
						MessageLevel::Important,
					);
				} else {
					process.0.display(
						MessageContents::StartProcess("Downloading Paper server".into()),
						MessageLevel::Important,
					);
					paper::download_server_jar(
						paper::Mode::Paper,
						version,
						build_num,
						&file_name,
						&paths.core,
						client,
					)
					.await
					.context("Failed to download Paper server JAR")?;
					process.0.display(
						MessageContents::Success("Paper server downloaded".into()),
						MessageLevel::Important,
					);
				}

				out.files_updated.insert(paper_jar_path.clone());
				self.jar_path_override = Some(paper_jar_path);
			}
			ServerType::Folia => {
				let process = OutputProcess::new(o);
				process.0.display(
					MessageContents::StartProcess("Checking for Folia updates".into()),
					MessageLevel::Important,
				);

				let build_num = paper::get_newest_build(paper::Mode::Folia, version, client)
					.await
					.context("Failed to get the newest Folia version")?;
				let file_name =
					paper::get_jar_file_name(paper::Mode::Folia, version, build_num, client)
						.await
						.context("Failed to get the Folia file name")?;
				let folia_jar_path =
					paper::get_local_jar_path(paper::Mode::Folia, version, &paths.core);
				if !manager.should_update_file(&folia_jar_path) {
					process.0.display(
						MessageContents::Success("Folia is up to date".into()),
						MessageLevel::Important,
					);
				} else {
					process.0.display(
						MessageContents::StartProcess("Downloading Folia server".into()),
						MessageLevel::Important,
					);
					paper::download_server_jar(
						paper::Mode::Folia,
						version,
						build_num,
						&file_name,
						&paths.core,
						client,
					)
					.await
					.context("Failed to download Paper server JAR")?;
					process.0.display(
						MessageContents::Success("Paper server downloaded".into()),
						MessageLevel::Important,
					);
				}

				out.files_updated.insert(folia_jar_path.clone());
				self.jar_path_override = Some(folia_jar_path);
			}
			ServerType::Sponge => {
				let process = OutputProcess::new(o);
				process.0.display(
					MessageContents::StartProcess("Checking for Sponge updates".into()),
					MessageLevel::Important,
				);

				let sponge_version =
					sponge::get_newest_version(sponge::Mode::Vanilla, version, client)
						.await
						.context("Failed to get newest Sponge version")?;
				let sponge_jar_path =
					sponge::get_local_jar_path(sponge::Mode::Vanilla, version, &paths.core);
				if !manager.should_update_file(&sponge_jar_path) {
					process.0.display(
						MessageContents::Success("Sponge is up to date".into()),
						MessageLevel::Important,
					);
				} else {
					process.0.display(
						MessageContents::StartProcess("Downloading Sponge server".into()),
						MessageLevel::Important,
					);
					sponge::download_server_jar(
						sponge::Mode::Vanilla,
						version,
						&sponge_version,
						&paths.core,
						client,
					)
					.await
					.context("Failed to download Sponge server JAR")?;
					process.0.display(
						MessageContents::Success("Sponge server downloaded".into()),
						MessageLevel::Important,
					);
				}

				out.files_updated.insert(sponge_jar_path.clone());
				self.jar_path_override = Some(sponge_jar_path);
			}
			_ => {}
		}

		let mut keys = HashMap::new();
		let version_info = manager.version_info.get();
		if let Some(global_options) = &manager.options {
			if let Some(global_options) = &global_options.server {
				let global_keys = mcvm_options::server::create_keys(global_options, version_info)
					.context("Failed to create keys for global options")?;
				keys.extend(global_keys);
			}
		}
		if let InstKind::Server {
			options: Some(options),
		} = &self.kind
		{
			let override_keys = mcvm_options::server::create_keys(options, version_info)
				.context("Failed to create keys for override options")?;
			keys.extend(override_keys);
		}
		if !keys.is_empty() {
			let options_path = self.dirs.get().game_dir.join("server.properties");
			write_server_properties(keys, &options_path)
				.await
				.context("Failed to write server.properties")?;
		}

		self.classpath_extension = classpath;

		Ok(out)
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
