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
	Notice, "Header for a notice", "Notice";
	Warning, "Header for a warning", "Warning";
	Error, "Header for an error", "Error";
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
	StartUpdatingPackages, "When starting to update packages on a profile", "Updating packages";
	FinishUpdatingPackages, "When finishing updating packages on a profile", "All packages installed";
	StartUpdatingProfileVersion, "When starting to update a profile's version", "Updating profile version";
	FinishUpdatingProfileVersion, "When finishing updating a profile's version", "Profile version updated";
	StartUpdatingProxy, "When starting to update a proxy", "Checking for proxy updates";
	FinishUpdatingProxy, "When finishing updating a proxy", "Proxy updated";
	StartRunningCommands, "When starting to run package commands", "Running commands";
	FinishRunningCommands, "When finishing running package commands", "Finished running commands";
	StartUpdatingInstance, "When starting to update an instance", "Updating instance %inst";
	PreparingLaunch, "When preparing to launch the game", "Preparing to launch";
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
	StartDownloadingVersionManifest, "When starting to download the version manifest", "Downloading version manifest";
	StartDownloadingGameJar, "When starting to download the game jar", "Downloading %side jar";
	FinishDownloadingGameJar, "When finishing downloading the game jar", "%side jar downloaded";
	StartCheckingForJavaUpdates, "When starting to check for Java updates", "Checking for Java updates";
	FinishCheckingForJavaUpdates, "When finishing checking for Java updates", "Java updated";
	FinishJavaInstallation, "When finishing installing Java", "Java installation finished";
	StartExtractingJava, "When starting to extract the JRE", "Extracting JRE";
	StartRemovingJavaArchive, "When starting to remove the Java archive", "Removing archive";
	DownloadingGraalVM, "When starting to download GraalVM", "Downloading GraalVM";
	DownloadingZulu, "When starting to download Zulu", "Downloading Azul Zulu JRE version %version";
	DownloadingAdoptium, "When starting to download Adoptium", "Downloading Adoptium Temurin JRE version %version";
	StartUpdatingClient, "When starting to update a client", "Updating client '%id'";
	StartUpdatingServer, "When starting to update a server", "Updating server '%id'";
	PasskeyAccepted, "When finishing decrypting with a passkey", "Passkey accepted";
	TransferFeatureUnsupportedByFormat, "When an instance transfer feature is unsupported by the format", "Transferring %feat is not supported by the format";
	TransferFeatureUnsupportedByPlugin, "When an instance transfer feature is unsupported by the plugin", "Transferring %feat is not supported by the plugin yet";
	TransferModloaderFeature, "Instance transfer modloader feature", "the modloader";
	TransferModsFeature, "Instance transfer modloader feature", "mods";
	TransferLaunchSettingsFeature, "Instance transfer launch settings feature", "launch setttings";
	AuthenticationSuccessful, "When authentication succeeds", "Authentication successful";
	StartInstallingPlugin, "When starting to install a plugin", "Installing plugin";
	FinishInstallingPlugin, "When finishing installing a plugin", "Plugin installed";
	StartExporting, "When starting to export an instance", "Exporting instance '$inst' in format '%format' using plugin '%plugin'";
	ExportPluginNoResult, "When the plugin used for instance export doesn't return anything", "Export plugin did not return a result";
	FinishExporting, "When finishing exporting an instance", "Export finished";
	StartImporting, "When starting to import an instance", "Importing instance '$inst' in format '%format' using plugin '%plugin'";
	ImportPluginNoResult, "When the plugin used for instance import doesn't return anything", "Import plugin did not return a result";
	FinishImporting, "When finishing importing an instance", "Import finished";
}

/// Replaces placeholders in a translated key
pub fn replace_placeholders(string: &str, placeholder_name: &str, value: &str) -> String {
	string.replace(&format!("%{placeholder_name}"), value)
}

/// Utility macro to translate from output
#[macro_export]
macro_rules! translate {
	($o:expr, $key:ident) => {
		$o.translate($crate::lang::translate::TranslationKey::$key).into()
	};

	($o:expr, $key:ident, $($placeholder:literal = $value:expr),+) => {
		{
			let mut out = $o.translate($crate::lang::translate::TranslationKey::$key).to_string();
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
