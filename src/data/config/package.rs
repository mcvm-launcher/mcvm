use std::borrow::Cow;
use std::fmt::Display;
use std::sync::Arc;

use anyhow::{bail, ensure};
use mcvm_pkg::properties::PackageProperties;
use mcvm_pkg::PackageContentType;
use mcvm_shared::pkg::{is_valid_package_id, ArcPkgReq, PackageID, PackageStability};
use mcvm_shared::util::is_valid_identifier;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::package::eval::EvalPermissions;
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
#[serde(untagged)]
#[serde(rename_all = "snake_case")]
pub enum FullPackageConfig {
	/// Config for a local package
	Local {
		/// The type of the package
		r#type: PackageType,
		/// The ID of the pcakage
		id: PackageID,
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
		/// Worlds to use for the package
		#[serde(default)]
		worlds: Vec<String>,
	},
	/// Config for a repository package
	Repository {
		/// The ID of the pcakage
		id: PackageID,
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
		/// Worlds to use for the package
		#[serde(default)]
		worlds: Vec<String>,
	},
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
				Self::Full(FullPackageConfig::Local { id, .. }) => id,
				Self::Full(FullPackageConfig::Repository { id, .. }) => id,
			}
		)
	}
}

impl PackageConfig {
	/// Get the package ID of the config
	pub fn get_pkg_id(&self) -> PackageID {
		match &self {
			Self::Basic(id) => id.clone(),
			Self::Full(cfg) => match cfg {
				FullPackageConfig::Local { id, .. } => id.clone(),
				FullPackageConfig::Repository { id, .. } => id.clone(),
			},
		}
	}

	/// Get the request of the config
	pub fn get_request(&self) -> ArcPkgReq {
		let id = self.get_pkg_id();
		Arc::new(PkgRequest::new(id.clone(), PkgRequestSource::UserRequire))
	}

	/// Get the features of the config
	pub fn get_features(&self) -> Vec<String> {
		match &self {
			Self::Basic(..) => Vec::new(),
			Self::Full(cfg) => match cfg {
				FullPackageConfig::Local { features, .. } => features.clone(),
				FullPackageConfig::Repository { features, .. } => features.clone(),
			},
		}
	}

	/// Get the use_default_features option of the config
	pub fn get_use_default_features(&self) -> bool {
		match &self {
			Self::Basic(..) => use_default_features_default(),
			Self::Full(cfg) => match cfg {
				FullPackageConfig::Local {
					use_default_features,
					..
				} => *use_default_features,
				FullPackageConfig::Repository {
					use_default_features,
					..
				} => *use_default_features,
			},
		}
	}

	/// Get the permissions of the config
	pub fn get_permissions(&self) -> EvalPermissions {
		match &self {
			Self::Basic(..) => EvalPermissions::Standard,
			Self::Full(cfg) => match cfg {
				FullPackageConfig::Local { permissions, .. } => *permissions,
				FullPackageConfig::Repository { permissions, .. } => *permissions,
			},
		}
	}

	/// Get the stability of the config
	pub fn get_stability(&self, profile_stability: PackageStability) -> PackageStability {
		match &self {
			Self::Basic(..) => profile_stability,
			Self::Full(cfg) => match cfg {
				FullPackageConfig::Local { stability, .. } => {
					stability.unwrap_or(profile_stability)
				}
				FullPackageConfig::Repository { stability, .. } => {
					stability.unwrap_or(profile_stability)
				}
			},
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
			Self::Full(cfg) => match cfg {
				FullPackageConfig::Local { worlds, .. } => Cow::Borrowed(worlds),
				FullPackageConfig::Repository { worlds, .. } => Cow::Borrowed(worlds),
			},
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
