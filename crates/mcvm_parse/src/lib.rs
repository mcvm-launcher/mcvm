use std::fmt::Display;

/// Parsing for conditions, used in if instructions
pub mod conditions;
/// Parsing for most instructions, with the exception of a few complex ones
pub mod instruction;
/// Token generation from a string, which is passed into the parser
pub mod lex;
/// Package metadata
pub mod metadata;
/// General parsing
pub mod parse;
/// Package properties
pub mod properties;
/// Things related to package script routines
pub mod routine;
/// Things related to script variables
pub mod vars;

/// Reason why the package reported a failure
#[derive(Debug, Clone)]
pub enum FailReason {
	None,
	UnsupportedVersion,
	UnsupportedModloader,
	UnsupportedPluginLoader,
	UnsupportedFeatures,
	UnsupportedOperatingSystem,
}

impl FailReason {
	pub fn from_string(string: &str) -> Option<Self> {
		match string {
			"unsupported_version" => Some(Self::UnsupportedVersion),
			"unsupported_modloader" => Some(Self::UnsupportedModloader),
			"unsupported_plugin_loader" => Some(Self::UnsupportedPluginLoader),
			"unsupported_features" => Some(Self::UnsupportedFeatures),
			"unsupported_operating_system" => Some(Self::UnsupportedFeatures),
			_ => None,
		}
	}
}

impl Display for FailReason {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::None => "",
				Self::UnsupportedVersion => "Unsupported Minecraft version",
				Self::UnsupportedModloader => "Unsupported modloader",
				Self::UnsupportedPluginLoader => "Unsupported plugin loader",
				Self::UnsupportedFeatures => "Unsupported feature set",
				Self::UnsupportedOperatingSystem => "Unsupported operating system",
			}
		)
	}
}
