use crate::{package::repo::PkgRepo, net::download::validate_url};

use anyhow::Context;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SerRepo {
	id: String,
	url: String,
}

#[derive(Deserialize, Default)]
pub struct SerRepositories {
	#[serde(default)]
	pub preferred: Vec<SerRepo>,
	#[serde(default)]
	pub backup: Vec<SerRepo>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct PrefDeser {
	pub repositories: SerRepositories,
}

#[derive(Debug)]
pub struct ConfigPreferences {}

impl ConfigPreferences {
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
