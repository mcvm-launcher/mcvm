use crate::net::download::validate_url;
use crate::package::repo::PkgRepo;

use anyhow::Context;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RepoDeser {
	id: String,
	url: String,
}

#[derive(Deserialize, Default)]
pub struct RepositoriesDeser {
	#[serde(default)]
	preferred: Vec<RepoDeser>,
	#[serde(default)]
	backup: Vec<RepoDeser>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct PrefDeser {
	repositories: RepositoriesDeser,
}

#[derive(Debug)]
pub struct ConfigPreferences {}

impl ConfigPreferences {
	/// Convert deserialized preferences to the stored format and returns
	/// a list of repositories to add.
	pub fn read(prefs: &Option<PrefDeser>) -> anyhow::Result<(Self, Vec<PkgRepo>)> {
		match prefs {
			Some(prefs) => {
				let mut repositories = Vec::new();
				for repo in prefs.repositories.preferred.iter() {
					repositories.push(PkgRepo::new(&repo.id, &repo.url));
				}
				for repo in prefs.repositories.backup.iter() {
					repositories.push(PkgRepo::new(&repo.id, &repo.url));
				}

				for repo in prefs.repositories.preferred.iter().chain(prefs.repositories.backup.iter()) {
					validate_url(&repo.url)
						.with_context(|| format!("Invalid url '{}' in package repository '{}'", repo.url, repo.id))?;
				}

				Ok((Self {}, repositories))
			}
			None => Ok((Self {}, vec![])),
		}
	}
}
