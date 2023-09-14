use std::fmt::Display;

use mcvm_pkg::PackageContentType;
use mcvm_shared::pkg::PackageStability;
use serde::{Deserialize, Serialize};

use crate::package::{eval::EvalPermissions, PkgProfileConfig};
use crate::util::merge_options;
use mcvm_pkg::{PkgRequest, PkgRequestSource};

/// Different representations for the configuration of a package
#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum PackageConfig {
	/// Basic configuration for a repository package with just the package ID
	Basic(String),
	/// Full configuration for a package
	Full(FullPackageConfig),
}

/// Full configuration for a package
#[derive(Deserialize, Serialize)]
#[serde(untagged)]
#[serde(rename_all = "snake_case")]
pub enum FullPackageConfig {
	/// Config for a local package
	Local {
		/// The type of the package
		r#type: PackageType,
		/// The ID of the pcakage
		id: String,
		/// The package's content type
		#[serde(default)]
		content_type: PackageContentType,
		/// The path to the local package
		path: String,
		/// The package's enabled features
		#[serde(default)]
		features: Vec<String>,
		/// Whether or not to use the package's default features
		#[serde(default = "use_default_features_default")]
		use_default_features: bool,
		/// Permissions for the package
		#[serde(default)]
		permissions: EvalPermissions,
		/// Expected stability for the package
		#[serde(default)]
		stability: Option<PackageStability>,
	},
	/// Config for a repository package
	Repository {
		/// The ID of the pcakage
		id: String,
		#[serde(default)]
		/// The package's enabled features
		features: Vec<String>,
		/// Whether or not to use the package's default features
		#[serde(default = "use_default_features_default")]
		use_default_features: bool,
		/// Permissions for the package
		#[serde(default)]
		permissions: EvalPermissions,
		/// Expected stability for the package
		#[serde(default)]
		stability: Option<PackageStability>,
	},
}

/// Trick enum used to make deserialization work in the way we want
#[derive(Deserialize, Serialize)]
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
				Self::Full(FullPackageConfig::Local { id, .. }) => id,
				Self::Full(FullPackageConfig::Repository { id, .. }) => id,
			}
		)
	}
}

impl PackageConfig {
	/// Convert this package config into a PkgProfileConfig
	pub fn to_profile_config(
		&self,
		profile_stability: PackageStability,
	) -> anyhow::Result<PkgProfileConfig> {
		let package = match self {
			PackageConfig::Basic(id) => PkgProfileConfig {
				req: PkgRequest::new(id.clone(), PkgRequestSource::UserRequire),
				features: vec![],
				use_default_features: true,
				permissions: EvalPermissions::Standard,
				stability: profile_stability,
			},
			PackageConfig::Full(FullPackageConfig::Local {
				r#type: _,
				id,
				path: _,
				content_type: _,
				features,
				use_default_features,
				permissions,
				stability,
			}) => PkgProfileConfig {
				req: PkgRequest::new(id.clone(), PkgRequestSource::UserRequire),
				features: features.clone(),
				use_default_features: *use_default_features,
				permissions: *permissions,
				stability: merge_options(Some(profile_stability), stability.to_owned()).unwrap(),
			},
			PackageConfig::Full(FullPackageConfig::Repository {
				id,
				features,
				use_default_features,
				permissions,
				stability,
			}) => PkgProfileConfig {
				req: PkgRequest::new(id.clone(), PkgRequestSource::UserRequire),
				features: features.clone(),
				use_default_features: *use_default_features,
				permissions: *permissions,
				stability: merge_options(Some(profile_stability), stability.to_owned()).unwrap(),
			},
		};

		Ok(package)
	}
}
