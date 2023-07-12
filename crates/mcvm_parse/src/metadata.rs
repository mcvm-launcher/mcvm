use crate::{instruction::InstrKind, parse::Parsed, routine::METADATA_ROUTINE};
use anyhow::bail;

/// Package metadata derived from running the 'meta' routine
#[derive(Default, Debug)]
pub struct PackageMetadata {
	pub name: Option<String>,
	pub description: Option<String>,
	pub long_description: Option<String>,
	pub version: Option<String>,
	pub authors: Option<Vec<String>>,
	pub package_maintainers: Option<Vec<String>>,
	pub website: Option<String>,
	pub support_link: Option<String>,
	pub documentation: Option<String>,
	pub source: Option<String>,
	pub issues: Option<String>,
	pub community: Option<String>,
	pub icon: Option<String>,
	pub banner: Option<String>,
	pub license: Option<String>,
}

/// Collect the metadata from a package
pub fn eval_metadata(parsed: &Parsed) -> anyhow::Result<PackageMetadata> {
	if let Some(routine_id) = parsed.routines.get(METADATA_ROUTINE) {
		if let Some(block) = parsed.blocks.get(routine_id) {
			let mut out = PackageMetadata::default();

			for instr in &block.contents {
				match &instr.kind {
					InstrKind::Name(val) => out.name = val.clone(),
					InstrKind::Description(val) => out.description = val.clone(),
					InstrKind::LongDescription(val) => out.long_description = val.clone(),
					InstrKind::Version(val) => out.version = val.clone(),
					InstrKind::Authors(val) => out.authors = Some(val.clone()),
					InstrKind::PackageMaintainers(val) => {
						out.package_maintainers = Some(val.clone())
					}
					InstrKind::Website(val) => out.website = val.clone(),
					InstrKind::SupportLink(val) => out.support_link = val.clone(),
					InstrKind::Documentation(val) => out.documentation = val.clone(),
					InstrKind::Source(val) => out.source = val.clone(),
					InstrKind::Issues(val) => out.issues = val.clone(),
					InstrKind::Community(val) => out.community = val.clone(),
					InstrKind::Icon(val) => out.icon = val.clone(),
					InstrKind::Banner(val) => out.banner = val.clone(),
					InstrKind::License(val) => out.license = val.clone(),
					_ => bail!("Instruction is not allowed in this context"),
				}
			}

			Ok(out)
		} else {
			Ok(PackageMetadata::default())
		}
	} else {
		Ok(PackageMetadata::default())
	}
}
