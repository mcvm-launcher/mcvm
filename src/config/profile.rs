use std::collections::HashMap;

use anyhow::bail;
use mcvm_config::profile::ProfileConfig;
use mcvm_shared::id::ProfileID;

/// Consolidates profile configs into the full profiles
pub fn consolidate_profile_configs(
	profiles: HashMap<ProfileID, ProfileConfig>,
	global_profile: Option<&ProfileConfig>,
) -> anyhow::Result<HashMap<ProfileID, ProfileConfig>> {
	let mut out: HashMap<_, ProfileConfig> = HashMap::with_capacity(profiles.len());

	let max_iterations = 10000;

	// We do this by repeatedly finding a profile with an already resolved ancenstor
	let mut i = 0;
	while out.len() != profiles.len() {
		for (id, profile) in &profiles {
			// Don't redo profiles that are already done
			if out.contains_key(id) {
				continue;
			}

			if profile.instance.common.from.is_empty() {
				// Profiles with no ancestor can just be added directly to the output, after deriving from the global profile
				let mut profile = profile.clone();
				if let Some(global_profile) = global_profile {
					let overlay = profile;
					profile = global_profile.clone();
					profile.merge(overlay);
				}
				out.insert(id.clone(), profile);
			} else {
				for parent in profile.instance.common.from.iter() {
					// If the parent is already in the map (already consolidated) then we can derive from it and add to the map
					if let Some(parent) = out.get(&ProfileID::from(parent.clone())) {
						let mut new = parent.clone();
						new.merge(profile.clone());
						out.insert(id.clone(), new);
					} else {
						bail!("Parent profile '{parent}' does not exist, or cyclic profiles were found");
					}
				}
			}
		}

		i += 1;
		if i > max_iterations {
			panic!("Max iterations exceeded while resolving profiles. This is a bug in MCVM.");
		}
	}

	Ok(out)
}
