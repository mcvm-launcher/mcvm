use mcvm_parse::{properties::PackageProperties, metadata::PackageMetadata, conditions::OSCondition};
use mcvm_shared::{
	instance::Side,
	lang::Language,
	modifications::{ModloaderMatch, PluginLoaderMatch},
	versions::VersionPattern, pkg::PackageStability,
};
use serde::Deserialize;

/// Package relationships for declarative packages
#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct DeclarativePackageRelations {
	pub dependencies: Vec<String>,
	pub explicit_dependencies: Vec<String>,
	pub conflicts: Vec<String>,
	pub bundled: Vec<String>,
	pub compats: Vec<(String, String)>,
	pub recommendations: Vec<String>,
}

/// Properties that are used for choosing the best addon version from a declarative package
#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct DeclarativeAddonVersionProperties {
	pub minecraft_versions: Option<Vec<VersionPattern>>,
	pub side: Option<Side>,
	pub modloaders: Option<Vec<ModloaderMatch>>,
	pub plugin_loaders: Option<Vec<PluginLoaderMatch>>,
	pub stability: PackageStability,
	pub features: Vec<String>,
	pub os: Option<OSCondition>,
	pub language: Option<Language>,
}

/// Properties for declarative addon versions that can be changed with patches
#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct DeclarativeAddonVersionPatch {
	pub relations: DeclarativePackageRelations,
	pub filename: String,
}

/// Version for an addon in a declarative package
#[derive(Deserialize, Debug)]
pub struct DeclarativeAddonVersion {
	#[serde(flatten)]
	pub properties: DeclarativeAddonVersionProperties,
	#[serde(flatten, default)]
	pub patch: DeclarativeAddonVersionPatch,
	#[serde(default)]
	pub path: Option<String>,
	#[serde(default)]
	pub url: Option<String>,
	#[serde(default)]
	pub version: String,
}

/// Addon in a declarative package
#[derive(Deserialize, Debug)]
pub struct DeclarativeAddon {
	pub id: String,
	pub versions: Vec<DeclarativeAddonVersion>,
}

/// Structure for a declarative / JSON package
#[derive(Deserialize, Debug, Default)]
pub struct DeclarativePackage {
	#[serde(default)]
	pub meta: PackageMetadata,
	#[serde(default)]
	pub properties: PackageProperties,
	pub addons: Vec<DeclarativeAddon>,
	#[serde(default)]
	pub relations: DeclarativePackageRelations,
}
