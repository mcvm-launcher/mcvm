use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use mcvm_core::Paths;
use mcvm_mods::paper;
use mcvm_plugin::{api::CustomPlugin, hooks::OnInstanceSetupResult};
use mcvm_shared::{modifications::ServerType, versions::VersionPattern, Side};

fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::new("paper")?;
	plugin.on_instance_setup(|_, arg| {
		let Some(side) = arg.side else {
			bail!("Instance side is empty");
		};

		// Make sure this is a Paper or Folia server instance
		if side != Side::Server
			|| (arg.server_type != ServerType::Paper && arg.server_type != ServerType::Folia)
		{
			return Ok(OnInstanceSetupResult::default());
		}

		let mode = if arg.server_type == ServerType::Paper {
			paper::Mode::Paper
		} else {
			paper::Mode::Folia
		};

		let client = mcvm_net::download::Client::new();

		let runtime = tokio::runtime::Runtime::new()?;

		// Check if this Minecraft version is available
		let versions = runtime
			.block_on(paper::get_all_versions(mode, &client))
			.context("Failed to get list of Paper versions")?;

		if !versions.iter().any(|x| *x == arg.version_info.version) {
			bail!("Could not find a Paper version for the given Minecraft version");
		}

		// Get the build numbers (actual project versions)
		let build_nums = runtime
			.block_on(paper::get_builds(mode, &arg.version_info.version, &client))
			.context("Failed to get list of build numbers for {mode} project")?;

		let build_nums_strings: Vec<_> = build_nums.iter().map(|x| x.to_string()).collect();

		let desired_version = arg
			.desired_game_modification_version
			.unwrap_or(VersionPattern::Any)
			.get_match(&build_nums_strings)
			.with_context(|| format!("Failed to find the given {mode} version"))?;
		let desired_build_num: u16 = desired_version
			.parse()
			.context("The desired version must be a an unsigned integer")?;

		let current_build_num: Option<u16> = arg
			.current_game_modification_version
			.and_then(|x| x.parse().ok());

		// If the new and current build nums mismatch, then get info for the current build num and
		// use it to teardown
		if let Some(current_build_num) = current_build_num {
			if desired_build_num != current_build_num {
				let remote_jar_file_name = runtime
					.block_on(paper::get_jar_file_name(
						mode,
						&arg.version_info.version,
						current_build_num,
						&client,
					))
					.with_context(|| {
						format!("Failed to get JAR file name for current {mode} version")
					})?;

				remove_paper(&PathBuf::from(arg.game_dir), remote_jar_file_name)
					.with_context(|| format!("Failed to remove {mode} from the instance"))?;
			}
		}

		// Get the name of the remote JAR file we need to download
		let remote_jar_file_name = runtime
			.block_on(paper::get_jar_file_name(
				mode,
				&arg.version_info.version,
				desired_build_num,
				&client,
			))
			.with_context(|| format!("Failed to get JAR file name for new {mode} version"))?;

		// Download it
		let paths = Paths::new()?;
		runtime
			.block_on(paper::download_server_jar(
				mode,
				&arg.version_info.version,
				desired_build_num,
				&remote_jar_file_name,
				&paths,
				&client,
			))
			.with_context(|| format!("Failed to download JAR file for {mode}"))?;

		let jar_path = paper::get_local_jar_path(mode, &arg.version_info.version, &paths);
		let main_class = paper::PAPER_SERVER_MAIN_CLASS;

		Ok(OnInstanceSetupResult {
			main_class_override: Some(main_class.into()),
			jar_path_override: Some(jar_path.to_string_lossy().to_string()),
			game_modification_version: Some(desired_build_num.to_string()),
			..Default::default()
		})
	})?;

	Ok(())
}

fn remove_paper(game_dir: &Path, paper_file_name: String) -> anyhow::Result<()> {
	let paper_path = game_dir.join(paper_file_name);
	if paper_path.exists() {
		std::fs::remove_file(paper_path).context("Failed to remove Paper jar")?;
	}

	Ok(())
}
