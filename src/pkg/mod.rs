/// Core packages that are built into the binary
mod core;
/// Package evaluation functions
pub mod eval;
/// Registry used to store packages
pub mod reg;
/// Interacting with package repositories
pub mod repo;

use crate::io::files::paths::Paths;
use mcvm_core::net::download;
use mcvm_pkg::declarative::{deserialize_declarative_package, DeclarativePackage};
use mcvm_pkg::repo::PackageFlag;
use mcvm_pkg::PackageContentType;
use mcvm_shared::later::Later;

use std::collections::HashSet;
use std::fs;
use std::future::Future;
use std::path::PathBuf;

use self::core::get_core_package;
use anyhow::{anyhow, bail, Context};
use mcvm_parse::parse::{lex_and_parse, Parsed};
use mcvm_pkg::metadata::{eval_metadata, PackageMetadata};
use mcvm_pkg::properties::{eval_properties, PackageProperties};
use mcvm_shared::pkg::PackageID;
use reqwest::Client;

/// An installable package that loads content into your game
#[derive(Debug)]
pub struct Package {
	/// The package ID
	pub id: PackageID,
	/// Where the package is being retrieved from
	pub location: PkgLocation,
	/// Type of the content in the package
	pub content_type: PackageContentType,
	/// Flags for the package from the repository
	pub flags: HashSet<PackageFlag>,
	/// The data of the package
	pub data: Later<PkgData>,
}

/// Location of a package
#[derive(Debug, Clone)]
pub enum PkgLocation {
	/// Contained on the local filesystem
	Local(PathBuf),
	/// Contained on an external repository
	Remote(Option<String>),
	/// Included in the binary
	Core,
}

/// Data pertaining to the contents of a package
#[derive(Debug)]
pub struct PkgData {
	text: String,
	contents: Later<PkgContents>,
	metadata: Later<PackageMetadata>,
	properties: Later<PackageProperties>,
}

impl PkgData {
	/// Create a new PkgData
	pub fn new(text: &str) -> Self {
		Self {
			text: text.to_owned(),
			contents: Later::new(),
			metadata: Later::new(),
			properties: Later::new(),
		}
	}

	/// Get the text content of the PkgData
	pub fn get_text(&self) -> String {
		self.text.clone()
	}
}

/// Type of data inside a package
#[derive(Debug)]
pub enum PkgContents {
	/// A package script
	Script(Parsed),
	/// A declarative package
	Declarative(Box<DeclarativePackage>),
}

impl PkgContents {
	/// Get the contents with an assertion that it is a script package
	pub fn get_script_contents(&self) -> &Parsed {
		if let Self::Script(parsed) = &self {
			parsed
		} else {
			panic!("Attempted to get script package contents from a non-script package");
		}
	}

	/// Get the contents with an assertion that it is a declarative package
	pub fn get_declarative_contents(&self) -> &DeclarativePackage {
		if let Self::Declarative(contents) = &self {
			contents
		} else {
			panic!("Attempted to get declarative package contents from a non-declarative package");
		}
	}
}

impl Package {
	/// Create a new Package
	pub fn new(
		id: PackageID,
		location: PkgLocation,
		content_type: PackageContentType,
		flags: HashSet<PackageFlag>,
	) -> Self {
		Self {
			id,
			location,
			data: Later::new(),
			content_type,
			flags,
		}
	}

	/// Get the cached file name of the package
	pub fn filename(&self) -> String {
		let extension = match self.content_type {
			PackageContentType::Declarative => ".json",
			PackageContentType::Script => ".pkg.txt",
		};
		format!("{}{extension}", self.id)
	}

	/// Get the cached path of the package
	pub fn cached_path(&self, paths: &Paths) -> PathBuf {
		let cache_dir = paths.project.cache_dir().join("pkg");
		cache_dir.join(self.filename())
	}

	/// Remove the cached package file
	pub fn remove_cached(&self, paths: &Paths) -> anyhow::Result<()> {
		let path = self.cached_path(paths);
		if path.exists() {
			fs::remove_file(path)?;
		}
		Ok(())
	}

