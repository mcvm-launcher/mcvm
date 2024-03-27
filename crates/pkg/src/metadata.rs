use crate::parse::{instruction::InstrKind, parse::Parsed, routine::METADATA_ROUTINE};
use anyhow::bail;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Nonessential display information about a package
#[derive(Default, Debug, Deserialize, Serialize, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct PackageMetadata {
	/// The name of the package
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	/// The short description of the package
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	/// The long description of the package
	#[serde(skip_serializing_if = "Option::is_none")]
	pub long_description: Option<String>,
	/// The authors of the package content
	#[serde(skip_serializing_if = "Option::is_none")]
	pub authors: Option<Vec<String>>,
	/// The maintainers of the package file
	#[serde(skip_serializing_if = "Option::is_none")]
	pub package_maintainers: Option<Vec<String>>,
	/// The package's website
	#[serde(skip_serializing_if = "Option::is_none")]
	pub website: Option<String>,
	/// The package's support page
	#[serde(skip_serializing_if = "Option::is_none")]
	pub support_link: Option<String>,
	/// The package's documentation link
	#[serde(skip_serializing_if = "Option::is_none")]
	pub documentation: Option<String>,
	/// The package's source repository
	#[serde(skip_serializing_if = "Option::is_none")]
	pub source: Option<String>,
	/// The package's issues tracker
	#[serde(skip_serializing_if = "Option::is_none")]
	pub issues: Option<String>,
	/// The package's online community
	#[serde(skip_serializing_if = "Option::is_none")]
	pub community: Option<String>,
	/// A link to the package's icon
	#[serde(skip_serializing_if = "Option::is_none")]
	pub icon: Option<String>,
	/// A link to the package's banner
	#[serde(skip_serializing_if = "Option::is_none")]
	pub banner: Option<String>,
	/// Links to gallery images for the package
	#[serde(skip_serializing_if = "Option::is_none")]
	pub gallery: Option<Vec<String>>,
	/// The license of the package
	#[serde(skip_serializing_if = "Option::is_none")]
	pub license: Option<String>,
	/// The keywords for the package
	#[serde(skip_serializing_if = "Option::is_none")]
	pub keywords: Option<Vec<String>>,
	/// The categories for the package
	#[serde(skip_serializing_if = "Option::is_none")]
	pub categories: Option<Vec<String>>,
}

impl PackageMetadata {
	/// Check the validity of the metadata
	pub fn check_validity(&self) -> anyhow::Result<()> {
		Ok(())
	}

	/// Check if all metadata fields are empty
	pub fn is_empty(&self) -> bool {
		self.name.is_none()
			&& self.description.is_none()
			&& self.long_description.is_none()
			&& self.authors.is_none()
			&& self.package_maintainers.is_none()
			&& self.website.is_none()
			&& self.support_link.is_none()
			&& self.documentation.is_none()
			&& self.source.is_none()
			&& self.issues.is_none()
			&& self.community.is_none()
			&& self.icon.is_none()
			&& self.banner.is_none()
			&& self.gallery.is_none()
			&& self.license.is_none()
			&& self.keywords.is_none()
			&& self.categories.is_none()
	}
}

/// Collect the metadata from a package script
pub fn eval_metadata(parsed: &Parsed) -> anyhow::Result<PackageMetadata> {
	if let Some(routine_id) = parsed.routines.get(METADATA_ROUTINE) {
		if let Some(block) = parsed.blocks.get(routine_id) {
			let mut out = PackageMetadata::default();

			for instr in &block.contents {
				match &instr.kind {
					InstrKind::Name(val) => out.name = Some(val.get_clone()),
					InstrKind::Description(val) => out.description = Some(val.get_clone()),
					InstrKind::LongDescription(val) => out.long_description = Some(val.get_clone()),
					InstrKind::Authors(val) => out.authors = Some(val.clone()),
					InstrKind::PackageMaintainers(val) => {
						out.package_maintainers = Some(val.clone())
					}
					InstrKind::Website(val) => out.website = Some(val.get_clone()),
					InstrKind::SupportLink(val) => out.support_link = Some(val.get_clone()),
					InstrKind::Documentation(val) => out.documentation = Some(val.get_clone()),
					InstrKind::Source(val) => out.source = Some(val.get_clone()),
					InstrKind::Issues(val) => out.issues = Some(val.get_clone()),
					InstrKind::Community(val) => out.community = Some(val.get_clone()),
					InstrKind::Icon(val) => out.icon = Some(val.get_clone()),
					InstrKind::Banner(val) => out.banner = Some(val.get_clone()),
					InstrKind::License(val) => out.license = Some(val.get_clone()),
					InstrKind::Keywords(val) => out.keywords = Some(val.clone()),
					InstrKind::Categories(val) => out.categories = Some(val.clone()),
					_ => bail!("Instruction is not allowed in this context"),
				}
			}

			out.check_validity()?;

			Ok(out)
		} else {
			Ok(PackageMetadata::default())
		}
	} else {
		Ok(PackageMetadata::default())
	}
}
