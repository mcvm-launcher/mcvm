use std::{collections::HashMap, path::Path};

use anyhow::Context;
use mcvm_plugin::hooks::{
	AddInstanceTransferFormat, ExportInstance, ExportInstanceArg, InstanceTransferFeatureSupport,
	InstanceTransferFormat, InstanceTransferFormatDirection,
};
use mcvm_shared::lang::translate::TranslationKey;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::translate;

use crate::{config::plugin::PluginManager, io::paths::Paths};

use super::Instance;

impl Instance {
	/// Export this instance using the given format
	pub fn export(
		&mut self,
		format: &str,
		result_path: &Path,
		formats: &Formats,
		plugins: &PluginManager,
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

		// TODO: Ensure that the instance has been created once from lockfile before allowing export
		self.ensure_dirs(paths)
			.context("Failed to ensure instance directories")?;

		// Export using the plugin
		let arg = ExportInstanceArg {
			id: self.id.to_string(),
			format: format.info.id.clone(),
			name: self.config.name.clone(),
			side: Some(self.get_side()),
			game_dir: self.dirs.get().game_dir.to_string_lossy().to_string(),
			result_path: result_path.to_string_lossy().to_string(),
		};
		let result = plugins
			.call_hook_on_plugin(ExportInstance, &format.plugin, &arg, paths, o)
			.context("Failed to export instance using plugin")?;

		if let Some(result) = result {
			result.result(o)?;
		} else {
			o.display(
				MessageContents::Error("Export plugin did not return a result".into()),
				MessageLevel::Debug,
			);
		}

		Ok(())
	}
}

/// Load transfer formats from plugins
pub fn load_formats(
	plugins: &PluginManager,
	paths: &Paths,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<Formats> {
	let results = plugins
		.call_hook(AddInstanceTransferFormat, &(), paths, o)
		.context("Failed to get transfer formats from plugins")?;
	let mut formats = HashMap::with_capacity(results.len());
	for result in results {
		let plugin_id = result.get_id().to_owned();
		let result = result.result(o)?;
		formats.insert(
			result.id.clone(),
			Format {
				plugin: plugin_id,
				info: result,
			},
		);
	}

	Ok(Formats { formats })
}

/// Represents loaded transfer formats from plugins
pub struct Formats {
	/// Map of the format IDs to the formats themselves
	formats: HashMap<String, Format>,
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
