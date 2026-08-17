use std::{collections::HashMap, path::Path};

use anyhow::{Context, bail};
use nitro_config::instance::InstanceConfig;
use nitro_plugin::hook::hooks::{
	AddInstanceTransferFormats, ExportInstance, ExportInstanceArg, ImportInstance,
	ImportInstanceArg, InstanceTransferFeatureSupport, InstanceTransferFormat,
	InstanceTransferFormatDirection, MigrateInstances, MigrateInstancesArg,
};
use nitro_shared::lang::translate::TranslationKey;
use nitro_shared::output::{MessageContents, NitroOutput};
use nitro_shared::{Side, translate};

use crate::io::lock::Lockfile;
use crate::{io::paths::Paths, plugin::PluginManager};

use super::Instance;

impl Instance {
	/// Export this instance using the given format
	pub async fn export(
		&self,
		format: &str,
		result_path: &Path,
		formats: &Formats,
		plugins: &PluginManager,
		lock: &Lockfile,
		paths: &Paths,
		o: &mut impl NitroOutput,
	) -> anyhow::Result<()> {
		// Get and print info about the format
		let format = formats
			.formats
			.get(format)
			.context("Transfer format does not exist")?;

		let export_info = format
			.info
			.export
			.as_ref()
			.context("This format or the plugin providing it does not support exporting")?;

		output_support_warnings(export_info, o);

		if !lock.has_instance_done_first_update(&self.id) {
			bail!("Instance has not done it's first update and is not ready for transfer");
		}

		o.display(MessageContents::StartProcess(translate!(
			o,
			StartExporting,
			"instance" = &self.id,
			"format" = &format.info.id,
			"plugin" = &format.plugin
		)));

		let inst_lock = self.get_lockfile(paths)?;

		let Some(minecraft_version) = inst_lock.get_minecraft_version() else {
			bail!("Version missing. Please update the instance before exporting.");
		};

		let Some(inst_dir) = &self.dir else {
			bail!("This instance has no game directory and cannot be exported");
		};

		// Export using the plugin
		let arg = ExportInstanceArg {
			id: self.id.to_string(),
			format: format.info.id.clone(),
			config: self.config.clone(),
			minecraft_version: minecraft_version.clone(),
			loader_version: inst_lock.get_loader_version().cloned(),
			inst_dir: inst_dir.to_string_lossy().to_string(),
			result_path: result_path.canonicalize()?.to_string_lossy().to_string(),
		};
		let result = plugins
			.call_hook_on_plugin(ExportInstance, &format.plugin, &arg, paths, o)
			.await
			.context("Failed to export instance using plugin")?;

		if let Some(result) = result {
			result.result(o).await?;
			o.display(MessageContents::Success(
				o.translate(TranslationKey::FinishExporting).into(),
			));
		} else {
			o.debug(MessageContents::Error(
				o.translate(TranslationKey::ExportPluginNoResult).into(),
			));
		}

		Ok(())
	}

	/// Import an instance using the given format. Returns an InstanceConfig to add to the config file
	pub async fn import(
		id: &str,
		format: &str,
		source_path: &Path,
		side: Option<Side>,
		formats: &Formats,
		minecraft_versions: &[String],
		plugins: &PluginManager,
		paths: &Paths,
		o: &mut impl NitroOutput,
	) -> anyhow::Result<InstanceConfig> {
		if !source_path.exists() {
			bail!("Imported instance file does not exist");
		}

		// Get and print info about the format
		let format = formats
			.formats
			.get(format)
			.context("Transfer format does not exist")?;

		if format.info.needs_import_side && side.is_none() {
			bail!("This format requires a side to be specified");
		}

		let import_info = format
			.info
			.import
			.as_ref()
			.context("This format or the plugin providing it does not support importing")?;

		output_support_warnings(import_info, o);

		o.display(MessageContents::Header(translate!(
			o,
			StartImporting,
			"instance" = id,
			"format" = &format.info.id,
			"plugin" = &format.plugin
		)));

		// Create the target directory
		let target_dir = paths.data.join("instances").join(id);
		std::fs::create_dir_all(&target_dir)
			.context("Failed to create directory for new instance")?;

		let source_path = if source_path.exists() {
			source_path.canonicalize()?.to_string_lossy().to_string()
		} else {
			source_path.to_string_lossy().to_string()
		};

		// Import using the plugin
		let arg = ImportInstanceArg {
			format: format.info.id.clone(),
			id: id.to_string(),
			source_path,
			result_path: target_dir.to_string_lossy().to_string(),
			side,
			minecraft_versions: minecraft_versions.to_vec(),
		};
		let result = plugins
			.call_hook_on_plugin(ImportInstance, &format.plugin, &arg, paths, o)
			.await
			.context("Failed to import instance using plugin")?;

		let Some(result) = result else {
			o.debug(MessageContents::Error(
				o.translate(TranslationKey::ImportPluginNoResult).into(),
			));

			bail!("Import plugin did not return a result");
		};

		let mut result = result.result(o).await?;
		o.debug(MessageContents::Success(
			o.translate(TranslationKey::FinishImporting).into(),
		));

		result.config.imported = true;

		Ok(result.config)
	}
}

