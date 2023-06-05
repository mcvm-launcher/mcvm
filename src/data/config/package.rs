use std::fmt::Display;

use mcvm_shared::pkg::PackageStability;
use serde::{Deserialize, Serialize};

use crate::{
	package::{
		eval::EvalPermissions,
		reg::{PkgRequest, PkgRequestSource},
		PkgProfileConfig,
	},
	util::merge_options,
};

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
		path: String,
		#[serde(default)]
		features: Vec<String>,
		#[serde(default)]
		permissions: EvalPermissions,
		#[serde(default)]
		stability: Option<PackageStability>,
	},
	Remote {
		id: String,
		version: Option<u32>,
		#[serde(default)]
		features: Vec<String>,
		#[serde(default)]
		permissions: EvalPermissions,
		#[serde(default)]
		stability: Option<PackageStability>,
	},
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
				Self::Full(FullPackageConfig::Remote { id, .. }) => id,
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
				permissions: EvalPermissions::Standard,
				stability: profile_stability,
			},
			PackageConfig::Full(FullPackageConfig::Local {
				r#type: _,
				id,
				version: _,
				path: _,
				features,
				permissions,
				stability,
			}) => PkgProfileConfig {
				req: PkgRequest::new(id, PkgRequestSource::UserRequire),
				features: features.clone(),
				permissions: permissions.clone(),
				stability: merge_options(Some(profile_stability), stability.to_owned()).unwrap(),
			},
			PackageConfig::Full(FullPackageConfig::Remote {
				id,
				version: _,
				features,
				permissions,
				stability,
			}) => PkgProfileConfig {
				req: PkgRequest::new(id, PkgRequestSource::UserRequire),
				features: features.clone(),
				permissions: permissions.clone(),
				stability: merge_options(Some(profile_stability), stability.to_owned()).unwrap(),
			},
		};

		Ok(package)
	}
}
