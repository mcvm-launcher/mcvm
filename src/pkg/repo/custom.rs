use std::sync::Arc;

use anyhow::Context;
use mcvm_pkg::repo::RepoMetadata;
use mcvm_plugin::hooks::{QueryCustomPackageRepository, QueryCustomPackageRepositoryArg};
use mcvm_shared::output::MCVMOutput;

use crate::{io::paths::Paths, pkg::PkgLocation, plugin::PluginManager};

use super::RepoQueryResult;

/// A custom package repository from a plugin
pub struct CustomPackageRepository {
	/// The ID of this repository
	id: String,
	/// The plugin that added this repository and implements all of its functions
	plugin: String,
	/// The metadata for the repository
	meta: RepoMetadata,
}

impl CustomPackageRepository {
	/// Creates a new CustomPackageRepository
	pub fn new(id: String, plugin: String, metadata: RepoMetadata) -> Self {
		Self {
			id,
			plugin,
			meta: metadata,
		}
	}

	/// Queries this repository for a package
	pub fn query(
		&self,
		package: &str,
		plugins: &PluginManager,
		paths: &Paths,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Option<RepoQueryResult>> {
		let arg = QueryCustomPackageRepositoryArg {
			repository: self.id.clone(),
			package: package.to_string(),
		};
		let result = plugins
			.call_hook_on_plugin(QueryCustomPackageRepository, &self.plugin, &arg, paths, o)
			.context("Failed to call query hook")?;

		let Some(result) = result else {
			return Ok(None);
		};

		let result = result.result(o)?;

		Ok(result.map(|x| RepoQueryResult {
			location: PkgLocation::Inline(Arc::from(x.contents)),
			content_type: x.content_type,
			flags: x.flags,
		}))
	}

	/// Gets the ID for this repository
	pub fn get_id(&self) -> &str {
		&self.id
	}

	/// Gets the plugin ID for this repository
	pub fn get_plugin_id(&self) -> &str {
		&self.plugin
	}

	/// Gets the metadata for this repository
	pub fn get_meta(&self) -> &RepoMetadata {
		&self.meta
	}
}
