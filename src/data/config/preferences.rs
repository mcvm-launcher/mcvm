use std::collections::HashSet;
use std::path::PathBuf;

use crate::package::reg::CachingStrategy;
use crate::package::repo::{PkgRepo, PkgRepoLocation};
use mcvm_core::net::download::validate_url;

use anyhow::{bail, Context};
use mcvm_shared::lang::Language;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Configured user preferences
#[derive(Debug)]
pub struct ConfigPreferences {
	/// Caching strategy for packages
	pub package_caching_strategy: CachingStrategy,
	/// The global language
	pub language: Language,
}

/// Deserialization struct for user preferences
#[derive(Deserialize, Serialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct PrefDeser {
	/// The user's configured repositories
	pub repositories: RepositoriesDeser,
	/// The user's configured strategy for package caching
	pub package_caching_strategy: CachingStrategy,
	/// The user's configured language
	pub language: Language,
}

/// Deserialization struct for a package repo
#[derive(Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct RepoDeser {
	/// The ID of the repository
	pub id: String,
	/// The URL to the repository, which may not exist
	pub url: Option<String>,
	/// The Path to the repository, which may not exist
	pub path: Option<String>,
}

/// Deserialization struct for all configured package repositories
#[derive(Deserialize, Serialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct RepositoriesDeser {
	/// The preferred repositories over the default ones
	#[serde(default)]
	pub preferred: Vec<RepoDeser>,
	/// The backup repositories included after the default ones
	#[serde(default)]
	pub backup: Vec<RepoDeser>,
}

impl ConfigPreferences {
	/// Convert deserialized preferences to the stored format and returns
	/// a list of repositories to add.
	pub fn read(prefs: &PrefDeser) -> anyhow::Result<(Self, Vec<PkgRepo>)> {
		let mut repositories = Vec::new();
		for repo in prefs.repositories.preferred.iter() {
			add_repo(&mut repositories, repo)?;
		}
		repositories.extend(get_default_repos());
		for repo in prefs.repositories.backup.iter() {
			add_repo(&mut repositories, repo)?;
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

/// Get the default set of repositories
fn get_default_repos() -> Vec<PkgRepo> {
	vec![PkgRepo::new(
		"std",
		PkgRepoLocation::Remote("https://carbonsmasher.github.io/mcvm/std".into()),
	)]
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
