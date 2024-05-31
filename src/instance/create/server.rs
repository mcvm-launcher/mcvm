use anyhow::Context;
use mcvm_core::io::java::classpath::Classpath;
use mcvm_mods::paper;
use mcvm_mods::sponge;
use mcvm_shared::modifications::{Modloader, ServerType};
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel, OutputProcess};
use reqwest::Client;

use crate::io::paths::Paths;

use super::super::update::manager::{UpdateManager, UpdateMethodResult};
use super::{InstKind, Instance};

impl Instance {
	/// Create a server
	pub async fn create_server(
		&mut self,
		manager: &UpdateManager,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<UpdateMethodResult> {
		debug_assert!(matches!(self.kind, InstKind::Server { .. }));

		let mut out = UpdateMethodResult::new();

		self.ensure_dirs(paths)?;

		// Initialize the classpath based on the modifications we are using
		let classpath = if let Modloader::Fabric | Modloader::Quilt =
			self.config.modifications.get_modloader(self.kind.to_side())
		{
			self.get_fabric_quilt(paths, manager)
				.context("Failed to get Fabric/Quilt")?
		} else {
			Classpath::new()
		};

		match self.config.modifications.server_type {
			ServerType::Paper => {
				let result = self
					.create_paper_folia(paper::Mode::Paper, manager, paths, client, o)
					.await
					.context("Failed to create Paper")?;
				out.merge(result);
			}
			ServerType::Folia => {
				let result = self
					.create_paper_folia(paper::Mode::Folia, manager, paths, client, o)
					.await
					.context("Failed to create Folia")?;
				out.merge(result);
			}
			ServerType::Sponge => {
				let result = self
					.create_sponge(manager, paths, client, o)
					.await
					.context("Failed to create Sponge")?;
				out.merge(result);
			}
			_ => {}
		}

		self.modification_data.classpath_extension = classpath;

		Ok(out)
	}

	/// Create data for Paper or Folia on the server
	async fn create_paper_folia(
		&mut self,
		mode: paper::Mode,
		manager: &UpdateManager,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<UpdateMethodResult> {
		let version = &manager.version_info.get().version;

		let process = OutputProcess::new(o);
		process.0.display(
			MessageContents::StartProcess("Checking for {mode} updates".into()),
			MessageLevel::Important,
		);

		let build_num = paper::get_newest_build(mode, version, client)
			.await
			.context("Failed to get the newest {mode} version")?;
		let file_name = paper::get_jar_file_name(mode, version, build_num, client)
			.await
			.context("Failed to get the {mode} file name")?;
		let paper_jar_path = paper::get_local_jar_path(mode, version, &paths.core);
		if !manager.should_update_file(&paper_jar_path) {
			process.0.display(
				MessageContents::Success(format!("{mode} is up to date")),
				MessageLevel::Important,
			);
		} else {
			process.0.display(
				MessageContents::StartProcess("Downloading {mode} server".into()),
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
			.context("Failed to download {mode} server JAR")?;
			process.0.display(
				MessageContents::Success("{mode} server downloaded".into()),
				MessageLevel::Important,
			);
		}

		self.modification_data.jar_path_override = Some(paper_jar_path.clone());

		Ok(UpdateMethodResult::from_path(paper_jar_path))
	}

	/// Create data for Sponge on the serer
	async fn create_sponge(
		&mut self,
		manager: &UpdateManager,
		paths: &Paths,
		client: &Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<UpdateMethodResult> {
		let version = &manager.version_info.get().version;

		let process = OutputProcess::new(o);
		process.0.display(
			MessageContents::StartProcess("Checking for Sponge updates".into()),
			MessageLevel::Important,
		);

		let sponge_version = sponge::get_newest_version(sponge::Mode::Vanilla, version, client)
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

		self.modification_data.jar_path_override = Some(sponge_jar_path.clone());
		Ok(UpdateMethodResult::from_path(sponge_jar_path))
	}
}
