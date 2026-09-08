use anyhow::{Context, bail};
use nitro_instance::addon::Addon;
use nitro_instance::addon::storage::get_sha256_addon_path;
use nitro_instance::lock::LockfileAddon;
use nitro_pkg::addon::PackageAddon;
use nitro_shared::io::update_link;
use nitro_shared::manual_files::ManualFile;
use nitro_shared::minecraft::AddonKind;
use nitro_shared::pkg::AddonOptionalHashes;
use reqwest::Client;

use crate::io::paths::Paths;
use crate::util::hash::{get_best_hash, hash_file_with_best_hash};
use nitro_core::io::files::create_leading_dirs;
use nitro_core::net::download;

use std::future::Future;
use std::path::{Path, PathBuf};

/// Extension methods for addons that this crate uses
pub trait AddonExt {
	/// Get the addon directory where an addon is stored
	fn get_dir(&self, paths: &Paths) -> PathBuf;

	/// Get the path to this addon stored in the internal addons folder
	fn get_path(&self, paths: &Paths, instance_id: &str) -> PathBuf;

	/// Get a unique identifier for this addon
	fn get_unique_id(&self, instance_id: &str) -> String;

	/// Whether this addon has different behavior based on the filename
	fn filename_important(&self) -> bool;

	/// Split this addon's filename into base and extension
	fn split_filename(&self) -> (&str, &str);

	/// Whether this addon should be updated
	fn should_update(&self, paths: &Paths, instance_id: &str) -> bool;
}

impl AddonExt for PackageAddon {
	fn get_dir(&self, paths: &Paths) -> PathBuf {
		paths.addons.join(self.kind.to_plural_string())
	}

	fn get_path(&self, paths: &Paths, instance_id: &str) -> PathBuf {
		if let Some(hash) = &self.hashes.sha256 {
			return get_sha256_addon_path(&paths.addons, hash);
		}

		let pkg_dir = self
			.get_dir(paths)
			.join(self.pkg.to_string_no_version().replace(":", "_"));
		if let Some(version) = &self.version {
			pkg_dir.join(self.id.clone()).join(version)
		} else {
			pkg_dir.join(format!("{}_{instance_id}", self.id))
		}
	}

	fn get_unique_id(&self, instance_id: &str) -> String {
		if let Some(version) = &self.version {
			format!("{}_{instance_id}_{version}", self.id)
		} else {
			format!("{}_{instance_id}", self.id)
		}
	}

	fn filename_important(&self) -> bool {
		matches!(self.kind, AddonKind::ResourcePack)
	}

	fn split_filename(&self) -> (&str, &str) {
		if let Some(index) = self.file_name.find('.') {
			self.file_name.split_at(index)
		} else {
			(&self.file_name, "")
		}
	}

	fn should_update(&self, paths: &Paths, instance_id: &str) -> bool {
		self.version.is_none() || !self.get_path(paths, instance_id).exists()
	}
}

/// Gets the formulaic filename for an addon in the instance, meant to reduce name clashes
pub fn get_addon_instance_filename(package_id: &str, id: &str, kind: &AddonKind) -> String {
	format!("nitro_{package_id}_{id}{}", kind.get_extension())
}

/// Checks if this path is in the stored addons directory
pub fn is_stored_addon_path(path: &Path, paths: &Paths) -> bool {
	path.starts_with(&paths.addons)
}

/// The location of an addon
#[derive(Debug, Clone)]
pub enum AddonLocation {
	/// Located at a remote URL
	Remote(String),
	/// Located on the local filesystem
	Local(PathBuf),
}

/// A request for an addon file that will be fulfilled later
#[derive(Debug, Clone)]
pub struct AddonRequest {
	/// The addon that will be retrieved
	pub addon: PackageAddon,
	/// Where the addon is located
	location: AddonLocation,
}

impl AddonRequest {
	/// Create a new AddonRequest from an addon and location
	pub fn new(addon: PackageAddon, location: AddonLocation) -> Self {
		Self { addon, location }
	}

	/// Get a unique identifier for this addon
	pub fn get_unique_id(&self, instance_id: &str) -> String {
		self.addon.get_unique_id(instance_id)
	}

