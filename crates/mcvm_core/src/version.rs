use std::collections::HashMap;

use anyhow::Context;
use mcvm_shared::output::MCVMOutput;
use mcvm_shared::output::{MessageContents, MessageLevel};

use crate::instance::{Instance, InstanceConfiguration, InstanceParameters};
use crate::io::files::paths::Paths;
use crate::io::persistent::PersistentData;
use crate::io::update::UpdateManager;
use crate::net::game_files::client_meta::{self, ClientMeta};
use crate::net::game_files::version_manifest::VersionManifestAndList;
use crate::user::UserManager;
use crate::util::versions::VersionName;

/// An installed version of the game. This cannot be constructed directly,
/// only from the MCVMCore struct by using the `install_version()` method
pub struct InstalledVersion<'inner, 'params> {
	pub(crate) inner: &'inner mut InstalledVersionInner,
	pub(crate) params: VersionParameters<'params>,
}

impl<'inner, 'params> InstalledVersion<'inner, 'params> {
	/// Get the version name
	pub fn get_version(&self) -> &VersionName {
		&self.inner.version
	}

	/// Get the client meta
	pub fn get_client_meta(&self) -> &ClientMeta {
		&self.inner.client_meta
	}

	/// Create an instance and its files using this version,
	/// ready to be launched
	pub async fn get_instance(
		&mut self,
		config: InstanceConfiguration,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Instance> {
		let params = InstanceParameters {
			version: &self.inner.version,
			version_manifest: self.params.version_manifest,
			paths: self.params.paths,
			req_client: self.params.req_client,
			persistent: self.params.persistent,
			update_manager: self.params.update_manager,
			client_meta: &self.inner.client_meta,
			users: self.params.users,
			client_assets_and_libs: &mut self.inner.client_assets_and_libs,
		};
		let instance = Instance::load(config, params, o)
			.await
			.context("Failed to load instance")?;
		Ok(instance)
	}
}

pub(crate) struct InstalledVersionInner {
	version: VersionName,
	client_meta: ClientMeta,
	client_assets_and_libs: ClientAssetsAndLibraries,
}

impl InstalledVersionInner {
	/// Load a version
	async fn load(
		version: VersionName,
		params: LoadVersionParameters<'_>,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<Self> {
		// Get the client meta
		o.start_process();
		o.display(
			MessageContents::StartProcess("Obtaining client metadata".into()),
			MessageLevel::Important,
		);

		let client_meta = client_meta::get(
			&version,
			&params.version_manifest.manifest,
			params.paths,
			params.update_manager,
			params.req_client,
		)
		.await
		.context("Failed to get client meta")?;

		o.display(
			MessageContents::Success("Client meta obtained".into()),
			MessageLevel::Important,
		);
		o.end_process();

		Ok(Self {
			version,
			client_meta,
			client_assets_and_libs: ClientAssetsAndLibraries::new(),
		})
	}
}

/// A registry of installed versions
pub(crate) struct VersionRegistry {
	versions: HashMap<VersionName, InstalledVersionInner>,
}

impl VersionRegistry {
	pub fn new() -> Self {
		Self {
			versions: HashMap::new(),
		}
	}

	/// Load a version if it is not already loaded, and get it otherwise
	pub async fn get_version(
		&mut self,
		version: &VersionName,
		params: LoadVersionParameters<'_>,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<&mut InstalledVersionInner> {
		if !self.versions.contains_key(version) {
			let installed_version = InstalledVersionInner::load(version.clone(), params, o).await?;
			self.versions.insert(version.clone(), installed_version);
		}
		Ok(self
			.versions
			.get_mut(version)
			.expect("Version should exist in map"))
	}
}

/// Container struct for parameters for versions and instances
pub(crate) struct VersionParameters<'a> {
	pub paths: &'a Paths,
	pub req_client: &'a reqwest::Client,
	pub persistent: &'a mut PersistentData,
	pub version_manifest: &'a VersionManifestAndList,
	pub update_manager: &'a mut UpdateManager,
	pub users: &'a mut UserManager,
}

/// Container struct for parameters for loading version innards
#[derive(Clone)]
pub(crate) struct LoadVersionParameters<'a> {
	pub paths: &'a Paths,
	pub req_client: &'a reqwest::Client,
	pub version_manifest: &'a VersionManifestAndList,
	pub update_manager: &'a UpdateManager,
}

/// Data for client assets and libraries that are only
/// loaded when a client needs them
pub(crate) struct ClientAssetsAndLibraries {
	loaded: bool,
}

impl ClientAssetsAndLibraries {
	pub fn new() -> Self {
		Self { loaded: false }
	}

	pub async fn load(
		&mut self,
		params: ClientAssetsAndLibsParameters<'_>,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		if self.loaded {
			return Ok(());
		}
		let result = crate::net::game_files::assets::get(
			params.client_meta,
			params.paths,
			params.version,
			&params.version_manifest.list,
			params.update_manager,
			params.req_client,
			o,
		)
		.await
		.context("Failed to get game assets")?;
		params.update_manager.add_result(result);

		let result = crate::net::game_files::libraries::get(
			params.client_meta,
			params.paths,
			params.version,
			params.update_manager,
			params.req_client,
			o,
		)
		.await
		.context("Failed to get game libraries")?;
		params.update_manager.add_result(result);

		self.loaded = true;
		Ok(())
	}
}

/// Container struct for parameters for loading client assets and libraries
pub(crate) struct ClientAssetsAndLibsParameters<'a> {
	pub client_meta: &'a ClientMeta,
	pub version: &'a VersionName,
	pub paths: &'a Paths,
	pub req_client: &'a reqwest::Client,
	pub version_manifest: &'a VersionManifestAndList,
	pub update_manager: &'a mut UpdateManager,
}
