#![warn(missing_docs)]

//! This library is used for parsing MCVM package scripts
//!
//! # Features:
//!
//! - `schema`: Enable generation of JSON schemas using the `schemars` crate

/// Parsing for conditions, used in if instructions
pub mod conditions;
/// Parsing for most instructions, with the exception of a few complex ones
pub mod instruction;
/// Token generation from a string, which is passed into the parser
pub mod lex;
/// General parsing
pub mod parse;
/// Things related to package script routines
pub mod routine;
/// Things related to script variables
pub mod vars;

use std::fmt::Display;

/// Reason why the package reported a failure
#[derive(Debug, Clone)]
pub enum FailReason {
	/// No fail reason is provided
	None,
	/// The Minecraft version is unsupported
	UnsupportedVersion,
	/// The modloader is unsupported
	UnsupportedModloader,
	/// The plugin loader is unsupported
	UnsupportedPluginLoader,
	/// The configured set of features is unsupported
	UnsupportedFeatures,
	/// The operating system is unsupported
	UnsupportedOperatingSystem,
	/// The architecture is unsupported
	UnsupportedArchitecture,
}

impl FailReason {
	/// Parse a FailReason from a string
	pub fn from_string(string: &str) -> Option<Self> {
		match string {
			"unsupported_version" => Some(Self::UnsupportedVersion),
			"unsupported_modloader" => Some(Self::UnsupportedModloader),
			"unsupported_plugin_loader" => Some(Self::UnsupportedPluginLoader),
			"unsupported_features" => Some(Self::UnsupportedFeatures),
			"unsupported_operating_system" => Some(Self::UnsupportedFeatures),
			"unsupported_architecture" => Some(Self::UnsupportedArchitecture),
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
				Self::UnsupportedArchitecture => "Unsupported system architecture",
			}
		)
	}
}
