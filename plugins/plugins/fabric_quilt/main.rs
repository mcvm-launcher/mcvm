use std::path::PathBuf;

use anyhow::{bail, Context};
use mcvm_core::io::update::UpdateManager;
use mcvm_mods::fabric_quilt;
use mcvm_plugin::{api::CustomPlugin, hooks::OnInstanceSetupResult};
use mcvm_shared::{
	modifications::{ClientType, ServerType},
	Side, UpdateDepth,
};

fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::from_manifest_file("fabric_quilt", include_str!("plugin.json"))?;
	plugin.on_instance_setup(|mut ctx, arg| {
		let Some(side) = arg.side else {
			bail!("Instance side is empty");
		};

		// Make sure this is a Fabric or Quilt instance
		if (side == Side::Client
			&& !(arg.client_type == ClientType::Fabric || arg.client_type == ClientType::Quilt))
			|| (side == Side::Server
				&& !(arg.server_type == ServerType::Fabric || arg.server_type == ServerType::Quilt))
		{
			return Ok(OnInstanceSetupResult::default());
		}

		let mode = if side == Side::Client {
			if arg.client_type == ClientType::Fabric {
				fabric_quilt::Mode::Fabric
			} else {
				fabric_quilt::Mode::Quilt
			}
		} else {
			if arg.server_type == ServerType::Fabric {
				fabric_quilt::Mode::Fabric
			} else {
				fabric_quilt::Mode::Quilt
			}
		};

		let internal_dir = PathBuf::from(arg.internal_dir);

		let manager = UpdateManager::new(UpdateDepth::Full);

		let client = mcvm_net::download::Client::new();

		let runtime = tokio::runtime::Runtime::new()?;

		let meta = runtime
			.block_on(fabric_quilt::get_meta(
				&arg.version_info.version,
				&mode,
				&internal_dir,
				&manager,
				&client,
			))
			.context("Failed to get metadata")?;

		let libraries_dir = internal_dir.join("libraries");

		runtime
			.block_on(fabric_quilt::download_files(
				&meta,
				&libraries_dir,
				mode,
				&manager,
				&client,
				ctx.get_output(),
			))
			.context("Failed to download common files")?;

		runtime
			.block_on(fabric_quilt::download_side_specific_files(
				&meta,
				&libraries_dir,
				side,
				&manager,
				&client,
			))
			.context("Failed to download side-specific files")?;

		let classpath = fabric_quilt::get_classpath(&meta, &libraries_dir, side)
			.context("Failed to get classpath")?;

		let main_class = meta
			.launcher_meta
			.main_class
			.get_main_class_string(side)
			.to_string();

		Ok(OnInstanceSetupResult {
			main_class_override: Some(main_class),
			classpath_extension: classpath.get_entries().to_vec(),
			..Default::default()
		})
	})?;

	Ok(())
}