	/// Get the addon and store it
	pub async fn acquire(
		&self,
		paths: &Paths,
		instance_id: &str,
		client: &Client,
	) -> anyhow::Result<()> {
		let task = self.get_acquire_task(paths, instance_id, client);
		task.await.context("Failed to acquire addon")
	}

	/// Get the task to acquire the addon for use in concurrent operations
	pub fn get_acquire_task(
		&self,
		paths: &Paths,
		instance_id: &str,
		client: &Client,
	) -> impl Future<Output = anyhow::Result<()>> + Send + 'static {
		let path = self.addon.get_path(paths, instance_id);
		let _ = create_leading_dirs(&path);

		let id = self.addon.id.clone();
		let pkg = self.addon.pkg.clone();
		let location = self.location.clone();
		let client = client.clone();
		let hashes = self.addon.hashes.clone();
		let task = async move {
			match location {
				AddonLocation::Remote(url) => {
					if url.is_empty() {
						bail!("Empty URL for addon {id} from package {pkg}");
					}
					download::file(url, &path, &client)
						.await
						.context("Failed to download addon")?;
				}
				AddonLocation::Local(actual_path) => {
					update_link(&actual_path, &path).context("Failed to hardlink local addon")?;
				}
			}

			let result = Self::check_hashes_impl(hashes, &path);
			// Remove the addon file if it fails the checksum
			if result.is_err() {
				std::fs::remove_file(path).context("Failed to remove stored addon file")?;
			}
			result?;

			Ok(())
		};

		task
	}

	/// Gets the manual file associated with this addon, if it is one
	pub fn manual_file(&self) -> Option<ManualFile> {
		if !self.addon.is_manual {
			return None;
		}
		if let AddonLocation::Remote(url) = &self.location {
			Some(ManualFile {
				filename: self.addon.file_name.clone(),
				url: url.clone(),
				req: Some(self.addon.pkg.clone()),
			})
		} else {
			None
		}
	}

	/// Check the addon's hashes. The stored addon file must exist at this time
	pub fn check_hashes(&self, path: &Path) -> anyhow::Result<()> {
		Self::check_hashes_impl(self.addon.hashes.clone(), path)
	}

	/// Implementation for hash checking
	fn check_hashes_impl(hashes: AddonOptionalHashes, path: &Path) -> anyhow::Result<()> {
		let best_hash = get_best_hash(&hashes);
		if let Some(best_hash) = best_hash {
			let matches = hash_file_with_best_hash(path, best_hash)
				.context("Failed to checksum addon file")?;

			if !matches {
				bail!("Checksum for addon file does not match");
			}
		}

		Ok(())
	}
}

/// Package addon with target paths
pub struct ResolvedPackageAddon {
	/// Requested addon
	pub pkg_addon: PackageAddon,
	/// Target addon
	pub addon: Addon,
}

impl ResolvedPackageAddon {
	/// Converts to the addon format in the lockfile
	pub fn to_lockfile_addon(&self) -> LockfileAddon {
		LockfileAddon {
			id: Some(self.pkg_addon.id.clone()),
			package: Some(self.pkg_addon.pkg.to_string_no_version()),
			from_modpack: false,
			file_name: self.addon.file_name.clone(),
			files: self
				.addon
				.target_paths
				.iter()
				.map(|x| x.to_string_lossy().to_string())
				.collect(),
			kind: self.addon.kind,
			hashes: self.addon.hashes.clone(),
		}
	}
}

#[cfg(test)]
mod tests {
	use nitro_pkg::PkgRequest;
	use nitro_shared::pkg::AddonOptionalHashes;

	use super::*;

	#[test]
	fn test_addon_split_filename() {
		let addon = PackageAddon {
			kind: AddonKind::Mod,
			id: "foo".into(),
			file_name: "FooBar.baz.jar".into(),
			pkg: PkgRequest::parse("package", nitro_pkg::PkgRequestSource::UserRequire).arc(),
			version: None,
			modpack_format: None,
			hashes: AddonOptionalHashes::default(),
			is_manual: false,
		};
		assert_eq!(addon.split_filename(), ("FooBar", ".baz.jar"));
	}
}
