use anyhow::{bail, Context};
use mcvm_core::Paths;
use mcvm_mods::sponge;
use mcvm_plugin::{api::CustomPlugin, hooks::OnInstanceSetupResult};
use mcvm_shared::{modifications::ServerType, Side};

fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::new("sponge")?;
	plugin.on_instance_setup(|_, arg| {
		let Some(side) = arg.side else {
			bail!("Instance side is empty");
		};

		// Make sure this is a Sponge server instance
		if side != Side::Server || arg.server_type != ServerType::Sponge {
			return Ok(OnInstanceSetupResult::default());
		}

		let paths = Paths::new().context("Failed to create paths")?;

		let client = mcvm_net::download::Client::new();

		let runtime = tokio::runtime::Runtime::new()?;

		let sponge_version = runtime
			.block_on(sponge::get_newest_version(
				sponge::Mode::Vanilla,
				&arg.version_info.version,
				&client,
			))
			.context("Failed to get latest Sponge version")?;

		runtime
			.block_on(sponge::download_server_jar(
				sponge::Mode::Vanilla,
				&arg.version_info.version,
				&sponge_version,
				&paths,
				&client,
			))
			.context("Failed to download Sponge server JAR")?;

		let jar_path =
			sponge::get_local_jar_path(sponge::Mode::Vanilla, &arg.version_info.version, &paths);
		let main_class = sponge::SPONGE_SERVER_MAIN_CLASS;

		Ok(OnInstanceSetupResult {
			main_class_override: Some(main_class.into()),
			jar_path_override: Some(jar_path.to_string_lossy().to_string()),
			..Default::default()
		})
	})?;

	Ok(())
}
