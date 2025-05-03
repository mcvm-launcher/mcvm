use std::{
	fs::File,
	io::BufReader,
	path::{Path, PathBuf},
};

use anyhow::Context;
use mcvm_core::util::versions::MinecraftVersionDeser;
use mcvm_plugin::{api::CustomPlugin, hooks::ImportInstanceResult};
use mcvm_shared::{
	modifications::{ClientType, ServerType},
	Side,
};
use serde::{Deserialize, Serialize};
use zip::{write::FileOptions, ZipArchive, ZipWriter};

fn main() -> anyhow::Result<()> {
	let mut plugin =
		CustomPlugin::from_manifest_file("mcvm_transfer", include_str!("plugin.json"))?;

	plugin.export_instance(|_, arg| {
		let game_dir = PathBuf::from(arg.game_dir);
		let target_path = PathBuf::from(arg.result_path);
		let target_file = File::create(target_path).context("Failed to open target file")?;

		// Write the instance files
		let mut zip = ZipWriter::new(target_file);

		visit_dir(&game_dir, &mut zip, &game_dir).context("Failed to read instance directory")?;

		fn visit_dir(dir: &Path, zip: &mut ZipWriter<File>, game_dir: &Path) -> anyhow::Result<()> {
			let dir_read = dir.read_dir().context("Failed to read directory")?;

			for item in dir_read {
				let item = item?;
				if item.file_type()?.is_dir() {
					visit_dir(&item.path(), zip, game_dir)?;
				} else {
					if !should_include_file(&item.path()) {
						continue;
					}

					zip.start_file_from_path(
						item.path().strip_prefix(game_dir)?,
						FileOptions::<()>::default(),
					)?;
					let mut src = BufReader::new(File::open(item.path())?);
					std::io::copy(&mut src, zip).context("Failed to copy file into ZIP")?;
				}
			}

			Ok(())
		}

		// Write the metadata file
		zip.start_file("mcvm_meta.json", FileOptions::<()>::default())
			.context("Failed to create metadata file in export")?;

		let meta = Metadata {
			id: arg.id,
			name: arg.name,
			side: arg.side,
			minecraft_version: arg.minecraft_version,
			client_type: arg.client_type,
			server_type: arg.server_type,
		};

		serde_json::to_writer(&mut zip, &meta).context("Failed to write metadata file")?;

		Ok(())
	})?;

	plugin.import_instance(|_, arg| {
		let source_path = PathBuf::from(arg.source_path);
		let target_path = PathBuf::from(arg.result_path);

		// Read the metadata
		let mut zip = ZipArchive::new(File::open(source_path).context("Failed to open instance")?)?;
		let mut meta_file = zip
			.by_name("mcvm_meta.json")
			.context("Metadata file is missing in instance")?;
		let meta: Metadata = serde_json::from_reader(&mut meta_file)
			.context("Failed to deserialize instance metadata")?;

		std::mem::drop(meta_file);

		// We need to write in the .minecraft directory for clients
		let target_path = match meta.side.context("Side is missing in metadata")? {
			Side::Client => target_path.join(".minecraft"),
			Side::Server => target_path,
		};

		// Extract all the instance files
		zip.extract(target_path)
			.context("Failed to extract instance")?;

		Ok(ImportInstanceResult {
			format: arg.format,
			name: meta.name,
			side: meta.side,
			version: meta.minecraft_version,
			client_type: meta.client_type,
			server_type: meta.server_type,
		})
	})?;

	Ok(())
}

/// Checks if a file should be included in the export
fn should_include_file(path: &Path) -> bool {
	if let Some(file_name) = path.file_name() {
		let file_name = file_name.to_string_lossy();
		if file_name.starts_with("mcvm_") {
			return false;
		}
	}

	true
}

/// Metadata file for exported instances
#[derive(Serialize, Deserialize)]
struct Metadata {
	id: String,
	name: Option<String>,
	side: Option<Side>,
	minecraft_version: Option<MinecraftVersionDeser>,
	client_type: Option<ClientType>,
	server_type: Option<ServerType>,
}
