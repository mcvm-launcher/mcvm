use crate::package::repo::PkgRepo;
use crate::{net::download::validate_url, package::reg::CachingStrategy};

use anyhow::Context;
use mcvm_shared::lang::Language;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct RepoDeser {
	id: String,
	url: String,
}

#[derive(Deserialize, Serialize, Default)]
pub struct RepositoriesDeser {
	#[serde(default)]
	preferred: Vec<RepoDeser>,
	#[serde(default)]
	backup: Vec<RepoDeser>,
}

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct PrefDeser {
	repositories: RepositoriesDeser,
	package_caching_strategy: CachingStrategy,
	language: Language,
}

#[derive(Debug)]
pub struct ConfigPreferences {
	pub package_caching_strategy: CachingStrategy,
	pub language: Language,
}

impl ConfigPreferences {
	/// Convert deserialized preferences to the stored format and returns
	/// a list of repositories to add.
	pub fn read(prefs: &PrefDeser) -> anyhow::Result<(Self, Vec<PkgRepo>)> {
		let mut repositories = Vec::new();
		for repo in prefs.repositories.preferred.iter() {
			repositories.push(PkgRepo::new(&repo.id, &repo.url));
		}
		for repo in prefs.repositories.backup.iter() {
			repositories.push(PkgRepo::new(&repo.id, &repo.url));
		}

		for repo in prefs
			.repositories
			.preferred
			.iter()
			.chain(prefs.repositories.backup.iter())
		{
			validate_url(&repo.url).with_context(|| {
				format!(
					"Invalid url '{}' in package repository '{}'",
					repo.url, repo.id
				)
			})?;
		}

		Ok((
			Self {
				package_caching_strategy: prefs.package_caching_strategy.clone(),
				language: prefs.language,
			},
			repositories,
		))
	}
}
