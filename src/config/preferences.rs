use std::collections::HashSet;
use std::path::PathBuf;

use crate::{
	io::paths::Paths,
	pkg::repo::{
		basic::{BasicPackageRepository, RepoLocation},
		custom::CustomPackageRepository,
		PackageRepository,
	},
	plugin::PluginManager,
};
use mcvm_config::preferences::{PrefDeser, RepoDeser};
use mcvm_core::net::download::validate_url;

use anyhow::{bail, Context};
use mcvm_plugin::hooks::AddCustomPackageRepositories;
use mcvm_shared::{lang::Language, output::MCVMOutput};

/// Configured user preferences
#[derive(Debug)]
pub struct ConfigPreferences {
	/// The global language
	pub language: Language,
}

impl ConfigPreferences {
	/// Convert deserialized preferences to the stored format and returns
	/// a list of repositories to add.
	pub fn read(
		prefs: &PrefDeser,
		plugins: &PluginManager,
		paths: &Paths,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<(Self, Vec<PackageRepository>)> {
		let mut repositories = Vec::new();

		// Get repositories from plugins
		let mut preferred_plugin_repositories = Vec::new();
		let mut backup_plugin_repositories = Vec::new();
		let results = plugins
			.call_hook(AddCustomPackageRepositories, &(), paths, o)
			.context("Failed to call custom package repositories hook")?;
		for result in results {
			let plugin_id = result.get_id().clone();
			let results = result.result(o)?;
			for result in results {
				let repository = PackageRepository::Custom(CustomPackageRepository::new(
					result.id,
					plugin_id.clone(),
					result.metadata,
				));
				if result.is_preferred {
					preferred_plugin_repositories.push(repository);
				} else {
					backup_plugin_repositories.push(repository);
				}
			}
		}

		for repo in prefs.repositories.preferred.iter() {
			if !repo.disable {
				add_repo(&mut repositories, repo)?;
			}
		}
		repositories.extend(preferred_plugin_repositories);
		repositories.extend(PackageRepository::default_repos(
			prefs.repositories.enable_core,
			prefs.repositories.enable_std,
		));
		repositories.extend(backup_plugin_repositories);
		for repo in prefs.repositories.backup.iter() {
			if !repo.disable {
				add_repo(&mut repositories, repo)?;
			}
		}

		// Check for duplicate IDs
		let mut existing = HashSet::new();
		for repo in &repositories {
			if existing.contains(&repo.get_id()) {
				bail!("Duplicate repository ID '{}'", repo.get_id());
			}
			existing.insert(repo.get_id());
		}

		Ok((
			Self {
				language: prefs.language,
			},
			repositories,
		))
	}
}

/// Add a repo to the list
fn add_repo(repos: &mut Vec<PackageRepository>, repo: &RepoDeser) -> anyhow::Result<()> {
	let location = if let Some(url) = &repo.url {
		validate_url(url).with_context(|| {
			format!("Invalid url '{}' in package repository '{}'", url, repo.id)
		})?;
		RepoLocation::Remote(url.clone())
	} else if let Some(path) = &repo.path {
		RepoLocation::Local(PathBuf::from(path))
	} else {
		bail!("Niether path nor URL was set for repository {}", repo.id);
	};
	repos.push(PackageRepository::Basic(BasicPackageRepository::new(
		&repo.id, location,
	)));
	Ok(())
}
