use anyhow::bail;

use crate::{routine::PROPERTIES_ROUTINE, parse::Parsed};

/// Package properties derived from running the 'properties' routine
#[derive(Default, Debug)]
pub struct PackageProperties {
	
}

/// Collect the properties from a package
pub fn eval_properties(parsed: &Parsed) -> anyhow::Result<PackageProperties> {
	if let Some(routine_id) = parsed.routines.get(PROPERTIES_ROUTINE) {
		if let Some(block) = parsed.blocks.get(routine_id) {
			let out = PackageProperties::default();

			for instr in &block.contents {
				match &instr.kind {
					_ => bail!("Instruction is not allowed in this context"),
				}
			}

			Ok(out)
		} else {
			Ok(PackageProperties::default())
		}
	} else {
		Ok(PackageProperties::default())
	}
}
