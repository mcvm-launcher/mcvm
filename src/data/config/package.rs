use std::fmt::Display;

use mcvm_pkg::PackageContentType;
use mcvm_shared::pkg::PackageStability;
use serde::{Deserialize, Serialize};

use crate::{
	package::{
		eval::EvalPermissions,
		PkgProfileConfig,
	},
	util::merge_options,
};
use mcvm_pkg::{PkgRequest, PkgRequestSource};

/// Trick enum used to make deserialization work in the way we want
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageType {
	Local,
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
#[serde(rename_all = "snake_case")]
pub enum FullPackageConfig {
	Local {
		r#type: PackageType,
		id: String,
		version: u32,
		#[serde(default)]
		content_type: PackageContentType,
		path: String,
		#[serde(default)]
		features: Vec<String>,
		#[serde(default = "use_default_features_default")]
		use_default_features: bool,
		#[serde(default)]
		permissions: EvalPermissions,
		#[serde(default)]
		stability: Option<PackageStability>,
	},
	Repository {
		id: String,
		version: Option<u32>,
		#[serde(default)]
		features: Vec<String>,
		#[serde(default = "use_default_features_default")]
		use_default_features: bool,
		#[serde(default)]
		permissions: EvalPermissions,
		#[serde(default)]
		stability: Option<PackageStability>,
	},
}

/// Default value for use_default_features
fn use_default_features_default() -> bool {
	true
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum PackageConfig {
	Basic(String),
	Full(FullPackageConfig),
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
				req: PkgRequest::new(id, PkgRequestSource::UserRequire),
				features: vec![],
				use_default_features: true,
				permissions: EvalPermissions::Standard,
				stability: profile_stability,
			},
			PackageConfig::Full(FullPackageConfig::Local {
				r#type: _,
				id,
				version: _,
				path: _,
				content_type: _,
				features,
				use_default_features,
				permissions,
				stability,
			}) => PkgProfileConfig {
				req: PkgRequest::new(id, PkgRequestSource::UserRequire),
				features: features.clone(),
				use_default_features: *use_default_features,
				permissions: *permissions,
				stability: merge_options(Some(profile_stability), stability.to_owned()).unwrap(),
			},
			PackageConfig::Full(FullPackageConfig::Repository {
				id,
				version: _,
				features,
				use_default_features,
				permissions,
				stability,
			}) => PkgProfileConfig {
				req: PkgRequest::new(id, PkgRequestSource::UserRequire),
				features: features.clone(),
				use_default_features: *use_default_features,
				permissions: *permissions,
				stability: merge_options(Some(profile_stability), stability.to_owned()).unwrap(),
			},
		};

		Ok(package)
	}
}
