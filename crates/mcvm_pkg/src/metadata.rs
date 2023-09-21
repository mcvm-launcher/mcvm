use crate::parse::{instruction::InstrKind, parse::Parsed, routine::METADATA_ROUTINE};
use anyhow::bail;
use schemars::JsonSchema;
use serde::Deserialize;

/// Package metadata derived from running the 'meta' routine
#[derive(Default, Debug, Deserialize, Clone, JsonSchema)]
pub struct PackageMetadata {
	/// The name of the package
	pub name: Option<String>,
	/// The short description of the package
	pub description: Option<String>,
	/// The long description of the package
	pub long_description: Option<String>,
	/// The authors of the package content
	pub authors: Option<Vec<String>>,
	/// The maintainers of the package file
	pub package_maintainers: Option<Vec<String>>,
	/// The package's website
	pub website: Option<String>,
	/// The package's support page
	pub support_link: Option<String>,
	/// The package's documentation link
	pub documentation: Option<String>,
	/// The package's source repository
	pub source: Option<String>,
	/// The package's issues tracker
	pub issues: Option<String>,
	/// The package's online community
	pub community: Option<String>,
	/// A link to the package's icon
	pub icon: Option<String>,
	/// A link to the package's banner
	pub banner: Option<String>,
	/// Links to gallery images for the package
	pub gallery: Option<Vec<String>>,
	/// The license of the package
	pub license: Option<String>,
	/// The keywords for the package
	pub keywords: Option<Vec<String>>,
	/// The categories for the package
	pub categories: Option<Vec<String>>,
}

impl PackageMetadata {
	/// Check the validity of the metadata
	pub fn check_validity(&self) -> anyhow::Result<()> {
		Ok(())
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
