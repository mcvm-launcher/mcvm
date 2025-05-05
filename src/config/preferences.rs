use std::collections::HashSet;
use std::path::PathBuf;

use crate::pkg::repo::{PkgRepo, PkgRepoLocation};
use mcvm_config::preferences::{CachingStrategy, PrefDeser, RepoDeser};
use mcvm_core::net::download::validate_url;

use anyhow::{bail, Context};
use mcvm_shared::lang::Language;

/// Configured user preferences
#[derive(Debug)]
pub struct ConfigPreferences {
	/// Caching strategy for packages
	pub package_caching_strategy: CachingStrategy,
	/// The global language
	pub language: Language,
}

impl ConfigPreferences {
	/// Convert deserialized preferences to the stored format and returns
	/// a list of repositories to add.
	pub fn read(prefs: &PrefDeser) -> anyhow::Result<(Self, Vec<PkgRepo>)> {
		let mut repositories = Vec::new();
		for repo in prefs.repositories.preferred.iter() {
			if !repo.disable {
				add_repo(&mut repositories, repo)?;
			}
		}
		repositories.extend(PkgRepo::default_repos(
			prefs.repositories.enable_core,
			prefs.repositories.enable_std,
		));
		for repo in prefs.repositories.backup.iter() {
			if !repo.disable {
				add_repo(&mut repositories, repo)?;
			}
		}

		// Check for duplicate IDs
		let mut existing = HashSet::new();
		for repo in &repositories {
			if existing.contains(&repo.id) {
				bail!("Duplicate repository ID '{}'", repo.id);
			}
			existing.insert(&repo.id);
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
		bail!("Niether path nor URL was set for repository {}", repo.id);
	};
	repos.push(PkgRepo::new(&repo.id, location));
	Ok(())
}
