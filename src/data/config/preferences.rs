use std::path::PathBuf;

use crate::package::repo::{PkgRepo, PkgRepoLocation};
use crate::{net::download::validate_url, package::reg::CachingStrategy};

use anyhow::{bail, Context};
use mcvm_shared::lang::Language;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct RepoDeser {
	id: String,
	url: Option<String>,
	path: Option<String>,
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
			add_repo(&mut repositories, repo)?;
		}
		for repo in prefs.repositories.backup.iter() {
			add_repo(&mut repositories, repo)?;
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

/// Add a repo to the list
fn add_repo(repos: &mut Vec<PkgRepo>, repo: &RepoDeser) -> anyhow::Result<()> {
	let location = if let Some(url) = &repo.url {
		validate_url(url).with_context(|| {
			format!("Invalid url '{}' in package repository '{}'", url, repo.id)
		})?;
		PkgRepoLocation::Remote(url.clone())
	} else if let Some(path) = &repo.path {
		PkgRepoLocation::Local(PathBuf::from(path))
	} else {
		bail!("Nether path nor URL was set for repository {}", repo.id);
	};
	repos.push(PkgRepo::new(&repo.id, location));
	Ok(())
}
