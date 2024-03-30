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

/// Different representations for the configuration of a package
#[derive(Deserialize, Serialize, Clone, Debug)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum PackageConfig {
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

impl Display for PackageConfig {
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

impl PackageConfig {
	/// Get the package ID of the config
	pub fn get_pkg_id(&self) -> PackageID {
		match &self {
			Self::Basic(id) => id.clone(),
			Self::Full(cfg) => cfg.id.clone(),
		}
	}

	/// Get the request of the config
	pub fn get_request(&self) -> ArcPkgReq {
		let id = self.get_pkg_id();
		Arc::new(PkgRequest::parse(id, PkgRequestSource::UserRequire))
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

	/// Calculate the features of the config
	pub fn calculate_features(
		&self,
		properties: &PackageProperties,
	) -> anyhow::Result<Vec<String>> {
		let allowed_features = properties.features.clone().unwrap_or_default();
		let default_features = properties.default_features.clone().unwrap_or_default();

		let features = self.get_features();
		for feature in &features {
			ensure!(
				allowed_features.contains(feature),
				"Configured feature '{feature}' does not exist"
			);
		}

		let mut out = Vec::new();
		if self.get_use_default_features() {
			out.extend(default_features);
		}
		out.extend(features);

		Ok(out)
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
