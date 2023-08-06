pub mod declarative;
pub mod repo;

// Re-export
pub use mcvm_parse as parse;
use serde::{Serialize, Deserialize};

/// Content type of a package
#[derive(Deserialize, Serialize, Debug, Copy, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum PackageContentType {
	#[default]
	Script,
}
