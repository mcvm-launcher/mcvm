use std::sync::Arc;

use anyhow::Context;
use mcvm_pkg::{repo::RepoMetadata, PkgRequest};
use mcvm_plugin::hooks::{
	QueryCustomPackageRepository, QueryCustomPackageRepositoryArg, SearchCustomPackageRepository,
	SearchCustomPackageRepositoryArg,
};
use mcvm_shared::{
	output::MCVMOutput,
	pkg::{ArcPkgReq, PackageSearchParameters},
};

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

	/// Searches this repository for packages
	pub fn search(
		&self,
		params: PackageSearchParameters,
		plugins: &PluginManager,
		paths: &Paths,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Vec<String>> {
		let arg = SearchCustomPackageRepositoryArg {
			repository: self.id.clone(),
			parameters: params,
		};
		let result = plugins
			.call_hook_on_plugin(SearchCustomPackageRepository, &self.plugin, &arg, paths, o)
			.context("Failed to call search hook")?;

		let Some(result) = result else {
			return Ok(Vec::new());
		};

		result.result(o)
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
