use serde::{Deserialize, Serialize};

macro_rules! define_translations {
	($($key:ident, $doc:literal, $default:literal);* $(;)?) => {
		/// Keys for translations
		#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
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
	FinishInstallingPackages, "When finishing installing packages", "Packages installed";
	PackageOutOfDate, "When a package is out of date", "Package %pkg has been flagged as out of date";
	PackageDeprecated, "When a package is deprecated", "Package %pkg has been flagged as deprecated";
	PackageInsecure, "When a package is insecure", "Package %pkg has been flagged as insecure";
	PackageMalicious, "When a package is malicious", "Package %pkg has been flagged as malicious";
	PackageSupportHeader, "Header for packages to consider supporting", "Packages to consider supporting";
	StartUpdatingProfile, "When starting to update a profile", "Updating profile %prof";
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
