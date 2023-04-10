use crate::package::repo::PkgRepo;

use anyhow::Context;
use reqwest::Url;
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
				for repo in prefs.repositories.preferred.iter() {
					repositories.push(PkgRepo::new(&repo.id, &repo.url));
				}
				for repo in prefs.repositories.backup.iter() {
					repositories.push(PkgRepo::new(&repo.id, &repo.url));
				}

				for repo in prefs.repositories.preferred.iter().chain(prefs.repositories.backup.iter()) {
					Url::parse(&repo.url)
						.with_context(|| format!("Invalid url '{}' in package repository '{}'", repo.url, repo.id))?;
				}

				Ok((Self {}, repositories))
			}
			None => Ok((Self {}, vec![])),
		}
	}
}
