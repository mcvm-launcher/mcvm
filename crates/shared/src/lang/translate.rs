use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::Language;

macro_rules! define_translations {
	($($key:ident, $doc:literal, $default:literal);* $(;)?) => {
		/// Keys for translations
		#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
		#[serde(rename_all = "snake_case")]
		pub enum TranslationKey {
			$(
				#[doc = $doc]
				$key,
			)*
		}

		impl TranslationKey {
			/// Get the default translation for this key
			pub fn get_default(&self) -> &'static str {
				match self {
					$(
						Self::$key => $default,
					)*
				}
			}
		}
	};
}

define_translations! {
	StartResolvingDependencies, "When starting to resolve package dependencies", "Resolving package dependencies";
	FinishResolvingDependencies, "When finishing resolving package dependencies", "Dependencies resolved";
	StartAcquiringAddons, "When starting to acquire addons", "Acquiring addons";
	FinishAcquiringAddons, "When finishing acquiring addons", "Addons acquired";
	StartInstallingPackages, "When starting to install packages", "Installing packages";
	FinishInstallingPackage, "When finishing installing a single package", "Package installed";
	PackageOutOfDate, "When a package is out of date", "Package %pkg has been flagged as out of date";
	PackageDeprecated, "When a package is deprecated", "Package %pkg has been flagged as deprecated";
	PackageInsecure, "When a package is insecure", "Package %pkg has been flagged as insecure";
	PackageMalicious, "When a package is malicious", "Package %pkg has been flagged as malicious";
	PackageSupportHeader, "Header for packages to consider supporting", "Packages to consider supporting";
	StartUpdatingProfile, "When starting to update a profile", "Updating profile '%prof'";
	StartUpdatingPackages, "When starting to update packages on a profile", "Updating packages";
	FinishUpdatingPackages, "When finishing updating packages on a profile", "All packages installed";
	StartUpdatingProfileVersion, "When starting to update a profile's version", "Updating profile version";
	FinishUpdatingProfileVersion, "When finishing updating a profile's version", "Profile version updated";
	StartUpdatingProxy, "When starting to update a proxy", "Checking for proxy updates";
	FinishUpdatingProxy, "When finishing updating a proxy", "Proxy updated";
	StartRunningCommands, "When starting to run package commands", "Running commands";
	FinishRunningCommands, "When finishing running package commands", "Finished running commands";
	StartUpdatingInstance, "When starting to update an instance", "Checking for updates";
	Launch, "When launching the game", "Launching!";
	CoreRepoName, "Name of the core repo", "Core";
	CoreRepoDescription, "Description of the core repo", "The built-in set of packages";
	RepoVersionWarning, "Warning when a remote repo version is too high", "Minimum MCVM version for repository %repo is higher than current installation";
	OverwriteAddonFilePrompt, "Prompt when an addon file would be overwriten", "The existing file '%file' has the same path as an addon. Overwrite it?";
	CustomInstructionsWarning, "When a package uses unrecognized custom instructions", "Package uses custom instructions that MCVM does not recognize";
	Redownloading, "When something is being redownloaded", "Redownloading";
	AssetIndexFailed, "When the asset index is unable to be obtained", "Failed to obtain asset index";
	StartDownloadingAssets, "When starting to download assets", "Downloading %count assets";
	FinishDownloadingAssets, "When finishing downloading assets", "Assets downloaded";
	DownloadedAsset, "When an asset finishes downloading", "Downloaded asset %asset";
	DownloadingClientMeta, "While the client meta is downloading", "Downloading client meta";
	StartDownloadingLibraries, "When starting to download libraries", "Downloading %count libraries";
	FinishDownloadingLibraries, "When finishing downloading libraries", "Libraries downloaded";
	DownloadedLibrary, "When a library finishes downloading", "Downloaded library %lib";
	StartExtractingNative, "When a native library starts to extract", "Extracting native library %lib";
	ExtractedNativeFile, "When a native library file extract is extracted", "Extracted native file %file";
	NoDefaultUser, "When users are available but no default is set", "Users are available but no default user is set";
	NoUsers, "When no users are available", "No users are available";
	ModificationNotSupported, "When a game modification can't be installed by MCVM", "%mod installation is currently unimplemented by mcvm. You will be expected to install it yourself for the time being";
	EmptyProfile, "When a profile has no instances", "Profile '%profile' does not have any instances";
}

/// Replaces placeholders in a translated key
pub fn replace_placeholders(string: &str, placeholder_name: &str, value: &str) -> String {
	string.replace(&format!("%{placeholder_name}"), value)
}

/// Utility macro to translate from output
#[macro_export]
macro_rules! translate {
	($o:expr, $key:ident) => {
		$o.translate(TranslationKey::$key).into()
	};

	($o:expr, $key:ident, $($placeholder:literal = $value:expr),+) => {
		{
			let mut out = $o.translate(TranslationKey::$key).to_string();
			$(
				out = out.replace(&format!("%{}", $placeholder), $value);
			)+
			out
		}
	};
}

/// A translation map of translation keys to their translations
pub type TranslationMap = HashMap<TranslationKey, String>;
/// A map of languages to translation maps
pub type LanguageMap = HashMap<Language, TranslationMap>;
