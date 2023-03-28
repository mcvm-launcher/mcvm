use crate::package::repo::PkgRepo;

use anyhow::Context;
use serde::Deserialize;

#[derive(Deserialize)]
struct SerRepo {
	id: String,
	url: String,
}

#[derive(Deserialize, Default)]
struct SerRepositories {
	#[serde(default)]
	pub preferred: Vec<SerRepo>,
	#[serde(default)]
	pub backup: Vec<SerRepo>,
}

#[derive(Deserialize)]
struct PrefSerialize {
	#[serde(default)]
	pub repositories: SerRepositories,
}

#[derive(Debug)]
pub struct ConfigPreferences {}

impl ConfigPreferences {
	pub fn read(obj: Option<&serde_json::Value>) -> anyhow::Result<(Self, Vec<PkgRepo>)> {
		match obj {
			Some(obj) => {
				let prefs = serde_json::from_value::<PrefSerialize>(obj.clone())
					.context("Failed to parse preferences")?;
				let mut repositories = Vec::new();
				for repo in prefs.repositories.preferred {
					repositories.push(PkgRepo::new(&repo.id, &repo.url));
				}
				for repo in prefs.repositories.backup {
					repositories.push(PkgRepo::new(&repo.id, &repo.url));
				}

				Ok((Self {}, repositories))
			}
			None => Ok((Self {}, vec![])),
		}
	}
}
