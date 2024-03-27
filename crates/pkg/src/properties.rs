use anyhow::{bail, ensure};
use mcvm_parse::conditions::{ArchCondition, OSCondition};
use mcvm_shared::modifications::{ModloaderMatch, PluginLoaderMatch};
use mcvm_shared::versions::VersionPattern;
use mcvm_shared::Side;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::parse::{instruction::InstrKind, parse::Parsed, routine::PROPERTIES_ROUTINE};

/// Semantic properties and attributes of a package
#[derive(Default, Debug, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct PackageProperties {
	/// Available features that can be configured for the package
	#[serde(skip_serializing_if = "Option::is_none")]
	pub features: Option<Vec<String>>,
	/// Features enabled by default
	#[serde(skip_serializing_if = "Option::is_none")]
	pub default_features: Option<Vec<String>>,
	/// List of available content versions in order
	#[serde(skip_serializing_if = "Option::is_none")]
	pub content_versions: Option<Vec<String>>,
	/// The package's Modrinth ID
	#[serde(skip_serializing_if = "Option::is_none")]
	pub modrinth_id: Option<String>,
	/// The package's CurseForge ID
	#[serde(skip_serializing_if = "Option::is_none")]
	pub curseforge_id: Option<String>,
	/// The package's Smithed ID
	#[serde(skip_serializing_if = "Option::is_none")]
	pub smithed_id: Option<String>,
	/// The package's supported Minecraft versions
	#[serde(skip_serializing_if = "Option::is_none")]
	pub supported_versions: Option<Vec<VersionPattern>>,
	/// The package's supported modloaders
	#[serde(skip_serializing_if = "Option::is_none")]
	pub supported_modloaders: Option<Vec<ModloaderMatch>>,
	/// The package's supported plugin loaders
	#[serde(skip_serializing_if = "Option::is_none")]
	pub supported_plugin_loaders: Option<Vec<PluginLoaderMatch>>,
	/// The package's supported sides
	#[serde(skip_serializing_if = "Option::is_none")]
	pub supported_sides: Option<Vec<Side>>,
	/// The package's supported operating systems
	#[serde(skip_serializing_if = "Option::is_none")]
	pub supported_operating_systems: Option<Vec<OSCondition>>,
	/// The package's supported architectures
	#[serde(skip_serializing_if = "Option::is_none")]
	pub supported_architectures: Option<Vec<ArchCondition>>,
	/// The package's semantic tags
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tags: Option<Vec<String>>,
	/// Whether the package is open source
	#[serde(skip_serializing_if = "Option::is_none")]
	pub open_source: Option<bool>,
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

	/// Check if all properties are empty
	pub fn is_empty(&self) -> bool {
		self.features.is_none()
			&& self.default_features.is_none()
			&& self.modrinth_id.is_none()
			&& self.curseforge_id.is_none()
			&& self.smithed_id.is_none()
			&& self.supported_versions.is_none()
			&& self.supported_modloaders.is_none()
			&& self.supported_plugin_loaders.is_none()
			&& self.supported_sides.is_none()
			&& self.supported_operating_systems.is_none()
			&& self.supported_architectures.is_none()
			&& self.tags.is_none()
			&& self.open_source.is_none()
			&& self.content_versions.is_none()
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
					InstrKind::ContentVersions(list) => out.content_versions = Some(list.clone()),
					InstrKind::ModrinthID(id) => out.modrinth_id = Some(id.get_clone()),
					InstrKind::CurseForgeID(id) => out.curseforge_id = Some(id.get_clone()),
					InstrKind::SmithedID(id) => out.smithed_id = Some(id.get_clone()),
					InstrKind::SupportedVersions(list) => {
						out.supported_versions = Some(list.clone())
					}
					InstrKind::SupportedModloaders(list) => {
						out.supported_modloaders = Some(list.clone())
					}
					InstrKind::SupportedPluginLoaders(list) => {
						out.supported_plugin_loaders = Some(list.clone())
					}
					InstrKind::SupportedSides(list) => out.supported_sides = Some(list.clone()),
					InstrKind::SupportedOperatingSystems(list) => {
						out.supported_operating_systems = Some(list.clone())
					}
					InstrKind::SupportedArchitectures(list) => {
						out.supported_architectures = Some(list.clone())
					}
					InstrKind::Tags(list) => out.tags = Some(list.clone()),
					InstrKind::OpenSource(val) => out.open_source = Some(val.get_clone()),
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
