use std::borrow::Cow;
use std::fmt::Display;
use std::sync::Arc;

use anyhow::{bail, ensure};
use mcvm_pkg::properties::PackageProperties;
use mcvm_shared::pkg::{is_valid_package_id, ArcPkgReq, PackageID, PackageStability};
use mcvm_shared::util::is_valid_identifier;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pkg::eval::EvalPermissions;
use mcvm_pkg::{PkgRequest, PkgRequestSource};

/// Stored configuration for a package
#[derive(Clone, Debug)]
pub struct PackageConfig {
	/// The ID of the pcakage
	pub id: PackageID,
	/// The source of where the package was configured
	pub source: PackageConfigSource,
	/// The package's enabled features
	pub features: Vec<String>,
	/// Whether or not to use the package's default features
	pub use_default_features: bool,
	/// Permissions for the package
	pub permissions: EvalPermissions,
	/// Expected stability for the package
	pub stability: PackageStability,
	/// Worlds to use for the package
	pub worlds: Vec<String>,
}

impl PackageConfig {
	/// Create the default configuration for a package with a package ID
	pub fn from_id(id: PackageID) -> Self {
		Self {
			id,
			source: PackageConfigSource::Instance,
			features: Vec::new(),
			use_default_features: use_default_features_default(),
			permissions: EvalPermissions::default(),
			stability: PackageStability::default(),
			worlds: Vec::new(),
		}
	}

	/// Calculate the features of the config
	pub fn calculate_features(
		&self,
		properties: &PackageProperties,
	) -> anyhow::Result<Vec<String>> {
		let empty = Vec::new();
		let allowed_features = properties.features.as_ref().unwrap_or(&empty);

		for feature in &self.features {
			ensure!(
				allowed_features.contains(feature),
				"Configured feature '{feature}' does not exist"
			);
		}

		let mut out = self.features.clone();
		if self.use_default_features {
			let default_features = properties.default_features.clone().unwrap_or_default();
			out.extend(default_features);
		}

		Ok(out)
	}

	/// Get the request of the config
	pub fn get_request(&self) -> ArcPkgReq {
		Arc::new(PkgRequest::parse(
			self.id.clone(),
			PkgRequestSource::UserRequire,
		))
	}
}

/// Where a package was configured from
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageConfigSource {
	/// Configured globally for all profiles and instances
	Global,
	/// Configured for one profile and all of it's instances
	Profile,
	/// Configured for one instance
	Instance,
}

/// Different representations for the configuration of a package in deserialization
#[derive(Deserialize, Serialize, Clone, Debug)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum PackageConfigDeser {
	/// Basic configuration for a repository package with just the package ID
	Basic(PackageID),
	/// Full configuration for a package
	Full(FullPackageConfig),
}

/// Full configuration for a package
#[derive(Deserialize, Serialize, Clone, Debug)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct FullPackageConfig {
	/// The ID of the pcakage
	pub id: PackageID,
	#[serde(default)]
	/// The package's enabled features
	pub features: Vec<String>,
	/// Whether or not to use the package's default features
	#[serde(default = "use_default_features_default")]
	pub use_default_features: bool,
	/// Permissions for the package
	#[serde(default)]
	pub permissions: EvalPermissions,
	/// Expected stability for the package
	#[serde(default)]
	pub stability: Option<PackageStability>,
	/// Worlds to use for the package
	#[serde(default)]
	pub worlds: Vec<String>,
}

/// Trick enum used to make deserialization work in the way we want
#[derive(Deserialize, Serialize, Clone, Debug)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum PackageType {
	/// Yeah this is kinda stupid
	Local,
}

/// Default value for use_default_features
fn use_default_features_default() -> bool {
	true
}

impl Display for PackageConfigDeser {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Basic(id) => id,
				Self::Full(FullPackageConfig { id, .. }) => id,
			}
		)
	}
}

impl PackageConfigDeser {
	/// Convert this deserialized config into the actual package config
	pub fn to_package_config(
		self,
		profile_stability: PackageStability,
		source: PackageConfigSource,
	) -> PackageConfig {
		PackageConfig {
			id: self.get_pkg_id(),
			source,
			features: self.get_features(),
			use_default_features: self.get_use_default_features(),
			permissions: self.get_permissions(),
			stability: self.get_stability(profile_stability),
			worlds: self.get_worlds().into_owned(),
		}
	}

	/// Get the package ID of the config
	pub fn get_pkg_id(&self) -> PackageID {
		match &self {
			Self::Basic(id) => id.clone(),
			Self::Full(cfg) => cfg.id.clone(),
		}
	}

	/// Get the features of the config
	pub fn get_features(&self) -> Vec<String> {
		match &self {
			Self::Basic(..) => Vec::new(),
			Self::Full(cfg) => cfg.features.clone(),
		}
	}

	/// Get the use_default_features option of the config
	pub fn get_use_default_features(&self) -> bool {
		match &self {
			Self::Basic(..) => use_default_features_default(),
			Self::Full(cfg) => cfg.use_default_features,
		}
	}

	/// Get the permissions of the config
	pub fn get_permissions(&self) -> EvalPermissions {
		match &self {
			Self::Basic(..) => EvalPermissions::Standard,
			Self::Full(cfg) => cfg.permissions,
		}
	}

	/// Get the stability of the config
	pub fn get_stability(&self, profile_stability: PackageStability) -> PackageStability {
		match &self {
			Self::Basic(..) => profile_stability,
			Self::Full(cfg) => cfg.stability.unwrap_or(profile_stability),
		}
	}

	/// Get the  worlds of the config
	pub fn get_worlds(&self) -> Cow<[String]> {
		match &self {
			Self::Basic(..) => Cow::Owned(Vec::new()),
			Self::Full(cfg) => Cow::Borrowed(&cfg.worlds),
		}
	}

	/// Validate this config
	pub fn validate(&self) -> anyhow::Result<()> {
		let id = self.get_pkg_id();
		if !is_valid_package_id(&id) {
			bail!("Invalid package ID '{id}'");
		}

		for feature in self.get_features() {
			if !is_valid_identifier(&feature) {
				bail!("Invalid string '{feature}'");
			}
		}

		Ok(())
	}
}
