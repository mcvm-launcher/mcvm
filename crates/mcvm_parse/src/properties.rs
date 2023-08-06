use anyhow::{bail, ensure};
use mcvm_shared::{
	instance::Side,
	modifications::{ModloaderMatch, PluginLoaderMatch},
};
use serde::Deserialize;

use crate::{instruction::InstrKind, parse::Parsed, routine::PROPERTIES_ROUTINE};

/// Package properties derived from running the 'properties' routine
#[derive(Default, Debug, Deserialize, Clone)]
pub struct PackageProperties {
	pub features: Option<Vec<String>>,
	pub default_features: Option<Vec<String>>,
	pub modrinth_id: Option<String>,
	pub curseforge_id: Option<String>,
	pub supported_modloaders: Option<Vec<ModloaderMatch>>,
	pub supported_plugin_loaders: Option<Vec<PluginLoaderMatch>>,
	pub supported_sides: Option<Vec<Side>>,
}

impl PackageProperties {
	/// Check the validity of the properties
	pub fn check_validity(&self) -> anyhow::Result<()> {
		// Validate features
		if let Some(default_features) = &self.default_features {
			if let Some(features) = &self.features {
				for feature in default_features {
					ensure!(
						features.contains(feature),
						"Default feature '{feature}' does not exist"
					);
				}
			}
		}

		Ok(())
	}
}

/// Collect the properties from a package script
pub fn eval_properties(parsed: &Parsed) -> anyhow::Result<PackageProperties> {
	if let Some(routine_id) = parsed.routines.get(PROPERTIES_ROUTINE) {
		if let Some(block) = parsed.blocks.get(routine_id) {
			let mut out = PackageProperties::default();

			for instr in &block.contents {
				match &instr.kind {
					InstrKind::Features(list) => out.features = Some(list.clone()),
					InstrKind::DefaultFeatures(list) => out.default_features = Some(list.clone()),
					InstrKind::ModrinthID(id) => out.modrinth_id = Some(id.get_clone()),
					InstrKind::CurseForgeID(id) => out.curseforge_id = Some(id.get_clone()),
					InstrKind::SupportedModloaders(list) => {
						out.supported_modloaders = Some(list.clone())
					}
					InstrKind::SupportedPluginLoaders(list) => {
						out.supported_plugin_loaders = Some(list.clone())
					}
					InstrKind::SupportedSides(list) => out.supported_sides = Some(list.clone()),
					_ => bail!("Instruction is not allowed in this context"),
				}
			}

			out.check_validity()?;

			Ok(out)
		} else {
			Ok(PackageProperties::default())
		}
	} else {
		Ok(PackageProperties::default())
	}
}
