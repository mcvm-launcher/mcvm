mod core;
pub mod eval;
pub mod reg;
pub mod repo;

use crate::io::files::paths::Paths;
use crate::net::download;
use mcvm_shared::later::Later;

use std::fs;
use std::path::PathBuf;

use self::core::get_core_package;
use self::eval::EvalPermissions;
use self::reg::PkgRequest;
use anyhow::{anyhow, ensure, Context};
use mcvm_parse::metadata::{eval_metadata, PackageMetadata};
use mcvm_parse::parse::{lex_and_parse, Parsed};
use mcvm_parse::properties::{eval_properties, PackageProperties};
use mcvm_shared::pkg::{PackageStability, PkgIdentifier};
use reqwest::Client;

static PKG_EXTENSION: &str = ".pkg.txt";

/// Data pertaining to the contents of a package
#[derive(Debug)]
pub struct PkgData {
	contents: String,
	parsed: Later<Parsed>,
	metadata: Later<PackageMetadata>,
	properties: Later<PackageProperties>,
}

impl PkgData {
	pub fn new(contents: &str) -> Self {
		Self {
			contents: contents.to_owned(),
			parsed: Later::new(),
			metadata: Later::new(),
			properties: Later::new(),
		}
	}

	pub fn get_contents(&self) -> String {
		self.contents.clone()
	}
}

/// Location of a package
#[derive(Debug, Clone)]
pub enum PkgLocation {
	Local(PathBuf),         // Contained on the local filesystem
	Remote(Option<String>), // Contained on an external repository
	Core,                   // Included in the binary
}

/// An installable package that loads content into your game
#[derive(Debug)]
pub struct Package {
	pub id: PkgIdentifier,
	pub location: PkgLocation,
	pub data: Later<PkgData>,
}

impl Package {
	pub fn new(name: &str, version: u32, location: PkgLocation) -> Self {
		Self {
			id: PkgIdentifier::new(name, version),
			location,
			data: Later::new(),
		}
	}

	/// Get the cached file name of the package
	pub fn filename(&self) -> String {
		format!(
			"{}_{}{PKG_EXTENSION}",
			self.id.name.clone(),
			self.id.version
		)
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
					let contents = get_core_package(&self.id.name)
						.ok_or(anyhow!("Package is not a core package"))?;
					self.data.fill(PkgData::new(contents));
				}
			};
		}
		Ok(())
	}

	/// Parse the contents of the package
	pub async fn parse(&mut self, paths: &Paths, client: &Client) -> anyhow::Result<()> {
		self.ensure_loaded(paths, false, client).await?;
		let data = self.data.get_mut();
		if !data.parsed.is_empty() {
			return Ok(());
		}

		let parsed = lex_and_parse(&data.contents)?;

		data.parsed.fill(parsed);

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
		let parsed = data.parsed.get();
		if data.metadata.is_empty() {
			let metadata = eval_metadata(parsed).context("Failed to evaluate metadata")?;
			data.metadata.fill(metadata);
		}
		Ok(data.metadata.get())
	}

	/// Get the properties of the package
	pub async fn get_properties<'a>(
		&'a mut self,
		paths: &Paths,
		client: &Client,
	) -> anyhow::Result<&'a PackageProperties> {
		self.parse(paths, client).await.context("Failed to parse")?;
		let data = self.data.get_mut();
		let parsed = data.parsed.get();
		if data.properties.is_empty() {
			let properties = eval_properties(parsed).context("Failed to evaluate properties")?;
			data.properties.fill(properties);
		}
		Ok(data.properties.get())
	}
}

/// Evaluated configuration for a package, stored in a profile
#[derive(Debug)]
pub struct PkgProfileConfig {
	pub req: PkgRequest,
	pub features: Vec<String>,
	pub use_default_features: bool,
	pub permissions: EvalPermissions,
	pub stability: PackageStability,
}

/// Collect the final set of features for a package
pub fn calculate_features(
	config: &PkgProfileConfig,
	properties: &PackageProperties,
) -> anyhow::Result<Vec<String>> {
	let allowed_features = properties.features.clone().unwrap_or_default();
	let default_features = properties.default_features.clone().unwrap_or_default();

	for feature in &config.features {
		ensure!(
			allowed_features.contains(feature),
			"Configured feature '{feature}' does not exist"
		);
	}

	let mut out = Vec::new();
	if config.use_default_features {
		out.extend(default_features);
	}
	out.extend(config.features.clone());

	Ok(out)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_package_name() {
		let package = Package::new("sodium", 2, PkgLocation::Remote(None));
		assert_eq!(package.filename(), String::from("sodium_2") + PKG_EXTENSION);

		let package = Package::new("fabriclike-api", 80, PkgLocation::Remote(None));
		assert_eq!(
			package.filename(),
			String::from("fabriclike-api_80") + PKG_EXTENSION
		);
	}
}