/// Migrates all instances from another launcher using a plugin
pub async fn migrate_instances(
	format: &str,
	instances: Option<Vec<String>>,
	link: bool,
	formats: &Formats,
	plugins: &PluginManager,
	paths: &Paths,
	o: &mut impl NitroOutput,
) -> anyhow::Result<HashMap<String, InstanceConfig>> {
	let format = formats
		.formats
		.get(format)
		.context("Transfer format does not exist")?;

	let migrate_info = format
		.info
		.migrate
		.as_ref()
		.context("This format or the plugin providing it does not support migration")?;

	output_support_warnings(migrate_info, o);

	o.display(MessageContents::StartProcess(translate!(
		o,
		StartMigrating,
		"format" = &format.info.id,
		"plugin" = &format.plugin
	)));

	let arg = MigrateInstancesArg {
		format: format.info.id.clone(),
		instances,
		link,
	};
	let result = plugins
		.call_hook_on_plugin(MigrateInstances, &format.plugin, &arg, paths, o)
		.await
		.context("Failed to import instances using plugin")?;

	let Some(result) = result else {
		o.debug(MessageContents::Error(
			o.translate(TranslationKey::ImportPluginNoResult).into(),
		));

		bail!("Migration plugin did not return a result");
	};

	let mut result = result.result(o).await?;
	o.display(MessageContents::Success(
		o.translate(TranslationKey::FinishMigrating).into(),
	));

	for inst in result.instances.values_mut() {
		inst.imported = true;
	}

	Ok(result.instances)
}

/// Load transfer formats from plugins
pub async fn load_formats(
	plugins: &PluginManager,
	paths: &Paths,
	o: &mut impl NitroOutput,
) -> anyhow::Result<Formats> {
	let mut results = plugins
		.call_hook(AddInstanceTransferFormats, &(), paths, o)
		.await
		.context("Failed to get transfer formats from plugins")?;
	let mut formats = HashMap::with_capacity(results.len());
	while let Some(handle) = results.next() {
		let plugin_id = handle.get_id().to_owned();
		let result = handle.result(o).await?;
		for result in result {
			formats.insert(
				result.id.clone(),
				Format {
					plugin: plugin_id.clone(),
					info: result,
				},
			);
		}
	}

	Ok(Formats { formats })
}

/// Represents loaded transfer formats from plugins
pub struct Formats {
	/// Map of the format IDs to the formats themselves
	formats: HashMap<String, Format>,
}

impl Formats {
	/// Iterate over the names of the loaded formats
	pub fn iter_format_names(&self) -> impl Iterator<Item = &String> {
		self.formats.keys()
	}
}

/// A single loaded transfer format
pub struct Format {
	/// The plugin that provides this format
	plugin: String,
	/// Information about the format
	info: InstanceTransferFormat,
}

/// Output warnings about unsupported features in the transfer
fn output_support_warnings(info: &InstanceTransferFormatDirection, o: &mut impl NitroOutput) {
	for (support, name) in [
		(
			info.launch_settings,
			TranslationKey::TransferLaunchSettingsFeature,
		),
		(info.modloader, TranslationKey::TransferModloaderFeature),
		(info.mods, TranslationKey::TransferModsFeature),
	] {
		let feat = o.translate(name);
		match support {
			InstanceTransferFeatureSupport::Supported => {}
			InstanceTransferFeatureSupport::FormatUnsupported => {
				o.display(MessageContents::Warning(translate!(
					o,
					TransferFeatureUnsupportedByFormat,
					"feat" = feat
				)))
			}
			InstanceTransferFeatureSupport::PluginUnsupported => {
				o.display(MessageContents::Warning(translate!(
					o,
					TransferFeatureUnsupportedByPlugin,
					"feat" = feat
				)))
			}
		}
	}
}
