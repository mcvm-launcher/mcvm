use std::fmt::Display;

pub mod conditions;
pub mod instruction;
pub mod lex;
pub mod metadata;
pub mod parse;
pub mod properties;
pub mod routine;
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