	/// Ensure the raw contents of the package
	pub async fn ensure_loaded(
		&mut self,
		paths: &Paths,
		force: bool,
		client: &Client,
	) -> anyhow::Result<()> {
		if self.data.is_empty() {
			match &self.location {
				PkgLocation::Local(path) => {
					if !path.exists() {
						bail!("Local package path does not exist");
					}
					self.data
						.fill(PkgData::new(&tokio::fs::read_to_string(path).await?));
				}
				PkgLocation::Remote(url) => {
					let path = self.cached_path(paths);
					if !force && path.exists() {
						self.data
							.fill(PkgData::new(&tokio::fs::read_to_string(path).await?));
					} else {
						let url = url.as_ref().expect("URL for remote package missing");
						let text = download::text(url, client).await?;
						tokio::fs::write(&path, &text).await?;
						self.data.fill(PkgData::new(&text));
					}
				}
				PkgLocation::Core => {
					let contents = get_core_package(&self.id)
						.ok_or(anyhow!("Package is not a core package"))?;
					self.data.fill(PkgData::new(contents));
				}
			};
		}
		Ok(())
	}

	/// Returns a task that download's the package file if necessary. This will not
	/// update the contents and is only useful when doing repo resyncs
	pub fn get_download_task(
		&self,
		paths: &Paths,
		force: bool,
		client: &Client,
	) -> Option<impl Future<Output = anyhow::Result<()>> + 'static> {
		if let PkgLocation::Remote(url) = &self.location {
			let path = self.cached_path(paths);
			if force || !path.exists() {
				let url = url
					.as_ref()
					.expect("URL for remote package missing")
					.clone();
				let client = client.clone();
				return Some(async move { download::file(url, path, &client).await });
			}
		}

		None
	}

	/// Parse the contents of the package
	pub async fn parse(&mut self, paths: &Paths, client: &Client) -> anyhow::Result<()> {
		self.ensure_loaded(paths, false, client).await?;
		let data = self.data.get_mut();
		if data.contents.is_full() {
			return Ok(());
		}

		match self.content_type {
			PackageContentType::Script => {
				let parsed = lex_and_parse(&data.get_text())?;
				data.contents.fill(PkgContents::Script(parsed));
			}
			PackageContentType::Declarative => {
				let contents = deserialize_declarative_package(&data.get_text())
					.context("Failed to deserialize declarative package")?;
				data.contents
					.fill(PkgContents::Declarative(Box::new(contents)));
			}
		}

		Ok(())
	}

	/// Get the metadata of the package
	pub async fn get_metadata<'a>(
		&'a mut self,
		paths: &Paths,
		client: &Client,
	) -> anyhow::Result<&'a PackageMetadata> {
		self.parse(paths, client).await.context("Failed to parse")?;
		let data = self.data.get_mut();
		match self.content_type {
			PackageContentType::Script => {
				let parsed = data.contents.get().get_script_contents();
				if data.metadata.is_empty() {
					let metadata = eval_metadata(parsed).context("Failed to evaluate metadata")?;
					data.metadata.fill(metadata);
				}
				Ok(data.metadata.get())
			}
			PackageContentType::Declarative => {
				let contents = data.contents.get().get_declarative_contents();
				Ok(&contents.meta)
			}
		}
	}

	/// Get the properties of the package
	pub async fn get_properties<'a>(
		&'a mut self,
		paths: &Paths,
		client: &Client,
	) -> anyhow::Result<&'a PackageProperties> {
		self.parse(paths, client).await.context("Failed to parse")?;
		let data = self.data.get_mut();
		match self.content_type {
			PackageContentType::Script => {
				let parsed = data.contents.get().get_script_contents();
				if data.properties.is_empty() {
					let properties =
						eval_properties(parsed).context("Failed to evaluate properties")?;
					data.properties.fill(properties);
				}
				Ok(data.properties.get())
			}
			PackageContentType::Declarative => {
				let contents = data.contents.get().get_declarative_contents();
				Ok(&contents.properties)
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_package_id() {
		let package = Package::new(
			PackageID::from("sodium"),
			PkgLocation::Remote(None),
			PackageContentType::Script,
			HashSet::new(),
		);
		assert_eq!(package.filename(), "sodium.pkg.txt".to_string());

		let package = Package::new(
			PackageID::from("fabriclike-api"),
			PkgLocation::Remote(None),
			PackageContentType::Declarative,
			HashSet::new(),
		);
		assert_eq!(package.filename(), "fabriclike-api.json".to_string());
	}
}
