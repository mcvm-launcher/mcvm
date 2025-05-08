use std::{collections::HashMap, path::Path};

use anyhow::{bail, Context};
use mcvm_config::instance::InstanceConfig;
use mcvm_plugin::hooks::{
	AddInstanceTransferFormats, ExportInstance, ExportInstanceArg, ImportInstance,
	ImportInstanceArg, InstanceTransferFeatureSupport, InstanceTransferFormat,
	InstanceTransferFormatDirection,
};
use mcvm_shared::lang::translate::TranslationKey;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::translate;

use crate::io::lock::Lockfile;
use crate::{io::paths::Paths, plugin::PluginManager};

use super::Instance;

impl Instance {
	/// Export this instance using the given format
	pub fn export(
		&mut self,
		format: &str,
		result_path: &Path,
		formats: &Formats,
		plugins: &PluginManager,
		lock: &Lockfile,
		paths: &Paths,
		o: &mut impl MCVMOutput,
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

		self.ensure_dirs(paths)
			.context("Failed to ensure instance directories")?;

		o.display(
			MessageContents::StartProcess(translate!(
				o,
				StartExporting,
				"instance" = &self.id,
				"format" = &format.info.id,
				"plugin" = &format.plugin
			)),
			MessageLevel::Important,
		);

		let lock_instance = lock
			.get_instance(&self.id)
			.context("Instance does not exist in lockfile. Try updating it before exporting.")?;

		// Export using the plugin
		let arg = ExportInstanceArg {
			id: self.id.to_string(),
			format: format.info.id.clone(),
			config: self.config.original_config_with_profiles.clone(),
			minecraft_version: lock_instance.version.clone(),
			game_modification_version: lock_instance.game_modification_version.clone(),
			game_dir: self.dirs.get().game_dir.to_string_lossy().to_string(),
			result_path: result_path.to_string_lossy().to_string(),
		};
		let result = plugins
			.call_hook_on_plugin(ExportInstance, &format.plugin, &arg, paths, o)
			.context("Failed to export instance using plugin")?;

		if let Some(result) = result {
			result.result(o)?;
			o.display(
				MessageContents::Success(o.translate(TranslationKey::FinishExporting).into()),
				MessageLevel::Important,
			);
		} else {
			o.display(
				MessageContents::Error(o.translate(TranslationKey::ExportPluginNoResult).into()),
				MessageLevel::Debug,
			);
		}

		Ok(())
	}

	/// Import an instance using the given format. Returns an InstanceConfig to add to the config file
	pub fn import(
		id: &str,
		format: &str,
		source_path: &Path,
		formats: &Formats,
		plugins: &PluginManager,
		paths: &Paths,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<InstanceConfig> {
		// Get and print info about the format
		let format = formats
			.formats
			.get(format)
			.context("Transfer format does not exist")?;

		let import_info = format
			.info
			.import
			.as_ref()
			.context("This format or the plugin providing it does not support importing")?;

		output_support_warnings(import_info, o);

		o.display(
			MessageContents::StartProcess(translate!(
				o,
				StartImporting,
				"instance" = id,
				"format" = &format.info.id,
				"plugin" = &format.plugin
			)),
			MessageLevel::Important,
		);

		// Create the target directory
		let target_dir = paths.project.data_dir().join("instances").join(id);
		std::fs::create_dir_all(&target_dir)
			.context("Failed to create directory for new instance")?;

		// Import using the plugin
		let arg = ImportInstanceArg {
			format: format.info.id.clone(),
			id: id.to_string(),
			source_path: source_path.to_string_lossy().to_string(),
			result_path: target_dir.to_string_lossy().to_string(),
		};
		let result = plugins
			.call_hook_on_plugin(ImportInstance, &format.plugin, &arg, paths, o)
			.context("Failed to import instance using plugin")?;

		let Some(result) = result else {
			o.display(
				MessageContents::Error(o.translate(TranslationKey::ImportPluginNoResult).into()),
				MessageLevel::Debug,
			);

			bail!("Import plugin did not return a result");
		};

		let result = result.result(o)?;
		o.display(
			MessageContents::Success(o.translate(TranslationKey::FinishImporting).into()),
			MessageLevel::Important,
		);

		Ok(result.config)
	}
}

/// Load transfer formats from plugins
pub fn load_formats(
	plugins: &PluginManager,
	paths: &Paths,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<Formats> {
	let results = plugins
		.call_hook(AddInstanceTransferFormats, &(), paths, o)
		.context("Failed to get transfer formats from plugins")?;
	let mut formats = HashMap::with_capacity(results.len());
	for result in results {
		let plugin_id = result.get_id().to_owned();
		let result = result.result(o)?;
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
fn output_support_warnings(info: &InstanceTransferFormatDirection, o: &mut impl MCVMOutput) {
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
			InstanceTransferFeatureSupport::FormatUnsupported => o.display(
				MessageContents::Warning(translate!(
					o,
					TransferFeatureUnsupportedByFormat,
					"feat" = feat
				)),
				MessageLevel::Important,
			),
			InstanceTransferFeatureSupport::PluginUnsupported => o.display(
				MessageContents::Warning(translate!(
					o,
					TransferFeatureUnsupportedByPlugin,
					"feat" = feat
				)),
				MessageLevel::Important,
			),
		}
	}
}
