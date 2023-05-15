use crate::{parse::Parsed, instruction::InstrKind};
use anyhow::{anyhow, bail};

/// Package metadata derived from running the 'meta' routine
#[derive(Default)]
pub struct PackageMetadata {
	pub name: Option<String>,
	pub description: Option<String>,
	pub version: Option<String>,
	pub authors: Option<Vec<String>>,
	pub website: Option<String>,
	pub support: Option<String>,
}

/// Collect the metadata from a package
pub fn eval_metadata(parsed: &Parsed) -> anyhow::Result<Option<PackageMetadata>> {
	let routine_name = "meta";
	let routine_id = parsed
		.routines
		.get(routine_name)
		.ok_or(anyhow!("Routine {} does not exist", routine_name))?;
	let block = parsed
		.blocks
		.get(routine_id)
		.ok_or(anyhow!("Routine {} does not exist", routine_name))?;

	let mut out = PackageMetadata { ..Default::default() };

	for instr in &block.contents {
		match &instr.kind {
			InstrKind::Name(val) => out.name = val.clone(),
			InstrKind::Description(val) => out.description = val.clone(),
			InstrKind::Version(val) => out.version = val.clone(),
			InstrKind::Authors(val) => out.authors = Some(val.clone()),
			InstrKind::Website(val) => out.website = val.clone(),
			InstrKind::Support(val) => out.support = val.clone(),
			_ => bail!("Instruction is not allowed in this context"),
		}
	}

	Ok(Some(out))
}
